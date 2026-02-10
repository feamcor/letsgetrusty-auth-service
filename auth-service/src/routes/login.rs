use crate::app_state::AppState;
use crate::utils::auth::{GenerateTokenError, generate_auth_cookie};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::domain::{User, UserError};
use crate::services::UserStore;

#[allow(unused_imports)]
use tracing::Level;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum LoginResponse {
    Error(String),
    #[serde(untagged)]
    PartialSuccess {
        message: String,
        login_attempt_id: String,
    },
}

#[instrument(level = Level::TRACE)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, impl IntoResponse) {
    let store = &state.user_store.read().await;
    match User::try_new(&request.email, &request.password, false) {
        Ok(_) => {
            match store.validate_user(&request.email, &request.password).await {
                Ok(_) => {
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
                    // TODO: 2FA should be verified here
                    // So far, failing if requires 2FA is true
                    if user.requires_2fa {
                        return (
                            jar,
                            (
                                StatusCode::PARTIAL_CONTENT,
                                Json(LoginResponse::PartialSuccess {
                                    message: "Login requires 2FA".to_string(),
                                    login_attempt_id: Uuid::now_v7().to_string(),
                                }),
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
                    let updated_jar = jar.add(cookie);
                    (updated_jar, StatusCode::OK.into_response())
                }
                Err(error) => (
                    jar,
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(LoginResponse::Error(error.to_string())),
                    )
                        .into_response(),
                ),
            }
        }
        Err(UserError::InvalidEmail(error)) => (
            jar,
            (
                StatusCode::BAD_REQUEST,
                Json(LoginResponse::Error(error.to_string())),
            )
                .into_response(),
        ),
        Err(UserError::InvalidPassword(error)) => (
            jar,
            (
                StatusCode::BAD_REQUEST,
                Json(LoginResponse::Error(error.to_string())),
            )
                .into_response(),
        ),
    }
}
