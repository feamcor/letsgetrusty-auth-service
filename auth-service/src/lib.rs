use crate::app_state::AppState;
use axum::http::Method;
use axum::routing::{get, post};
use axum::serve::Serve;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::{warn, Level};
use tracing::{info, instrument};

pub mod app_state;
pub mod config;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

#[derive(Debug)]
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    pub address: SocketAddr,
}

impl Application {
    #[instrument(level = Level::TRACE, skip(state))]
    pub async fn build(state: AppState, address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let config = &state.config;
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
        let apis = Router::new()
            .route("/health", get(routes::health))
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/logout", post(routes::logout))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::verify_token));
        info!("Initialized: API routes");
        let router = Router::new()
            .fallback_service(assets_dir)
            .nest("/api", apis)
            .with_state(state)
            .layer(cors)
            .layer(TraceLayer::new_for_http());
        info!("Initialized: Router");
        let listener = TcpListener::bind(address).await?;
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
        self.server.with_graceful_shutdown(shutdown_signal(shutdown_token)).await
    }
}

#[instrument(level = Level::TRACE, skip(shutdown_token))]
async fn shutdown_signal(shutdown_token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c()
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
pub async fn get_database_pool(
    url: &str,
    min_pool_size: u32,
    max_pool_size: u32,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .min_connections(min_pool_size)
        .max_connections(max_pool_size)
        .connect(url)
        .await
}