use crate::utils::api_error::ApiError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

#[instrument(level = Level::TRACE)]
pub async fn health() -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::OK.into_response())
}
