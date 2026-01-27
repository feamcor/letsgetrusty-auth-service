use auth_service::app_state::AppState;
use auth_service::services::HashmapUserStore;
use auth_service::Application;
use std::net::SocketAddr;
use std::sync::Arc;
use dotenvy::dotenv_override;
use fmt::format::FmtSpan;
use tokio::sync::RwLock;
use tracing::{enabled, info, trace, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    let dotenv_loaded = dotenv_override().ok();
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .init();
    info!("Initialized: Tracing");
    if dotenv_loaded.is_some() {
        info!("Loaded file: {}", dotenv_loaded.unwrap().display());
    }
    if enabled!(Level::TRACE) {
        for var in dotenvy::vars() {
            trace!("Loaded envar: {}={}", var.0, var.1);
        }
    }
    let user_store = HashmapUserStore::default();
    info!("Initialized: User store");
    let app_state = AppState::new(Arc::new(RwLock::new(user_store)));
    info!("Initialized: App state");
    let socket_addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Initialized: Listening address: {}", socket_addr);
    Application::build(app_state, socket_addr)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app")
}
