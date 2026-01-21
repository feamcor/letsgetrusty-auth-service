use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::instrument;

#[instrument]
pub async fn verify_token() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}