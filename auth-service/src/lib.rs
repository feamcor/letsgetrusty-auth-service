use crate::app_state::AppState;
use crate::domain::Secret;
use axum::http::HeaderName;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::routing::get;
use axum::routing::post;
use axum::serve::Serve;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

pub mod app_state;
pub mod config;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

/// Static security headers set on every response. Defense-in-depth — none guard against logic
/// bugs in this service, but they neutralize common browser-side footguns. Keep this list as
/// the single source of truth so tests can iterate it and future additions are one row each.
pub const SECURITY_HEADERS: &[(&str, &str)] = &[
    ("x-content-type-options", "nosniff"),
    ("x-frame-options", "DENY"),
    ("referrer-policy", "strict-origin-when-cross-origin"),
];

#[derive(Debug)]
pub struct Application {
    server: Serve<tokio::net::TcpListener, axum::Router, axum::Router>,
    pub address: SocketAddr,
}

impl Application {
    #[tracing::instrument(name = "ApplicationBuild", level = tracing::Level::TRACE, skip_all)]
    pub async fn build(state: AppState, address: SocketAddr) -> color_eyre::eyre::Result<Self> {
        let config = &state.config.inner();
        // The CORS allow-origin must match the browser's Origin header (scheme + host + port of
        // the app-service as seen by the user-agent), NOT auth-service's bind address — those
        // diverge under 0.0.0.0 / Docker. Use the configured override, or `localhost:{app_port}`.
        // Use `.origin().ascii_serialization()` so we strip any accidental path/query/fragment
        // from the configured URL (browsers only send scheme://host[:port] as Origin).
        let allowed_origin = config.network.resolved_allowed_origin();
        let allowed_origin_value = allowed_origin.origin().ascii_serialization();
        tracing::info!("CORS allow-origin: {}", allowed_origin_value);
        let allowed_origins = [allowed_origin_value.parse()?];
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);
        let assets_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));
        tracing::info!("Initialized: Assets directory");
        let apis = axum::Router::new()
            .route("/health", get(routes::health))
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/logout", post(routes::logout))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::verify_token));
        tracing::info!("Initialized: API routes");
        let mut router = axum::Router::new()
            .fallback_service(assets_dir)
            .nest("/api", apis)
            .with_state(state)
            .layer(cors);
        for (name, value) in SECURITY_HEADERS {
            router = router.layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static(name),
                HeaderValue::from_static(value),
            ));
        }
        let router = router.layer(
            TraceLayer::new_for_http()
                .make_span_with(utils::tracing::make_span_with_request_id)
                .on_request(utils::tracing::on_request)
                .on_response(utils::tracing::on_response),
        );
        tracing::info!("Initialized: Router");
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        tracing::info!("Initialized: Listener");
        let server = axum::serve(listener, router);
        tracing::info!("Initialized: Server");
        let application = Self { server, address };
        tracing::info!("Initialized: Application");
        Ok(application)
    }

    #[tracing::instrument(name = "ApplicationRun", level = tracing::Level::TRACE, skip_all)]
    pub async fn run(self) -> std::io::Result<()> {
        tracing::info!("Server listening on {}", self.address);
        let shutdown_token = CancellationToken::new();
        self.server
            .with_graceful_shutdown(shutdown_signal(shutdown_token))
            .await
    }
}

#[tracing::instrument(name = "ApplicationShutdown", level = tracing::Level::TRACE, skip_all)]
async fn shutdown_signal(shutdown_token: CancellationToken) {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::SignalKind;
        use tokio::signal::unix::signal;
        let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => { tracing::warn!("Received CTRL+C"); },
        () = terminate => { tracing::warn!("Received SIGTERM"); },
        () = shutdown_token.cancelled() => { tracing::warn!("Shutdown triggered by application"); },
    }

    tracing::info!("Shutdown signal received!");
}

#[tracing::instrument(name = "GetStorePool", level = tracing::Level::TRACE, skip_all)]
pub async fn get_store_connection_pool(url: &Secret, pool_min: u32, pool_max: u32) -> sqlx::Result<sqlx::PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .min_connections(pool_min)
        .max_connections(pool_max)
        .connect(url.expose())
        .await
}

#[tracing::instrument(name = "ConfigureStore", level = tracing::Level::TRACE, skip_all)]
pub async fn configure_store(url: &Secret, pool_min: u32, pool_max: u32) -> sqlx::Result<sqlx::PgPool> {
    let pool = get_store_connection_pool(url, pool_min, pool_max).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[tracing::instrument(name = "GetCacheClient", level = tracing::Level::TRACE, skip_all)]
pub fn get_cache_client(url: &Secret) -> redis::RedisResult<redis::Client> {
    redis::Client::open(url.expose())
}

#[tracing::instrument(name = "ConfigureCache", level = tracing::Level::TRACE, skip_all)]
pub async fn configure_cache(url: &Secret) -> redis::RedisResult<redis::aio::MultiplexedConnection> {
    let client = get_cache_client(url)?;
    // MultiplexedConnection is Clone and internally synchronizes concurrent commands, so the same
    // handle can be shared across stores without an outer RwLock.
    client.get_multiplexed_async_connection().await
}
