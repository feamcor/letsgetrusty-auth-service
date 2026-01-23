use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::instrument;

#[instrument(level = Level::TRACE)]
pub async fn verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}
