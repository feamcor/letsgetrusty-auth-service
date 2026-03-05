use crate::app_state::AppState;
use crate::utils::api_error::ApiError;
use crate::utils::auth::validate_token;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[instrument(level = Level::TRACE)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let config = &state.config;
    validate_token(&request.token, config.inner().jwt_secret.as_ref().unwrap())
        .await
        .map_err(|_| ApiError::TokenInvalid)?;
    let is_banned = state
        .banned_token_store
        .inner()
        .is_token_banned(&request.token)
        .await
        .map_err(|e| ApiError::UnexpectedError(e.into()))?;
    if is_banned {
        return Err(ApiError::TokenBanned);
    }
    Ok(StatusCode::OK)
}
