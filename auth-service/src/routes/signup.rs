use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[instrument(level = "trace")]
pub async fn signup(Json(_request): Json<SignupRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
