use crate::app_state::AppState;
use crate::services::BannedTokenStore;
use crate::utils::api_error::ApiError;
use crate::utils::auth::{create_auth_cookie, validate_token};
use crate::utils::auth::JWT_COOKIE_NAME;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

#[instrument(level = Level::TRACE)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), ApiError> {
    let Some(cookie) = jar.get(JWT_COOKIE_NAME) else {
        return Err(ApiError::TokenMissing);
    };
    let token = cookie.value().to_owned();
    let config = &state.config;
    validate_token(&token, &config.jwt_secret.as_ref().unwrap())
        .await
        .map_err(|_| ApiError::TokenInvalid)?;
    let jar = jar.remove(create_auth_cookie("".to_string()));
    let store = &state.banned_token_store;
    store
        .add_token(&token)
        .await
        .map_err(|e| ApiError::UnexpectedError(e.into()))?;
    Ok((jar, StatusCode::OK.into_response()))
}
