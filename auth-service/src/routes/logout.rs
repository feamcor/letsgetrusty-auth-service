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
    let jwt_secret = state.jwt_secret()?;
    validate_token(&token, &jwt_secret).map_err(|_| ApiError::TokenInvalid)?;
    // Clear the cookie unconditionally — it's a purely client-side state change and we want the
    // browser to drop the JWT even if the server-side ban write below fails. Returning Err here
    // would discard the jar (axum only sends Set-Cookie on the Ok branch).
    let removal_cookie = create_auth_cookie(&Token::new(&String::new().into()));
    let jar = jar.remove(removal_cookie);
    // Best-effort ban: log transient store errors at error level (so operators see them) but
    // still report success to the user — the cookie clear above already ends the session as far
    // as the browser is concerned, and double-logout is idempotent.
    match state.banned_token_store.inner().add_token(&token).await {
        Ok(()) | Err(BannedTokenStoreError::TokenAlreadyExists) => {}
        Err(error) => {
            tracing::error!(
                "banned-token store failed during logout (cookie still cleared): {}",
                error
            );
        }
    }
    Ok((jar, StatusCode::OK.into_response()))
}
