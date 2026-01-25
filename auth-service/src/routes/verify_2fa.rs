use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

#[instrument(level = Level::TRACE)]
pub async fn verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response() // TODO: dummy response for task 4
}
