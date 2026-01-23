use crate::app_state::AppState;
use crate::domain::{User, UserError};
use crate::services::UserStoreError;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SignupResponse {
    Message(String),
    Error(String),
}

#[instrument(level = Level::TRACE)]
pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> impl IntoResponse {
    match User::try_new(
        request.email.as_str(),
        request.password.as_str(),
        request.requires_2fa)
    {
        Ok(user) => {
            let store = &mut state.user_store.write().await;
            match store.add_user(user) {
                Ok(()) => {
                    let response = Json(SignupResponse::Message("User created successfully!".to_string()));
                    (StatusCode::CREATED, response)
                }
                Err(UserStoreError::UserAlreadyExists(_)) => {
                    let response = Json(SignupResponse::Error("User already exists".to_string()));
                    (StatusCode::CONFLICT, response)
                }
                Err(UserStoreError::UnexpectedError) => {
                    let response = Json(SignupResponse::Error("Unexpected error".to_string()));
                    (StatusCode::INTERNAL_SERVER_ERROR, response)
                }
                _ => {
                    unreachable!()
                }
            }
        }
        Err(UserError::InvalidEmailAddress(error)) => {
            let response = Json(SignupResponse::Error(format!("Invalid email: {}", error)));
            (StatusCode::BAD_REQUEST, response)
        }
        Err(UserError::InvalidPassword(error)) => {
            let response = Json(SignupResponse::Error(format!("Invalid password: {}", error)));
            (StatusCode::BAD_REQUEST, response)
        }
        Err(UserError::UnexpectedError) => {
            let response = Json(SignupResponse::Error("Unexpected error".to_string()));
            (StatusCode::INTERNAL_SERVER_ERROR, response)
        }
    }
}
