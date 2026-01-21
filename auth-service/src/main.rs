use std::net::SocketAddr;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use auth_service::Application;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_span_events(fmt::format::FmtSpan::CLOSE))
        .init();
    let socket_addr = SocketAddr::from(([0,0,0,0], 3000));
    Application::build(socket_addr)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app")
}
