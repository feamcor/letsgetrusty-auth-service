use crate::app_state::AppState;
use axum::routing::post;
use axum::serve::Serve;
use axum::Router;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing::{info, instrument};

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;

#[derive(Debug)]
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    pub address: SocketAddr, // exposed as public for access in tests
}

impl Application {
    #[instrument(level = Level::INFO, skip(state))]
    pub async fn build(state: AppState, address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let assets_dir =
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));
        info!("Initialized: Assets directory");
        let apis = Router::new()
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

    #[instrument(level = Level::INFO, skip(self))]
    pub async fn run(self) -> Result<(), std::io::Error> {
        info!("Server listening on {}", self.address);
        self.server.await
    }
}
