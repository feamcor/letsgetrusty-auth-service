use crate::app_state::AppState;
use crate::domain::Email;
use crate::domain::HashedPassword;
use crate::domain::LoginAttemptId;
use crate::domain::Secret;
use crate::domain::TwoFactorAuthCode;
use crate::utils::api_error::ApiError;
use crate::utils::api_error::ApiResult;
use crate::utils::auth::generate_auth_cookie;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: Secret,
    pub password: Secret,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

#[tracing::instrument(name = "ApiHandlerLogin", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> ApiResult<(CookieJar, impl IntoResponse)> {
    let email = Email::parse(&request.email)?;
    let _ = HashedPassword::parse(&request.password, &email).await?;
    let user_store = state.user_store.inner();
    user_store.validate_user(&email, &request.password).await?;
    let user = user_store.get_user(&email).await?;
    if user.requires_2fa {
        let response = TwoFactorAuthResponse::default();
        let login_attempt_id = response.login_attempt_id.clone();
        let auth_code = TwoFactorAuthCode::default();
        state
            .email_client
            .inner()
            .send_email(
                &user.email,
                "Auth Service Login Attempt",
                &format!("2FA Code: {}", auth_code.as_secret().expose()),
            )
            .await?;
        state
            .two_factor_auth_code_store
            .inner()
            .add_code(user.email, login_attempt_id, auth_code)
            .await?;
        return Ok((jar, (StatusCode::PARTIAL_CONTENT, Json(response)).into_response()));
    }

    let config = state.config.inner();
    let jwt_secret = config
        .jwt
        .jwt_secret
        .clone()
        .ok_or(ApiError::UnexpectedError(color_eyre::eyre::eyre!(
            "JWT secret is not set."
        )))?;
    let cookie = generate_auth_cookie(&user.email, &jwt_secret, i64::from(config.jwt.jwt_ttl))?;
    let jar = jar.add(cookie);
    Ok((jar, StatusCode::OK.into_response()))
}
