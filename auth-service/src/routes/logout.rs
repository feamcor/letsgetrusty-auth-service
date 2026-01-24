use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::instrument;
use tracing::Level;

#[instrument(level = Level::TRACE)]
pub async fn logout() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}
