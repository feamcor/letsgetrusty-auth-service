use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

#[instrument(level = Level::TRACE)]
pub async fn health() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
