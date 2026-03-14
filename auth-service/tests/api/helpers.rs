use auth_service::app_state::AppState;
use auth_service::config::cache::CacheEngine;
use auth_service::config::database::DatabaseEngine;
use auth_service::config::email::EmailService;
use auth_service::config::Config;
use auth_service::config::ConfigType;
use auth_service::configure_cache;
use auth_service::domain::Email;
use auth_service::domain::Secret;
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
use auth_service::Application;
use axum::http::Uri;
use reqwest::cookie::Jar;
use reqwest::Client;
use reqwest::Response;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::Connection;
use sqlx::Executor;
use sqlx::PgConnection;
use sqlx::PgPool;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use test_context::AsyncTestContext;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub base_url: String,
    pub cookie_jar: std::sync::Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
    pub two_factor_auth_code_store: TwoFactorAuthCodeStoreType,
    pub http_client: reqwest::Client,
    //pub email_client: EmailClientType,
    pub email_server: Option<wiremock::MockServer>,
    pub db_url: Option<Secret>,
}

impl TestApp {
    pub async fn new(test_db_name: &str) -> Self {
        let config = Config::init_from_env();
        let config_type = ConfigType::new(config);
        let config = config_type.inner();
        let mut real_db_url = None;
        let user_store_type = match config.db.db_engine {
            DatabaseEngine::Memory => UserStoreType::new(HashmapUserStore::default()),
            DatabaseEngine::Postgres => {
                real_db_url = Some(config.db.db_url(None));
                let test_db_pool = configure_database_for_testing(
                    real_db_url.as_ref().unwrap(),
                    test_db_name,
                    &config.db.db_url(Some(test_db_name)),
                )
                    .await;
                UserStoreType::new(PostgresUserStore::new(test_db_pool))
            }
        };
        let banned_token_store_type = match config.cache.cache_engine {
            CacheEngine::Memory => BannedTokenStoreType::new(HashsetBannedTokenStore::default()),
            CacheEngine::Redis => {
                let connection = configure_cache(&config.cache.cache_url()).expect("Failed to configure cache");
                let connection = RwLock::new(connection);
                BannedTokenStoreType::new(RedisBannedTokenStore::new(connection, u64::from(config.jwt.jwt_ttl)))
            }
        };
        let two_factor_auth_code_store_type = match config.cache.cache_engine {
            CacheEngine::Memory => TwoFactorAuthCodeStoreType::new(HashmapTwoFactorAuthCodeStore::default()),
            CacheEngine::Redis => {
                let connection = configure_cache(&config.cache.cache_url()).expect("Failed to configure cache");
                let connection = RwLock::new(connection);
                TwoFactorAuthCodeStoreType::new(RedisTwoFactorAuthCodeStore::new(
                    connection,
                    u64::from(config.tfa.tfa_ttl),
                ))
            }
        };
        let mut email_server = None;
        let email_client_type = match config.email.email_service {
            EmailService::Mock => EmailClientType::new(MockEmailClient),
            EmailService::Postmark => {
                email_server = Some(wiremock::MockServer::start().await);
                let api_key = Secret::new("fake-auth-token");
                let api_url = url::Url::parse(&email_server.as_ref().unwrap().uri()).unwrap();
                let api_timeout = config.email.email_api_timeout;
                let client = configure_email_client_for_testing(
                    api_key,
                    api_url,
                    api_timeout,
                    config.email.email_stream.clone(),
                    config.email.email_sender.clone().unwrap());
                EmailClientType::new(client)
            }
        };
        let app_state = AppState::new(
            user_store_type.clone(),
            banned_token_store_type.clone(),
            two_factor_auth_code_store_type.clone(),
            email_client_type.clone(),
            config_type.clone(),
        );
        let socket_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
        let application = Application::build(app_state, socket_addr)
            .await
            .expect("Failed to build app");
        let socket_addr = application.address;
        let uri = Uri::builder()
            .scheme("http")
            .authority(socket_addr.to_string().as_str())
            .path_and_query("/")
            .build()
            .expect("Failed to build URI");
        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _task = tokio::spawn(application.run());
        let cookie_jar = Arc::new(Jar::default());
        let http_client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("Failed to build HTTP client");
        Self {
            base_url: uri.to_string(),
            cookie_jar,
            banned_token_store: banned_token_store_type,
            two_factor_auth_code_store: two_factor_auth_code_store_type,
            http_client,
            //email_client: email_client_type,
            email_server,
            db_url: real_db_url,
        }
    }

    pub async fn get_root(&self) -> Response {
        let request_url = self.base_url.clone();
        self.http_client
            .get(&request_url)
            .send()
            .await
            .expect("Failed to execute get_root request")
    }

    pub async fn post_signup<S: serde::Serialize>(&self, body: &S) -> Response {
        let request_url = format!("{}api/signup", &self.base_url);
        self.http_client
            .post(&request_url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_signup request")
    }

    pub async fn post_login<S: serde::Serialize>(&self, body: &S) -> Response {
        let request_url = format!("{}api/login", &self.base_url);
        self.http_client
            .post(&request_url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_login request")
    }

    pub async fn post_logout(&self) -> Response {
        let request_url = format!("{}api/logout", &self.base_url);
        self.http_client
            .post(&request_url)
            .send()
            .await
            .expect("Failed to execute post_logout request")
    }

    pub async fn post_verify_2fa<S: serde::Serialize>(&self, body: &S) -> Response {
        let request_url = format!("{}api/verify-2fa", &self.base_url);
        self.http_client
            .post(&request_url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_verify_2fa request")
    }

    pub async fn post_verify_token<S: serde::Serialize>(&self, body: &S) -> Response {
        let request_url = format!("{}api/verify-token", &self.base_url);
        self.http_client
            .post(&request_url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_verify_token request")
    }
}

pub struct TestAppAsyncContext {
    pub db_name: String,
    pub db_url: Option<Secret>,
}

impl AsyncTestContext for TestAppAsyncContext {
    async fn setup() -> Self {
        Self {
            db_name: Uuid::now_v7().to_string(),
            db_url: None,
        }
    }

    async fn teardown(self) {
        if let Some(db_url) = self.db_url {
            delete_database(&db_url, &self.db_name).await;
        }
    }
}

async fn configure_database_for_testing(real_db_url: &Secret, test_db_name: &str, test_db_url: &Secret) -> PgPool {
    let pool = PgPoolOptions::new()
        .connect(real_db_url.expose())
        .await
        .expect("Failed to connect to test database");
    pool.execute(format!(r#"CREATE DATABASE "{test_db_name}";"#).as_str())
        .await
        .expect("Failed to create test database");
    let pool = PgPoolOptions::new()
        .connect(test_db_url.expose())
        .await
        .expect("Failed to create to test database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate the test database");
    pool
}

async fn delete_database(real_db_url: &Secret, test_db_name: &str) {
    let options =
        PgConnectOptions::from_str(real_db_url.expose()).expect("Failed to parse the database connection string");
    let mut connection = PgConnection::connect_with(&options)
        .await
        .expect("Failed to connect to the app database");
    connection
        .execute(
            format!(
                r"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                  FROM pg_stat_activity
                 WHERE pg_stat_activity.datname = '{test_db_name}'
                   AND pid <> pg_backend_pid();
                "
            )
                .as_str(),
        )
        .await
        .expect("Failed to kill all connections to the test database");
    connection
        .execute(format!(r#"DROP DATABASE "{test_db_name}";"#).as_str())
        .await
        .expect("Failed to drop the test database");
}

fn configure_email_client_for_testing(
    api_key: Secret,
    api_url: url::Url,
    api_timeout: u32,
    stream: String,
    sender: Email,
) -> PostmarkEmailClient {
    let client = Client::builder()
        .timeout(std::time::Duration::from_millis(u64::from(api_timeout)))
        .build()
        .expect("Failed to build HTTP client");
    PostmarkEmailClient::new(client, api_key, api_url, stream, sender)
}
