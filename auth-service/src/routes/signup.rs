use crate::app_state::AppState;
use crate::domain::User;
use crate::utils::api_error::ApiError;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SignupResponse {
    pub message: String,
}

#[tracing::instrument(name = "ApiHandlerSignup", skip_all, err(Debug))]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = User::try_new(
        request.email.as_str(),
        request.password.as_str(),
        request.requires_2fa,
    )
    .await?;
    state.user_store.inner().add_user(user).await?;
    let response = Json(SignupResponse {
        message: "User created successfully".to_string(),
    });
    Ok((StatusCode::CREATED, response))
}
