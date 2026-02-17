use crate::domain::UserError;
use crate::services::UserStoreError;
use crate::utils::auth::GenerateTokenError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    UserError(#[from] UserError),
    #[error(transparent)]
    UserStoreError(#[from] UserStoreError),
    #[error(transparent)]
    GenerateTokenError(#[from] GenerateTokenError),
    #[error(transparent)]
    TwoFactorAuthError(#[from] TwoFactorAuthError),
    #[error("Invalid token")]
    TokenInvalid,
    #[error("Token missing")]
    TokenMissing,
    #[error("Token banned")]
    TokenBanned,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::UserError(err) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::from(err.to_string())),
            ),
            ApiError::UserStoreError(UserStoreError::UserNotFound(err)) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::from(err.to_string())),
            ),
            ApiError::UserStoreError(UserStoreError::IncorrectCredentials(err)) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::from(err.to_string())),
            ),
            ApiError::UserStoreError(UserStoreError::UserAlreadyExists(err)) => (
                StatusCode::CONFLICT,
                Json(ErrorResponse::from(err.to_string())),
            ),
            ApiError::UserStoreError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(err.to_string())),
            ),
            ApiError::GenerateTokenError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(err.to_string())),
            ),
            ApiError::TwoFactorAuthError(err) => (
                StatusCode::PARTIAL_CONTENT,
                Json(ErrorResponse::TwoFactorAuth(err)),
            ),
            err @ ApiError::TokenInvalid => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::from(err.to_string())),
            ),
            err @ ApiError::TokenMissing => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::from(err.to_string())),
            ),
            err @ ApiError::TokenBanned => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::from(err.to_string())),
            ),
            ApiError::UnexpectedError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from("Unexpected error".to_string())),
            ),
        };
        (status, body).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorResponse {
    Error(String),
    TwoFactorAuth(TwoFactorAuthError),
}

impl From<String> for ErrorResponse {
    fn from(s: String) -> Self {
        ErrorResponse::Error(s)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TwoFactorAuthError {
    message: String,
    login_attempt_id: String,
}

impl std::error::Error for TwoFactorAuthError {}

impl std::fmt::Display for TwoFactorAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Default for TwoFactorAuthError {
    fn default() -> Self {
        let message = "2FA required".to_string();
        let login_attempt_id = Uuid::now_v7().to_string();
        Self {
            message,
            login_attempt_id,
        }
    }
}
