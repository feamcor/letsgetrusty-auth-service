use crate::app_state::AppState;
use axum::routing::post;
use axum::serve::Serve;
use axum::Router;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
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
    #[instrument(level = "trace")]
    pub async fn build(state: AppState, address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let assets_dir =
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));
        let apis = Router::new()
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/logout", post(routes::logout))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::verify_token));
        let router = Router::new()
            .fallback_service(assets_dir)
            .nest("/api", apis)
            .with_state(state)
            .layer(TraceLayer::new_for_http());
        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let server = axum::serve(listener, router);
        let application = Self { server, address };
        Ok(application)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        info!("listening on {}", &self.address);
        self.server.await
    }
}
