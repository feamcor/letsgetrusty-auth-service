use crate::app_state::AppState;
use crate::utils::api_error::ApiError;
use crate::utils::auth::validate_token;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::services::BannedTokenStore;
#[allow(unused_imports)]
use tracing::Level;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum VerifyTokenResponse {
    Error(String),
}

#[instrument(level = Level::TRACE)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_token(&request.token)
        .await
        .map_err(|_| ApiError::TokenInvalid)?;
    let store = state.banned_token_store.read().await;
    let is_banned = store
        .is_token_banned(&request.token)
        .await
        .map_err(|e| ApiError::UnexpectedError(e.into()))?;
    if is_banned {
        return Err(ApiError::TokenBanned);
    }
    Ok(StatusCode::OK)
}
