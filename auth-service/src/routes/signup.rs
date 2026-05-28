use crate::app_state::AppState;
use crate::domain::Email;
use crate::domain::HashedPassword;
use crate::domain::Secret;
use crate::domain::User;
use crate::utils::api_error::ApiResult;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SignupRequest {
    pub email: Secret,
    pub password: Secret,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SignupResponse {
    pub message: String,
}

#[tracing::instrument(name = "ApiHandlerSignup", skip_all)]
pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> ApiResult<impl IntoResponse> {
    let email = Email::parse(&request.email)?;
    let password = HashedPassword::parse(&request.password, &email).await?;
    let user = User::new(&email, &password, request.requires_2fa);
    state.user_store.inner().add_user(user).await?;
    let response = Json(SignupResponse {
        message: "User created successfully".to_string(),
    });
    Ok((StatusCode::CREATED, response))
}
