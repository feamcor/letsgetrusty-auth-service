use crate::app_state::AppState;
use crate::domain::User;
use crate::services::UserStore;
use crate::utils::auth::{generate_auth_cookie, GenerateTokenError};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use uuid::Uuid;

#[allow(unused_imports)]
use tracing::Level;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TwoFactorAuthResponse {
    pub message: String,
    login_attempt_id: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum LoginResponse {
    Error(String),
    #[serde(untagged)]
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[instrument(level = Level::TRACE)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, impl IntoResponse) {
    let store = &state.user_store.read().await;
    if let Err(error) = User::try_new(&request.email, &request.password, false) {
        return (
            jar,
            (
                StatusCode::BAD_REQUEST,
                Json(LoginResponse::Error(error.to_string())),
            )
                .into_response(),
        )
    }

    if let Err(error) = store.validate_user(&request.email, &request.password).await {
        return (
            jar,
            (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse::Error(error.to_string())),
            )
                .into_response(),
        )
    }

    let user = match store.get_user(&request.email).await {
        Ok(user) => user,
        Err(error) => {
            error!("Unexpected error when getting user from store: {}", error);
            return (
                jar,
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LoginResponse::Error(error.to_string())),
                )
                    .into_response(),
            );
        }
    };

    if user.requires_2fa {
        // TODO: check against the 2FA provided previously
        let two_factor_auth_response = TwoFactorAuthResponse {
            message: "2FA required".to_string(),
            login_attempt_id: Uuid::now_v7().to_string(),
        };
        return (
            jar,
            (
                StatusCode::PARTIAL_CONTENT,
                Json(LoginResponse::TwoFactorAuth(two_factor_auth_response)),
            )
                .into_response(),
        );
    }

    let cookie = match generate_auth_cookie(&user.email) {
        Ok(cookie) => cookie,
        Err(GenerateTokenError::TokenError(error)) => {
            return (
                jar,
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LoginResponse::Error(error.to_string())),
                )
                    .into_response(),
            );
        }
        Err(GenerateTokenError::UnexpectedError) => {
            return (
                jar,
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LoginResponse::Error("Unexpected error".to_string())),
                )
                    .into_response(),
            );
        }
    };

    let jar = jar.add(cookie);
    (jar, StatusCode::OK.into_response())
}
