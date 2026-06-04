use askama::Template;
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum_extra::extract::CookieJar;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const DEFAULT_APP_SERVICE_PORT: u16 = 8000;
const DEFAULT_AUTH_SERVICE_PORT: u16 = 3000;
const DEFAULT_AUTH_SERVICE_BROWSER_HOST: &str = "localhost";
const DEFAULT_AUTH_SERVICE_INTERNAL_HOST: &str = "0.0.0.0";

#[derive(Clone)]
struct AppConfig {
    /// Host the browser uses to reach auth-service (rendered in login/logout links).
    auth_browser_host: String,
    /// Port the browser uses to reach auth-service.
    auth_browser_port: u16,
    /// Host this process uses for server-to-server calls to auth-service.
    auth_internal_host: String,
    /// Port for server-to-server calls (typically the same as the browser port).
    auth_internal_port: u16,
}

impl AppConfig {
    fn from_env() -> Self {
        let auth_browser_host = std::env::var("AUTH_SERVICE_IP")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_AUTH_SERVICE_BROWSER_HOST.to_owned());
        let auth_internal_host = std::env::var("AUTH_SERVICE_HOST_NAME")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_AUTH_SERVICE_INTERNAL_HOST.to_owned());
        let auth_browser_port = parse_port_env("AUTH_SERVICE_PORT", DEFAULT_AUTH_SERVICE_PORT);
        Self {
            auth_browser_host,
            auth_browser_port,
            auth_internal_host,
            auth_internal_port: auth_browser_port,
        }
    }
}

fn parse_port_env(key: &str, fallback: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|&p| p >= 1024)
        .unwrap_or(fallback)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();
    let config = Arc::new(AppConfig::from_env());
    let bind_port = parse_port_env("APP_SERVICE_PORT", DEFAULT_APP_SERVICE_PORT);
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/protected", get(protected))
        .with_state(config);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", bind_port))
        .await
        .expect("failed to bind app-service listener");
    tracing::info!(
        "listening on {}",
        listener.local_addr().expect("listener has a local address")
    );
    axum::serve(listener, app).await.expect("app-service server error");
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_link: String,
    logout_link: String,
}

//noinspection HttpUrlsUsage
async fn root(State(config): State<Arc<AppConfig>>) -> impl IntoResponse {
    let base = format!("http://{}:{}", config.auth_browser_host, config.auth_browser_port);
    let template = IndexTemplate {
        login_link: base.clone(),
        logout_link: format!("{base}/logout"),
    };
    Html(template.render().expect("index template renders"))
}

//noinspection HttpUrlsUsage
async fn protected(State(config): State<Arc<AppConfig>>, jar: CookieJar) -> impl IntoResponse {
    let Some(jwt_cookie) = jar.get("jwt") else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };
    let api_client = reqwest::Client::builder()
        .build()
        .expect("failed to build reqwest client");
    let verify_token_body = serde_json::json!({
        "token": &jwt_cookie.value(),
    });
    let url = format!(
        "http://{}:{}/api/verify-token",
        config.auth_internal_host, config.auth_internal_port
    );
    let Ok(response) = api_client.post(&url).json(&verify_token_body).send().await else {
        return reqwest::StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match response.status() {
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::BAD_REQUEST => {
            axum::http::StatusCode::UNAUTHORIZED.into_response()
        }
        reqwest::StatusCode::OK => Json(ProtectedRouteResponse {
            img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png".to_owned(),
        })
        .into_response(),
        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(serde::Serialize)]
pub struct ProtectedRouteResponse {
    pub img_url: String,
}
