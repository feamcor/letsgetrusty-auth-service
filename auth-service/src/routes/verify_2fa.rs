use crate::app_state::AppState;
use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::Secret;
use crate::domain::TwoFactorAuthCode;
use crate::services::TwoFactorAuthCodeStoreError;
use crate::utils::api_error::ApiError;
use crate::utils::api_error::ApiResult;
use crate::utils::auth::generate_auth_cookie;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Verify2FARequest {
    pub email: Secret,
    pub login_attempt_id: Secret,
    #[serde(rename = "2FACode")]
    pub two_factor_auth_code: Secret,
}

#[tracing::instrument(name = "ApiHandlerVerify2FA", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> ApiResult<(CookieJar, impl IntoResponse)> {
    let email = Email::parse(&request.email)?;
    let attempt_id = LoginAttemptId::parse(&request.login_attempt_id)?;
    let auth_code = TwoFactorAuthCode::parse(&request.two_factor_auth_code)?;
    let auth_code_store = state.two_factor_auth_code_store.inner();
    let (stored_attempt_id, stored_auth_code) = match auth_code_store.get_code(&email).await {
        Ok(code) => code,
        Err(TwoFactorAuthCodeStoreError::CodeNotFound) => {
            return Err(ApiError::IncorrectCredentials);
        }
        Err(error) => {
            return Err(ApiError::UnexpectedError(error.into()));
        }
    };
    if attempt_id != stored_attempt_id || auth_code != stored_auth_code {
        return Err(ApiError::IncorrectCredentials);
    }
    auth_code_store.remove_code(&email).await?;
    let jwt_secret = state.jwt_secret()?;
    let jwt_ttl = i64::from(state.config.inner().jwt.jwt_ttl);
    let cookie = generate_auth_cookie(&email, &jwt_secret, jwt_ttl)?;
    let jar = jar.add(cookie);
    Ok((jar, StatusCode::OK.into_response()))
}
