use auth_service::app_state::AppState;
use auth_service::services::{HashmapTwoFactorAuthCodeStore, HashmapUserStore, HashsetBannedTokenStore};
use auth_service::Application;
use axum::http::Uri;
use reqwest::cookie::Jar;
use reqwest::{Client, Response};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct TestApp {
    pub base_url: String,
    pub http_client: Client,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: Arc<HashsetBannedTokenStore>,
    pub two_fa_code_store: Arc<HashmapTwoFactorAuthCodeStore>,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(HashmapUserStore::default());
        let banned_token_store = Arc::new(HashsetBannedTokenStore::default());
        let two_fa_code_store = Arc::new(HashmapTwoFactorAuthCodeStore::default());
        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
        );
        let socket_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let application = Application::build(app_state, socket_addr, 8000)
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
            banned_token_store,
            two_fa_code_store,
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
