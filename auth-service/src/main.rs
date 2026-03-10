use auth_service::Application;
use auth_service::app_state::AppState;
use auth_service::config::Config;
use auth_service::config::ConfigType;
use auth_service::config::StoreEngine;
use auth_service::configure_cache;
use auth_service::configure_database;
use auth_service::services::BannedTokenStoreType;
use auth_service::services::EmailClientType;
use auth_service::services::HashmapTwoFactorAuthCodeStore;
use auth_service::services::HashmapUserStore;
use auth_service::services::HashsetBannedTokenStore;
use auth_service::services::MockEmailClient;
use auth_service::services::PostgresUserStore;
use auth_service::services::RedisBannedTokenStore;
use auth_service::services::RedisTwoFactorAuthCodeStore;
use auth_service::services::TwoFactorAuthCodeStoreType;
use auth_service::services::UserStoreType;
use auth_service::utils::tracing::init_tracing;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");

    let config = Config::init_from_env_and_cli();
    init_tracing(&config.log).expect("Failed to initialize tracing");
    let config_type = ConfigType::new(config);
    let config = config_type.inner();

    let user_store_type = match config.store_engine {
        StoreEngine::Ephemeral => UserStoreType::new(HashmapUserStore::default()),
        StoreEngine::Server => {
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
    tracing::info!(
        "Initialized: User Store: {}: {:?}",
        config.store_engine,
        user_store_type
    );

    let banned_token_store_type = match config.store_engine {
        StoreEngine::Ephemeral => BannedTokenStoreType::new(HashsetBannedTokenStore::default()),
        StoreEngine::Server => {
            let connection = configure_cache(&config.cache_url()).expect("Failed to configure cache");
            let connection = RwLock::new(connection);
            BannedTokenStoreType::new(RedisBannedTokenStore::new(connection))
        }
    };
    tracing::info!(
        "Initialized: Banned Token Store: {}: {:?}",
        config.store_engine,
        banned_token_store_type
    );

    let two_factor_auth_code_store_type = match config.store_engine {
        StoreEngine::Ephemeral => TwoFactorAuthCodeStoreType::new(HashmapTwoFactorAuthCodeStore::default()),
        StoreEngine::Server => {
            let connection = configure_cache(&config.cache_url()).expect("Failed to configure cache");
            let connection = RwLock::new(connection);
            TwoFactorAuthCodeStoreType::new(RedisTwoFactorAuthCodeStore::new(connection))
        }
    };
    tracing::info!(
        "Initialized: Two-Factor Auth Code Store: {}: {:?}",
        config.store_engine,
        two_factor_auth_code_store_type
    );

    let email_client_type = EmailClientType::new(MockEmailClient);
    tracing::info!("Initialized: Email Client");

    let ip_address = if let Some(v6) = config.ipv6 {
        IpAddr::V6(v6)
    } else if let Some(v4) = config.ipv4 {
        IpAddr::V4(v4)
    } else {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    };
    let socket_addr = SocketAddr::new(ip_address, config.port);
    tracing::info!("Initialized: Listening address: {}", socket_addr);

    let app_state = AppState::new(
        user_store_type,
        banned_token_store_type,
        two_factor_auth_code_store_type,
        email_client_type,
        config_type,
    );
    tracing::info!("Initialized: App State");

    Application::build(app_state, socket_addr)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app");
}
