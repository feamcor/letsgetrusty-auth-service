use crate::app_state::AppState;
use axum::http::Method;
use axum::routing::{get, post};
use axum::serve::Serve;
use std::error::Error;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};
use tracing::{warn, Level};

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
    #[instrument(level = Level::TRACE, skip(state))]
    pub async fn build(state: AppState, address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let config = &state.config.inner();
        // Allow the app service to call the auth service
        let allowed_origins =
            [format!("http://{}:{}", address.ip(), config.app_service_port).parse()?];
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);
        let assets_dir =
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));
        info!("Initialized: Assets directory");
        let apis = axum::Router::new()
            .route("/health", get(routes::health))
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/logout", post(routes::logout))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::verify_token));
        info!("Initialized: API routes");
        let router = axum::Router::new()
            .fallback_service(assets_dir)
            .nest("/api", apis)
            .with_state(state)
            .layer(cors)
            .layer(TraceLayer::new_for_http());
        info!("Initialized: Router");
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        info!("Initialized: Listener");
        let server = axum::serve(listener, router);
        info!("Initialized: Server");
        let application = Self { server, address };
        info!("Initialized: Application");
        Ok(application)
    }

    #[instrument(level = Level::TRACE, skip(self))]
    pub async fn run(self) -> Result<(), std::io::Error> {
        info!("Server listening on {}", self.address);
        let shutdown_token = CancellationToken::new();
        self.server
            .with_graceful_shutdown(shutdown_signal(shutdown_token))
            .await
    }
}

#[instrument(level = Level::TRACE, skip(shutdown_token))]
async fn shutdown_signal(shutdown_token: CancellationToken) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => { warn!("Received CTRL+C"); },
        () = terminate => { warn!("Received SIGTERM"); },
        () = shutdown_token.cancelled() => { warn!("Shutdown triggered by application"); },
    }

    info!("Shutdown signal received!");
}

#[instrument(level = Level::TRACE)]
pub async fn database_pool(
    db_url: &str,
    db_pool_min_size: u32,
    db_pool_max_size: u32,
) -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .min_connections(db_pool_min_size)
        .max_connections(db_pool_max_size)
        .connect(db_url)
        .await
}

#[instrument(level = Level::TRACE)]
pub async fn configure_database(
    db_url: &str,
    db_pool_min_size: u32,
    db_pool_max_size: u32,
) -> Result<sqlx::PgPool, sqlx::Error> {
    let pool = database_pool(db_url, db_pool_min_size, db_pool_max_size).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[instrument(level = Level::TRACE)]
pub fn cache_client(cache_url: &str) -> redis::RedisResult<redis::Client> {
    redis::Client::open(cache_url)
}

#[instrument(level = Level::TRACE)]
pub fn configure_cache(cache_url: &str) -> redis::RedisResult<redis::Connection> {
    let client = cache_client(cache_url)?;
    client.get_connection()
}
