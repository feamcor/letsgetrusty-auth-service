use auth_service::app_state::AppState;
use auth_service::services::HashmapUserStore;
use auth_service::Application;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with(fmt::layer().with_span_events(fmt::format::FmtSpan::FULL))
        .init();
    let user_store = HashmapUserStore::default();
    let app_state = AppState::new(Arc::new(RwLock::new(user_store)));
    let socket_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    Application::build(app_state, socket_addr)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app")
}
