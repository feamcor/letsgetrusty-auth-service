use crate::app_state::AppState;
use crate::domain::User;
use crate::services::UserStore;
use crate::utils::api_error::ApiError;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

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
pub enum SignupResponse {
    Message(String),
}

#[instrument(level = Level::TRACE)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = User::try_new(
        request.email.as_str(),
        request.password.as_str(),
        request.requires_2fa,
    )?;
    let store = &mut state.user_store.write().await;
    store.add_user(user).await?;
    let response = Json(SignupResponse::Message(
        "User created successfully".to_string(),
    ));
    Ok((StatusCode::CREATED, response))
}
