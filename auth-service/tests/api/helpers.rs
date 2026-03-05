use auth_service::app_state::AppState;
use auth_service::config::{Config, ConfigType, StoreEngine};
use auth_service::services::{
    BannedTokenStoreType, EmailClientType, HashmapTwoFactorAuthCodeStore, HashmapUserStore,
    HashsetBannedTokenStore, MockEmailClient, PostgresUserStore, TwoFactorAuthCodeStoreType,
    UserStoreType,
};
use auth_service::Application;
use axum::http::Uri;
use reqwest::cookie::Jar;
use reqwest::{Client, Response};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use uuid::Uuid;

pub struct TestApp {
    pub base_url: String,
    pub http_client: Client,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
    pub two_factor_auth_code_store: TwoFactorAuthCodeStoreType,
}

impl TestApp {
    pub async fn new() -> Self {
        let config_type = ConfigType::new(Config::init_from_env());
        let config = config_type.inner();

        let user_store_type = match config.store_engine {
            StoreEngine::Memory => {
                UserStoreType::new(HashmapUserStore::default())
            }
            StoreEngine::Database => {
                let pool = configure_database_for_testing(&config).await;
                UserStoreType::new(PostgresUserStore::new(pool))
            }
        };

        let banned_token_store_type = BannedTokenStoreType::new(HashsetBannedTokenStore::default());
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
        }
    }

    pub async fn get_root(&self) -> Response {
        let request_url = self.base_url.to_string();
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

async fn configure_database_for_testing(config: &Config) -> PgPool {
    let db_name = Uuid::now_v7().to_string();
    let db_url = config.database_url(None);
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to create connection pool");
    pool.execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database");
    let db_url = config.database_url(Some(&db_name));
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to create connection pool");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate the database");
    pool
}
