use crate::utils::auth::validate_token;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

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
pub async fn verify_token(Json(request): Json<VerifyTokenRequest>) -> impl IntoResponse {
    match validate_token(&request.token).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(error) => {
            let response = VerifyTokenResponse::Error(error.to_string());
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}
