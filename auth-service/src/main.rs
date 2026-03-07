use auth_service::app_state::AppState;
use auth_service::config::{Config, ConfigType, StoreEngine};
use auth_service::services::{
    BannedTokenStoreType, EmailClientType, HashmapTwoFactorAuthCodeStore, HashmapUserStore,
    HashsetBannedTokenStore, MockEmailClient, PostgresUserStore, TwoFactorAuthCodeStoreType,
    UserStoreType,
};
use auth_service::{configure_database, Application};
use fmt::format::FmtSpan;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
    let config_type = ConfigType::new(config);
    let config = config_type.inner();

    let log_level = config.log.clone();
    reload_handle
        .modify(|level_filter| *level_filter = LevelFilter::from_level(log_level.into()))
        .expect("Failed to modify log level filter");

    let user_store_type = match config.store_engine {
        StoreEngine::Memory => {
            info!("Initialized: Memory User Store");
            UserStoreType::new(HashmapUserStore::default())
        }
        StoreEngine::Database => {
            info!("Initialized: Database User Store");
            let pool = configure_database(
                &config.database_url(None),
                config.db_pool_min_size,
                config.db_pool_max_size,
            )
            .await
            .expect("Failed to configure database");
            UserStoreType::new(PostgresUserStore::new(pool))

        }
    };

    let banned_token_store_type = BannedTokenStoreType::new(HashsetBannedTokenStore::default());
    info!("Initialized: Banned Token Store");

    let two_factor_auth_code_store_type =
        TwoFactorAuthCodeStoreType::new(HashmapTwoFactorAuthCodeStore::default());
    info!("Initialized: Two Factor Auth Code Store");

    let email_client_type = EmailClientType::new(MockEmailClient);
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
        user_store_type,
        banned_token_store_type,
        two_factor_auth_code_store_type,
        email_client_type,
        config_type,
    );
    info!("Initialized: App State");

    Application::build(app_state, socket_addr)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app");
}
