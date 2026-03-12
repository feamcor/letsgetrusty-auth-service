use crate::app_state::AppState;
use crate::domain::Secret;
use crate::domain::Token;
use crate::utils::api_error::ApiError;
use crate::utils::api_error::ApiResult;
use crate::utils::auth::validate_token;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VerifyTokenRequest {
    pub token: Secret,
}

#[tracing::instrument(name = "ApiHandlerVerifyToken", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> ApiResult<impl IntoResponse> {
    let config = &state.config;
    let token = Token::new(&request.token);
    let jwt_secret = config
        .inner()
        .jwt_secret
        .clone()
        .ok_or(ApiError::UnexpectedError(color_eyre::eyre::eyre!(
            "JWT secret is not set."
        )))?;
    validate_token(&token, &jwt_secret)
        .await
        .map_err(|_| ApiError::TokenInvalid)?;
    let is_banned = state
        .banned_token_store
        .inner()
        .is_token_banned(&token)
        .await
        .map_err(|e| ApiError::UnexpectedError(e.into()))?;
    if is_banned {
        return Err(ApiError::TokenBanned);
    }
    Ok(StatusCode::OK)
}
