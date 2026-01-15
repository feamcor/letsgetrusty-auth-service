use std::net::SocketAddr;
use auth_service::Application;

#[tokio::main]
async fn main() {
    let address = SocketAddr::from(([0,0,0,0], 3000));
    let app = Application::build(address)
        .await
        .expect("Failed to build app");
    app.run().await.expect("Failed to run app");
}
