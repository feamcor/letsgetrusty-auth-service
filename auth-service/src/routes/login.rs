use crate::app_state::AppState;
use crate::domain::User;
use crate::services::UserStore;
use crate::utils::api_error::{ApiError, TwoFactorAuthError};
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

#[instrument(level = Level::TRACE)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), ApiError> {
    let store = &state.user_store.read().await;
    User::try_new(&request.email, &request.password, false)?;
    store
        .validate_user(&request.email, &request.password)
        .await?;
    let user = store.get_user(&request.email).await?;

    if user.requires_2fa {
        // TODO: check against the 2FA provided previously
        return Err(TwoFactorAuthError::default().into());
    }

    let cookie = generate_auth_cookie(&user.email)?;
    let jar = jar.add(cookie);
    Ok((jar, StatusCode::OK.into_response()))
}
