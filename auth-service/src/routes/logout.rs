use crate::app_state::AppState;
use crate::domain::Token;
use crate::services::BannedTokenStoreError;
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
    let cookie = cookie.value().into();
    let token = Token::new(&cookie);
    let config = state.config.inner();
    let jwt_secret = config
        .jwt
        .jwt_secret
        .clone()
        .ok_or(ApiError::UnexpectedError(color_eyre::eyre::eyre!(
            "JWT secret is not set."
        )))?;
    validate_token(&token, &jwt_secret)
        .await
        .map_err(|_| ApiError::TokenInvalid)?;
    match state.banned_token_store.inner().add_token(&token).await {
        Ok(()) | Err(BannedTokenStoreError::TokenAlreadyExists) => {}
        Err(error) => return Err(ApiError::UnexpectedError(error.into())),
    }
    let cookie = create_auth_cookie(&Token::new(&String::new().into()));
    let jar = jar.remove(cookie);
    Ok((jar, StatusCode::OK.into_response()))
}
