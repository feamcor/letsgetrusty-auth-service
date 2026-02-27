use auth_service::app_state::AppState;
use auth_service::config::Config;
use auth_service::services::{
    HashmapTwoFactorAuthCodeStore, HashmapUserStore, HashsetBannedTokenStore, MockEmailClient,
};
use auth_service::Application;
use fmt::format::FmtSpan;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, reload};

#[tokio::main]
async fn main() {
    let (filter, reload_handle) = reload::Layer::new(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .init();
    info!("Initialized: Tracing");

    let config = Config::init_from_env_and_cli();

    let log_level = config.log.clone();
    reload_handle
        .modify(|level_filter| *level_filter = LevelFilter::from_level(log_level.into()))
        .expect("Failed to modify log level filter");

    let user_store = HashmapUserStore::default();
    info!("Initialized: User Store");

    let banned_token_store = HashsetBannedTokenStore::default();
    info!("Initialized: Banned Token Store");

    let two_factor_auth_code_store = HashmapTwoFactorAuthCodeStore::default();
    info!("Initialized: Two Factor Auth Code Store");

    let email_client = MockEmailClient;
    info!("Initialized: Email Client");

    let ip_address = if let Some(v6) = config.ipv6 {
        IpAddr::V6(v6)
    } else if let Some(v4) = config.ipv4 {
        IpAddr::V4(v4)
    } else {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    };
    let socket_addr = SocketAddr::new(ip_address, config.port);
    info!("Initialized: Listening address: {}", socket_addr);

    let app_state = AppState::new(
        Arc::new(user_store),
        Arc::new(banned_token_store),
        Arc::new(two_factor_auth_code_store),
        Arc::new(email_client),
        Arc::new(config),
    );
    info!("Initialized: App State");

    Application::build(app_state, socket_addr)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app");
}
