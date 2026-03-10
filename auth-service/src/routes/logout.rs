use crate::app_state::AppState;
use crate::utils::api_error::ApiError;
use crate::utils::api_error::ApiResult;
use crate::utils::auth::JWT_COOKIE_NAME;
use crate::utils::auth::create_auth_cookie;
use crate::utils::auth::validate_token;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;

#[tracing::instrument(name = "ApiHandlerLogout", skip_all)]
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> ApiResult<(CookieJar, impl IntoResponse)> {
    let Some(cookie) = jar.get(JWT_COOKIE_NAME) else {
        return Err(ApiError::TokenMissing);
    };
    let token = cookie.value().to_owned();
    let config = state.config.inner();
    validate_token(&token, config.jwt_secret.as_ref().unwrap())
        .await
        .map_err(|_| ApiError::TokenInvalid)?;
    let jar = jar.remove(create_auth_cookie(String::new()));
    let _ = state
        .banned_token_store
        .inner()
        .add_token(&token)
        .await
        .map_err(|e| ApiError::UnexpectedError(e.into()));
    Ok((jar, StatusCode::OK.into_response()))
}
