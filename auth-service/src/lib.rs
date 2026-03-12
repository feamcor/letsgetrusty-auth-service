use crate::app_state::AppState;
use crate::domain::Secret;
use axum::http::Method;
use axum::routing::get;
use axum::routing::post;
use axum::serve::Serve;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;

pub mod app_state;
pub mod config;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

#[derive(Debug)]
pub struct Application {
    server: Serve<tokio::net::TcpListener, axum::Router, axum::Router>,
    pub address: SocketAddr,
}

impl Application {
    #[tracing::instrument(name = "ApplicationBuild", level = tracing::Level::TRACE, skip_all
    )]
    pub async fn build(state: AppState, address: SocketAddr) -> color_eyre::eyre::Result<Self> {
        let config = &state.config.inner();
        // Allow the app service to call the auth service
        let allowed_origins = [format!("http://{}:{}", address.ip(), config.app_service_port).parse()?];
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
        let router = axum::Router::new()
            .fallback_service(assets_dir)
            .nest("/api", apis)
            .with_state(state)
            .layer(cors)
            .layer(
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

    #[tracing::instrument(name = "ApplicationRun", level = tracing::Level::TRACE, skip_all
    )]
    pub async fn run(self) -> std::io::Result<()> {
        tracing::info!("Server listening on {}", self.address);
        let shutdown_token = CancellationToken::new();
        self.server
            .with_graceful_shutdown(shutdown_signal(shutdown_token))
            .await
    }
}

#[tracing::instrument(name = "ApplicationShutdown", level = tracing::Level::TRACE, skip_all
)]
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

#[tracing::instrument(name = "GetDatabasePool", level = tracing::Level::TRACE, skip_all
)]
pub async fn database_pool(
    db_url: &Secret,
    db_pool_min_size: u32,
    db_pool_max_size: u32,
) -> sqlx::Result<sqlx::PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .min_connections(db_pool_min_size)
        .max_connections(db_pool_max_size)
        .connect(db_url.expose())
        .await
}

#[tracing::instrument(name = "ConfigureDatabase", level = tracing::Level::TRACE, skip_all
)]
pub async fn configure_database(
    db_url: &Secret,
    db_pool_min_size: u32,
    db_pool_max_size: u32,
) -> sqlx::Result<sqlx::PgPool> {
    let pool = database_pool(db_url, db_pool_min_size, db_pool_max_size).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[tracing::instrument(name = "GetCacheClient", level = tracing::Level::TRACE, skip_all)]
pub fn cache_client(cache_url: &Secret) -> redis::RedisResult<redis::Client> {
    redis::Client::open(cache_url.expose())
}

#[tracing::instrument(name = "ConfigureCache", level = tracing::Level::TRACE, skip_all)]
pub fn configure_cache(cache_url: &Secret) -> redis::RedisResult<redis::Connection> {
    let client = cache_client(cache_url)?;
    client.get_connection()
}
