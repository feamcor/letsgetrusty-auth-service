use auth_service::app_state::AppState;
use auth_service::config::{Config, ConfigType, StoreEngine};
use auth_service::services::{
    BannedTokenStoreType, EmailClientType, HashmapTwoFactorAuthCodeStore, HashmapUserStore,
    HashsetBannedTokenStore, MockEmailClient, PostgresUserStore, RedisBannedTokenStore,
    TwoFactorAuthCodeStoreType, UserStoreType,
};
use auth_service::{configure_cache, Application};
use axum::http::Uri;
use reqwest::cookie::Jar;
use reqwest::{Client, Response};
use serde::Serialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use test_context::AsyncTestContext;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub base_url: String,
    pub http_client: Client,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
    pub two_factor_auth_code_store: TwoFactorAuthCodeStoreType,
    pub db_url: Option<String>,
}

impl TestApp {
    pub async fn new(test_db_name: &str) -> Self {
        let config = Config::init_from_env();
        let config_type = ConfigType::new(config);
        let config = config_type.inner();
        let mut real_db_url = None;
        let user_store_type = match config.store_engine {
            StoreEngine::Ephemeral => UserStoreType::new(HashmapUserStore::default()),
            StoreEngine::Server => {
                real_db_url = Some(config.database_url(None));
                let test_db_pool = configure_database_for_testing(
                    real_db_url.as_ref().unwrap(),
                    test_db_name,
                    &config.database_url(Some(test_db_name)),
                )
                .await;
                UserStoreType::new(PostgresUserStore::new(test_db_pool))
            }
        };
        let banned_token_store_type = match config.store_engine {
            StoreEngine::Ephemeral => BannedTokenStoreType::new(HashsetBannedTokenStore::default()),
            StoreEngine::Server => {
                let connection =
                    configure_cache(&config.cache_url()).expect("Failed to configure cache");
                let connection = RwLock::new(connection);
                BannedTokenStoreType::new(RedisBannedTokenStore::new(connection))
            }
        };
        let two_factor_auth_code_store_type =
            TwoFactorAuthCodeStoreType::new(HashmapTwoFactorAuthCodeStore::default());
        let email_client_type = EmailClientType::new(MockEmailClient);
        let app_state = AppState::new(
            user_store_type,
            banned_token_store_type.clone(),
            two_factor_auth_code_store_type.clone(),
            email_client_type,
            config_type,
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
            http_client,
            cookie_jar,
            banned_token_store: banned_token_store_type,
            two_factor_auth_code_store: two_factor_auth_code_store_type,
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

    pub async fn post_signup<S: Serialize>(&self, body: &S) -> Response {
        let request_url = format!("{}api/signup", &self.base_url);
        self.http_client
            .post(&request_url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_signup request")
    }

    pub async fn post_login<S: Serialize>(&self, body: &S) -> Response {
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

    pub async fn post_verify_2fa<S: Serialize>(&self, body: &S) -> Response {
        let request_url = format!("{}api/verify-2fa", &self.base_url);
        self.http_client
            .post(&request_url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_verify_2fa request")
    }

    pub async fn post_verify_token<S: Serialize>(&self, body: &S) -> Response {
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
    pub db_url: Option<String>,
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

async fn configure_database_for_testing(
    real_db_url: &str,
    test_db_name: &str,
    test_db_url: &str,
) -> PgPool {
    let pool = PgPoolOptions::new()
        .connect(real_db_url)
        .await
        .expect("Failed to connect to test database");
    pool.execute(format!(r#"CREATE DATABASE "{test_db_name}";"#).as_str())
        .await
        .expect("Failed to create test database");
    let pool = PgPoolOptions::new()
        .connect(test_db_url)
        .await
        .expect("Failed to create to test database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate the test database");
    pool
}

async fn delete_database(real_db_url: &str, test_db_name: &str) {
    let options = PgConnectOptions::from_str(real_db_url)
        .expect("Failed to parse the database connection string");
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
