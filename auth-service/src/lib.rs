use std::error::Error;
use std::net::SocketAddr;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::{get, post};
use axum::serve::Serve;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: SocketAddr,
}

impl Application {
    pub async fn build(address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let assets_dir = ServeDir::new("assets")
            .not_found_service(ServeFile::new("assets/index.html"));
        let apis = Router::new()
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token));
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

    pub async fn run(self) -> Result<(), std::io::Error> {
        info!("listening on {}", &self.address);
        self.server.await
    }
}

#[instrument]
async fn signup() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}

#[instrument]
async fn login() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}

#[instrument]
async fn logout() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}

#[instrument]
async fn verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}

#[instrument]
async fn verify_token() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}
