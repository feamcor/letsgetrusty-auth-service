use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::app_state::AppState;
use tracing::instrument;

#[allow(unused_imports)]
use tracing::Level;

#[instrument(level = Level::TRACE)]
pub async fn health(State(_state): State<AppState>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}