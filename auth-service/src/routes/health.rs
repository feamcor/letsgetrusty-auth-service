use crate::utils::api_error::ApiResult;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[tracing::instrument(name = "ApiHandlerHealth", skip_all)]
pub async fn health() -> ApiResult<impl IntoResponse> {
    Ok(StatusCode::OK.into_response())
}
