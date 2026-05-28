# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository layout

Two **independent Cargo packages** (not a workspace) at the repo root:

- `auth-service/` — the core auth microservice (axum), with PostgreSQL + Redis backends and email-based 2FA.
- `app-service/` — a small axum/askama web frontend that calls `auth-service` to manage sessions and protect `/protected`.

Each crate has its own `Cargo.toml`, `Cargo.lock`, `Dockerfile`, and `target/`. **All `cargo` commands must be run from inside one of these directories**, not the repo root. Rust edition is `2024`.

## Common commands

Build / run / test (run from the relevant crate dir, e.g. `cd auth-service`):

```bash
cargo build
cargo run
cargo test                                # all unit + integration tests
cargo test --test api                     # only the integration-test binary (tests/api/main.rs)
cargo test --test api -- login::test_x    # single integration test by path
cargo test password::                     # filter unit tests by module path
cargo fmt                                 # uses repo-root rustfmt.toml (max_width=120, vertical imports)
cargo clippy --all-targets
```

Full stack (PostgreSQL + Redis + both services) via Docker Compose:

```bash
./docker.sh                # docker compose build && up, sourcing auth-service/.env
docker compose up --build  # equivalent if env is already exported
```

`compose.override.yml` switches both services from the prebuilt Docker Hub images to local builds (`build.context: ./auth-service` / `./app-service`).

## sqlx offline data

`auth-service` uses `sqlx::query!`/`query_as!` macros that need a live DB at compile time **unless** `SQLX_OFFLINE=true` is set and `auth-service/.sqlx/` contains cached query metadata. CI sets `SQLX_OFFLINE=true`.

After changing any SQL query, regenerate the offline cache:

```bash
cd auth-service
# DATABASE_URL must point at a DB with migrations applied
cargo sqlx prepare -- --tests
```

Migrations live in `auth-service/migrations/` and run automatically on startup via `configure_store` → `sqlx::migrate!()`. The connection string forces `search_path=auth,public`, so all tables are under the `auth` schema.

## Architecture: auth-service

The service is wired as a **pluggable AppState** assembled in `src/main.rs` from `Config` + `clap`/`dotenvy`. Each dependency has a trait + multiple implementations selected by config:

| State field | Trait module | Backends (selected by env) |
|---|---|---|
| `user_store` | `services::store_users` | `HashmapUserStore` (memory) / `PostgresUserStore` — `AUTH_SERVICE_DB_ENGINE` |
| `banned_token_store` | `services::store_banned_tokens` | `HashsetBannedTokenStore` / `RedisBannedTokenStore` — `AUTH_SERVICE_CACHE_ENGINE` |
| `two_factor_auth_code_store` | `services::store_tfa_codes` | `HashmapTwoFactorAuthCodeStore` / `RedisTwoFactorAuthCodeStore` — same cache engine |
| `email_client` | `services::client_email` | `MockEmailClient` / `PostmarkEmailClient` — `AUTH_SERVICE_EMAIL_SERVICE` |

`AppState` is `Clone` because every concrete store/client is wrapped in an `Arc<RwLock<…>>` newtype (`UserStoreType`, `BannedTokenStoreType`, etc., defined in `services::stores` / `services::clients`) — clone the wrapper, not the impl. Get the inner trait object with `.inner()` before calling its async methods.

When adding a new backend for an existing trait: implement the trait, add a variant to the relevant `*Engine` enum in `src/config/*.rs`, branch on it in `main.rs`, and add the matching arm in `tests/api/helpers.rs::TestApp::new`.

### Module layout

- `src/lib.rs` — `Application::build` wires the axum router (`/api/{health,signup,login,logout,verify-2fa,verify-token}` + static `assets/` fallback), CORS, tracing layer, graceful shutdown. Also exposes `configure_store` / `configure_cache` helpers used by both `main.rs` and tests.
- `src/config/` — one file per concern (`network`, `jwt`, `tfa`, `database`, `cache`, `email`, `log`). Each is a `clap::Args` flattened into the top-level `Config`. `Config::init_from_env_and_cli()` is used in `main.rs`; `Config::init_from_env()` (no CLI parsing) is used in tests. All env vars are namespaced `AUTH_SERVICE_*`. `load_mandatory_arguments()` panics if a required secret (DB password, JWT secret, email API key/sender) is missing.
- `src/domain/` — newtypes that own all input validation: `Email`, `HashedPassword` (Argon2 + zxcvbn strength check), `Token`, `LoginAttemptId` (UUID), `TwoFactorAuthCode`, `User`, and `Secret`.
- `src/routes/` — one file per HTTP handler. Handlers return `ApiResult<…>` from `utils::api_error`.
- `src/services/` — store + client trait implementations (see table above).
- `src/utils/` — `api_error` (typed `ApiError` → HTTP response mapping), `auth` (JWT encode/decode, cookie helpers, `JWT_COOKIE_NAME = "jwt"`), `tracing` (request-id span propagation).

### `Secret` newtype

`domain::Secret` wraps `secrecy::SecretString` and is the only correct way to carry passwords, tokens, JWT secrets, DB URLs, API keys, etc. through the codebase — its `Display`/`Debug` redacts the value, and `Serialize` is implemented for round-tripping. Always pass `Secret` instead of `String`/`&str` for sensitive data and use `.expose()` only at the boundary (sqlx connection, JWT encode/decode, outbound HTTP).

## Architecture: app-service

Single-file axum app (`src/main.rs`) with two routes:
- `GET /` — renders `templates/index.html` via askama; the login/logout links point at `auth-service` (host from `AUTH_SERVICE_IP`, then `AUTH_SERVICE_HOST_NAME` inside Docker).
- `GET /protected` — reads the `jwt` cookie, POSTs it to `auth-service`'s `/api/verify-token`, and returns 401/200 accordingly.

## Integration tests (`auth-service/tests/api/`)

`main.rs` is the test binary entrypoint; one module per route (`login.rs`, `signup.rs`, …) plus shared `helpers.rs`. Each test uses `test_context::AsyncTestContext` via `TestAppAsyncContext`:

- A fresh PostgreSQL database is created **per test** (name = UUIDv7), migrated, and dropped on teardown — including terminating active connections. `db_url` must point at a real Postgres for these to run; with `AUTH_SERVICE_DB_ENGINE=memory` they fall back to the in-memory store.
- When `AUTH_SERVICE_EMAIL_SERVICE=postmark`, a `wiremock::MockServer` stands in for the Postmark API and is exposed as `TestApp::email_server`.
- The auth service binds to an ephemeral port (`SocketAddr::from((Ipv4Addr::LOCALHOST, 0))`) and is driven via `TestApp::http_client` (a `reqwest::Client` with a shared cookie jar). Use the `post_signup` / `post_login` / `post_logout` / `post_verify_2fa` / `post_verify_token` helpers rather than hand-rolling URLs.

Tests require the same env vars as the running service (DB password, JWT secret, etc.); a `.env` in `auth-service/` is picked up via `dotenvy`.

## Configuration

All runtime config is driven by `AUTH_SERVICE_*` env vars; see `src/config/*.rs` for the exhaustive list and defaults. CLI flags mirror the env vars (`clap` derive). The `compose.yml` `auth-service` block is the canonical reference for what must be set in production.
