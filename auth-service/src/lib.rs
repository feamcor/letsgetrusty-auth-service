use std::error::Error;
use std::net::SocketAddr;
use axum::Router;
use axum::routing::post;
use axum::serve::Serve;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

pub mod routes;

#[derive(Debug)]
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    pub address: SocketAddr, // exposed as public for access in tests
}

impl Application {
    #[instrument]
    pub async fn build(address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let assets_dir = ServeDir::new("assets")
            .not_found_service(ServeFile::new("assets/index.html"));
        let apis = Router::new()
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/logout", post(routes::logout))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::verify_token));
        let router = Router::new()
            .fallback_service(assets_dir)
            .nest("/api", apis)
            .layer(TraceLayer::new_for_http());
        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let server = axum::serve(listener, router);
        let application = Self { server, address };
        Ok(application)
    }

    #[instrument]
    pub async fn run(self) -> Result<(), std::io::Error> {
        info!("listening on {}", &self.address);
        self.server.await
    }
}
