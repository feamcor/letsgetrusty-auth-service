use crate::app_state::AppState;
use crate::domain::{LoginAttemptId, TwoFactorAuthCode, User};
use crate::services::{EmailClient, TwoFactorAuthCodeStore, UserStore};
use crate::utils::api_error::ApiError;
use crate::utils::auth::generate_auth_cookie;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::instrument;

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
    pub login_attempt_id: LoginAttemptId,
}

impl Default for TwoFactorAuthResponse {
    fn default() -> Self {
        let message = "2FA required".to_string();
        let login_attempt_id = LoginAttemptId::default();
        Self {
            message,
            login_attempt_id,
        }
    }
}

#[instrument(level = Level::TRACE)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), ApiError> {
    let user_store = &state.user_store;
    User::try_new(&request.email, &request.password, false)?;
    user_store
        .validate_user(&request.email, &request.password)
        .await?;
    let user = user_store.get_user(&request.email).await?;

    if user.requires_2fa {
        let response = TwoFactorAuthResponse::default();
        let login_attempt_id = response.login_attempt_id.clone();
        let auth_code = TwoFactorAuthCode::default();
        state
            .email_client
            .send_email(
                &user.email,
                "Auth Service Login Attempt",
                &format!("2FA Code: {}", auth_code),
            )
            .await?;
        let auth_code_store = &state.two_factor_auth_code_store;
        auth_code_store
            .add_code(user.email, login_attempt_id, auth_code)
            .await?;
        return Ok((
            jar,
            (StatusCode::PARTIAL_CONTENT, Json(response)).into_response(),
        ));
    }

    let config = &state.config;
    let cookie = generate_auth_cookie(
        &user.email,
        config.jwt_secret.as_ref().unwrap(),
        config.jwt_ttl_seconds,
    )?;
    let jar = jar.add(cookie);
    Ok((jar, StatusCode::OK.into_response()))
}
