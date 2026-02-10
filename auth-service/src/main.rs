mod config;

use crate::config::Config;
use auth_service::Application;
use auth_service::app_state::AppState;
use auth_service::services::HashmapUserStore;
use clap::Parser;
use dotenvy::dotenv_override;
use fmt::format::FmtSpan;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    let dotenv = dotenv_override().ok();
    let config = Config::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::from(config.log.to_string()))
        .with(fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .init();
    info!("Initialized: Tracing");

    if let Some(dotenv) = dotenv {
        info!("Initialized: {}", dotenv.display());
    }
    info!("Initialized: {}", config);

    let user_store = HashmapUserStore::default();
    info!("Initialized: User store");

    let app_state = AppState::new(Arc::new(RwLock::new(user_store)));
    info!("Initialized: App state");

    let ip_address = if let Some(v6) = config.ipv6 {
        IpAddr::V6(v6)
    } else if let Some(v4) = config.ipv4 {
        IpAddr::V4(v4)
    } else {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    };
    let socket_addr = SocketAddr::new(ip_address, config.port);
    info!("Initialized: Listening address: {}", socket_addr);

    Application::build(app_state, socket_addr, config.app_service_port)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app");
}
