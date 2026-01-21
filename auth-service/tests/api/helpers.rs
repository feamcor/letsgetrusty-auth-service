use auth_service::Application;
use axum::http::Uri;
use reqwest::header::SET_COOKIE;
use reqwest::{Client, Response};
use serde::Serialize;
use std::net::SocketAddr;

pub struct TestApp {
    pub base_url: String,
    pub http_client: Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let socket_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let application = Application::build(socket_addr)
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
        let http_client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build HTTP client");
        Self { base_url: uri.to_string(), http_client }
    }

    pub async fn get_root(&self) -> Response {
        let request_url = format!("{}", &self.base_url);
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

pub fn jwt_cookie(response: &Response) -> Option<String> {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().to_string())
        .find(|cookie| cookie.starts_with("jwt="))
}

pub fn assert_jwt(jwt: Option<String>) -> String {
    let Some(jwt) = jwt else { panic!("JWT cookie is missing"); };
    assert!(jwt.contains("HttpOnly;"), "JWT must be HttpOnly");
    assert!(jwt.contains("Secure;"), "JWT must be Secure");
    assert!(jwt.contains("SameSite=Lax;"), "JWT must have SameSite=Lax set");
    assert!(jwt.ends_with("Path=/"), "JWT must have Path=/ set");
    jwt
}
