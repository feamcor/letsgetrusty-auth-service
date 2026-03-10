use crate::domain::EmailError;
use crate::domain::LoginAttemptIdError;
use crate::domain::TwoFactorAuthCodeError;
use crate::domain::UserError;
use crate::services::EmailClientError;
use crate::services::TwoFactorAuthCodeStoreError;
use crate::services::UserStoreError;
use crate::utils::auth::GenerateTokenError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;

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
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        log_error_chain(&self);
        let (status, body) = match self {
            ApiError::UserError(error) => (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(error.to_string()))),
            ApiError::UserStoreError(
                error @ (UserStoreError::UserNotFound(_) | UserStoreError::IncorrectCredentials(_)),
            ) => (StatusCode::UNAUTHORIZED, Json(ErrorResponse::from(error.to_string()))),
            ApiError::UserStoreError(error @ UserStoreError::UserAlreadyExists(_)) => {
                (StatusCode::CONFLICT, Json(ErrorResponse::from(error.to_string())))
            }
            ApiError::UserStoreError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::GenerateTokenError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::TwoFactorAuthCodeError(error) => {
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(error.to_string())))
            }
            ApiError::TwoFactorAuthCodeStoreError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            ApiError::EmailError(error) => (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(error.to_string()))),
            ApiError::LoginAttemptIdError(error) => {
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(error.to_string())))
            }
            ApiError::EmailClientError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
            error @ (ApiError::TokenInvalid | ApiError::TokenBanned | ApiError::IncorrectCredentials) => {
                (StatusCode::UNAUTHORIZED, Json(ErrorResponse::from(error.to_string())))
            }
            error @ ApiError::TokenMissing => (StatusCode::BAD_REQUEST, Json(ErrorResponse::from(error.to_string()))),
            ApiError::UnexpectedError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::from(error.to_string())),
            ),
        };
        (status, body).into_response()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ErrorResponse {
    Error(String),
}

impl From<String> for ErrorResponse {
    fn from(s: String) -> Self {
        ErrorResponse::Error(s)
    }
}


fn log_error_chain(error: &(dyn std::error::Error + 'static)) {
    use std::fmt::Write;
    let mut buffer = format!("{error:?}");
    let mut current_error = error.source();
    while let Some(error_cause) = current_error {
        let _ = write!(buffer, " <== {error_cause:?}");
        current_error = error_cause.source();
    }
    tracing::debug!("{}", buffer);
}