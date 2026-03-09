use crate::utils::api_error::ApiError;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[tracing::instrument(name = "ApiHandlerHealth", skip_all, err(Debug))]
pub async fn health() -> Result<impl IntoResponse, ApiError> {
    Ok(StatusCode::OK.into_response())
}
