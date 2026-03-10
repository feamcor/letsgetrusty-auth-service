use crate::config::LogLevel;
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing(log_level: &LogLevel) -> color_eyre::eyre::Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer().compact();
    let filter_layer = tracing_subscriber::EnvFilter::try_new::<String>(log_level.to_string())?;
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(tracing_error::ErrorLayer::default())
        .init();
    tracing::info!("Initialized: Tracing");
    Ok(())
}

// Creates a new tracing span with a unique request ID for each incoming request.
// This helps in tracking and correlating logs for individual requests.
pub fn make_span_with_request_id(request: &Request<Body>) -> tracing::Span {
    let request_id = uuid::Uuid::now_v7();
    tracing::span!(
        tracing::Level::TRACE,
        "ApiRequest",
        method = display(request.method()),
        uri = display(request.uri()),
        version = debug(request.version()),
        request_id = display(request_id),
    )
}

// Logs an event indicating the start of a request.
pub fn on_request(_request: &Request<Body>, _span: &tracing::Span) {
    tracing::event!(tracing::Level::TRACE, "[START]");
}

// Logs an event indicating the end of a request, including its latency and status code.
// If the status code indicates an error (4xx or 5xx), it logs at the ERROR level.
pub fn on_response(response: &Response, latency: std::time::Duration, _span: &tracing::Span) {
    let status = response.status();
    let status_code = status.as_u16();
    let status_code_class = status_code / 100;

    match status_code_class {
        4..=5 => {
            tracing::event!(
                tracing::Level::ERROR,
                latency = ?latency,
                status = status_code,
                "[END]"
            );
        }
        _ => {
            tracing::event!(
                tracing::Level::TRACE,
                latency = ?latency,
                status = status_code,
                "[END]"
            );
        }
    }
}
