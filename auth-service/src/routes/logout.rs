use crate::utils::auth::{create_auth_cookie, validate_token};
use crate::utils::constants::JWT_COOKIE_NAME;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum LogoutResponse {
    Error(String),
}

#[instrument(level = Level::TRACE)]
pub async fn logout(jar: CookieJar) -> (CookieJar, impl IntoResponse) {
    let Some(cookie) = jar.get(JWT_COOKIE_NAME) else {
        return (
            jar,
            (
                StatusCode::BAD_REQUEST,
                Json(LogoutResponse::Error("JWT cookie not found".to_string())),
            )
                .into_response(),
        );
    };
    let token = cookie.value().to_owned();
    match validate_token(&token).await {
        Ok(_claims) => {
            let jar = jar.remove(create_auth_cookie("".to_string()));
            (jar, StatusCode::OK.into_response())
        }
        Err(error) => (
            jar,
            (
                StatusCode::UNAUTHORIZED,
                Json(LogoutResponse::Error(error.to_string())),
            )
                .into_response(),
        ),
    }
}
