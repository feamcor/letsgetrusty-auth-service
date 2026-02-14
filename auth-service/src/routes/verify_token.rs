use crate::app_state::AppState;
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
) -> impl IntoResponse {
    match validate_token(&request.token).await {
        Ok(_) => {
            let store = state.banned_token_store.read().await;
            let is_banned = store.is_token_banned(&request.token).await;
            match is_banned {
                Ok(false) => StatusCode::OK.into_response(),
                Ok(true) => (
                    StatusCode::UNAUTHORIZED,
                    Json(VerifyTokenResponse::Error("Token is banned".to_string())),
                )
                    .into_response(),
                Err(error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(VerifyTokenResponse::Error(error.to_string())),
                )
                    .into_response(),
            }
        }
        Err(error) => {
            let response = VerifyTokenResponse::Error(error.to_string());
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}
