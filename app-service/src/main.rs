use askama::Template;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use axum_extra::extract::CookieJar;
use tower_http::services::ServeDir;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/protected", get(protected));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_link: String,
    logout_link: String,
}

//noinspection HttpUrlsUsage
async fn root() -> impl IntoResponse {
    let mut address = std::env::var("AUTH_SERVICE_IP").unwrap_or("localhost".to_owned());
    if address.is_empty() {
        "localhost".clone_into(&mut address);
    }
    let login_link = format!("http://{address}:3000");
    let logout_link = format!("http://{address}:3000/logout");
    let template = IndexTemplate {
        login_link,
        logout_link,
    };
    Html(template.render().unwrap())
}

//noinspection HttpUrlsUsage
async fn protected(jar: CookieJar) -> impl IntoResponse {
    let Some(jwt_cookie) = jar.get("jwt") else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };
    let api_client = reqwest::Client::builder().build().unwrap();
    let verify_token_body = serde_json::json!({
        "token": &jwt_cookie.value(),
    });
    let auth_hostname = std::env::var("AUTH_SERVICE_HOST_NAME").unwrap_or("0.0.0.0".to_owned());
    let url = format!("http://{auth_hostname}:3000/api/verify-token");
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
