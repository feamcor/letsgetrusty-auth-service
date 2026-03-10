use crate::config::LogLevel;
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use std::time::Duration;
use tracing::Level;
use tracing::Span;
use tracing::info;

pub fn init_tracing(log_level: &LogLevel) {
    tracing_subscriber::fmt().with_max_level(log_level).init();
    info!("Initialized: Tracing");
}

// Creates a new tracing span with a unique request ID for each incoming request.
// This helps in tracking and correlating logs for individual requests.
pub fn make_span_with_request_id(request: &Request<Body>) -> Span {
    let request_id = uuid::Uuid::now_v7();
    tracing::span!(
        Level::TRACE,
        "ApiRequest",
        method = tracing::field::display(request.method()),
        uri = tracing::field::display(request.uri()),
        version = tracing::field::debug(request.version()),
        request_id = tracing::field::display(request_id),
    )
}

// Logs an event indicating the start of a request.
pub fn on_request(_request: &Request<Body>, _span: &Span) {
    tracing::event!(Level::TRACE, "[START]");
}

// Logs an event indicating the end of a request, including its latency and status code.
// If the status code indicates an error (4xx or 5xx), it logs at the ERROR level.
pub fn on_response(response: &Response, latency: Duration, _span: &Span) {
    let status = response.status();
    let status_code = status.as_u16();
    let status_code_class = status_code / 100;

    match status_code_class {
        4..=5 => {
            tracing::event!(
                Level::ERROR,
                latency = ?latency,
                status = status_code,
                "[END]"
            );
        }
        _ => {
            tracing::event!(
                Level::TRACE,
                latency = ?latency,
                status = status_code,
                "[END]"
            );
        }
    }
}
