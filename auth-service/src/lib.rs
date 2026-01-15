use std::error::Error;
use std::net::SocketAddr;
use axum::Router;
use axum::serve::Serve;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: SocketAddr,
}

impl Application {
    pub async fn build(address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let assets_dir = ServeDir::new("assets");
        let router = Router::new().fallback_service(assets_dir);
        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let server = axum::serve(listener, router);
        let application = Self { server, address };
        Ok(application)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}