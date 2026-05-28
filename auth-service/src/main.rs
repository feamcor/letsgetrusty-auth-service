use auth_service::Application;
use auth_service::app_state::AppState;
use auth_service::config;
use auth_service::config::cache::CacheEngine;
use auth_service::config::database::DatabaseEngine;
use auth_service::config::email::EmailService;
use auth_service::configure_cache;
use auth_service::configure_store;
use auth_service::services::BannedTokenStoreType;
use auth_service::services::EmailClientType;
use auth_service::services::HashmapTwoFactorAuthCodeStore;
use auth_service::services::HashmapUserStore;
use auth_service::services::HashsetBannedTokenStore;
use auth_service::services::MockEmailClient;
use auth_service::services::PostgresUserStore;
use auth_service::services::PostmarkEmailClient;
use auth_service::services::RedisBannedTokenStore;
use auth_service::services::RedisTwoFactorAuthCodeStore;
use auth_service::services::TwoFactorAuthCodeStoreType;
use auth_service::services::UserStoreType;
use auth_service::utils::tracing::init_tracing;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");

    let config = config::Config::init_from_env_and_cli();
    init_tracing(&config.log.level).expect("Failed to initialize tracing");
    let config_type = config::ConfigType::new(config);
    let config = config_type.inner();

    let user_store_type = match config.db.db_engine {
        DatabaseEngine::Memory => UserStoreType::new(HashmapUserStore::default()),
        DatabaseEngine::Postgres => {
            let pool = configure_store(&config.db.db_url(None), config.db.db_pool_min, config.db.db_pool_max)
                .await
                .expect("Failed to configure database");
            let store = PostgresUserStore::new(pool);
            UserStoreType::new(store)
        }
    };
    tracing::info!("Initialized: User Store: {}: {:?}", config.db, user_store_type);

    // Open a single multiplexed Redis connection up-front and clone it into both stores. The
    // connection is internally synchronized so the same handle can serve concurrent commands
    // from both stores without an outer RwLock.
    let cache_connection = match config.cache.cache_engine {
        CacheEngine::Memory => None,
        CacheEngine::Redis => Some(
            configure_cache(&config.cache.cache_url())
                .await
                .expect("Failed to configure cache"),
        ),
    };

    let banned_token_store_type = match config.cache.cache_engine {
        CacheEngine::Memory => {
            BannedTokenStoreType::new(HashsetBannedTokenStore::new(u64::from(config.jwt.jwt_ttl)))
        }
        CacheEngine::Redis => {
            let connection = cache_connection.clone().expect("cache connection initialised above");
            let store = RedisBannedTokenStore::new(connection, u64::from(config.jwt.jwt_ttl));
            BannedTokenStoreType::new(store)
        }
    };
    tracing::info!(
        "Initialized: Banned Token Store: {}: {:?}",
        config.cache,
        banned_token_store_type
    );

    let two_factor_auth_code_store_type = match config.cache.cache_engine {
        CacheEngine::Memory => {
            TwoFactorAuthCodeStoreType::new(HashmapTwoFactorAuthCodeStore::new(u64::from(config.tfa.tfa_ttl)))
        }
        CacheEngine::Redis => {
            let connection = cache_connection.clone().expect("cache connection initialised above");
            let store = RedisTwoFactorAuthCodeStore::new(connection, u64::from(config.tfa.tfa_ttl));
            TwoFactorAuthCodeStoreType::new(store)
        }
    };
    tracing::info!(
        "Initialized: Two-Factor Auth Code Store: {}: {:?}",
        config.cache,
        two_factor_auth_code_store_type
    );

    let email_client_type = match config.email.email_service {
        EmailService::Mock => EmailClientType::new(MockEmailClient),
        EmailService::Postmark => {
            let timeout = std::time::Duration::from_millis(u64::from(config.email.email_api_timeout));
            let http_client = reqwest::Client::builder().timeout(timeout).build().unwrap();
            let client = PostmarkEmailClient::new(
                http_client,
                config.email.email_api_key.clone().unwrap(),
                config.email.email_api_url.clone(),
                config.email.email_stream.clone(),
                config.email.email_sender.clone().unwrap(),
            );
            EmailClientType::new(client)
        }
    };
    tracing::info!("Initialized: Email Client: {}: {:?}", config.email, email_client_type);

    let ip_address = if let Some(v6) = config.network.ipv6 {
        IpAddr::V6(v6)
    } else if let Some(v4) = config.network.ipv4 {
        IpAddr::V4(v4)
    } else {
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    };
    let socket_addr = SocketAddr::new(ip_address, config.network.port);
    tracing::info!("Initialized: Listening address: {}", socket_addr);

    let app_state = AppState::new(
        user_store_type,
        banned_token_store_type,
        two_factor_auth_code_store_type,
        email_client_type,
        config_type,
    );
    tracing::info!("Initialized: App State");

    // Pre-compute the user-enumeration decoy hash on the blocking pool so the first login that
    // hits the UserNotFound branch doesn't pay ~50–100 ms of Argon2id on a tokio worker thread.
    tokio::task::spawn_blocking(auth_service::services::warm_decoy_password_hash)
        .await
        .expect("decoy warm-up task panicked");
    tracing::info!("Initialized: decoy password hash");

    Application::build(app_state, socket_addr)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app");
}
