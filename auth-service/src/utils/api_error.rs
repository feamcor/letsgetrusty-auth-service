use crate::domain::{EmailError, LoginAttemptIdError, TwoFactorAuthCodeError, UserError};
use crate::services::{EmailClientError, TwoFactorAuthCodeStoreError, UserStoreError};
use crate::utils::auth::GenerateTokenError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    UserError(#[from] UserError),
    #[error(transparent)]
    UserStoreError(#[from] UserStoreError),
    #[error(transparent)]
    GenerateTokenError(#[from] GenerateTokenError),
    #[error(transparent)]
    TwoFactorAuthCodeError(#[from] TwoFactorAuthCodeError),
    #[error(transparent)]
    TwoFactorAuthCodeStoreError(#[from] TwoFactorAuthCodeStoreError),
    #[error(transparent)]
    EmailError(#[from] EmailError),
    #[error(transparent)]
    LoginAttemptIdError(#[from] LoginAttemptIdError),
    #[error(transparent)]
    EmailClientError(#[from] EmailClientError),
    #[error("Invalid token")]
    TokenInvalid,
    #[error("Token missing")]
    TokenMissing,
    #[error("Token banned")]
    TokenBanned,
    #[error("Incorrect Credentials")]
    IncorrectCredentials,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::UserError(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::UserStoreError(
                error @ (UserStoreError::UserNotFound(_) | UserStoreError::IncorrectCredentials(_)),
            ) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::UserStoreError(error @ UserStoreError::UserAlreadyExists(_)) => (
                StatusCode::CONFLICT,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::UserStoreError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::GenerateTokenError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::TwoFactorAuthCodeError(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::TwoFactorAuthCodeStoreError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::EmailError(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::LoginAttemptIdError(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::EmailClientError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            error @ (ApiError::TokenInvalid
            | ApiError::TokenBanned
            | ApiError::IncorrectCredentials) => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::from(error.to_string())),
            ),
            error @ ApiError::TokenMissing => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::UnexpectedError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
        };
        (status, body).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorResponse {
    Error(String),
}

impl From<String> for ErrorResponse {
    fn from(s: String) -> Self {
        ErrorResponse::Error(s)
    }
}
