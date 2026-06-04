//! Cross-cutting helpers: typed API errors ([`api_error`]), JWT/cookie handling ([`auth`]), and
//! request-id tracing spans ([`tracing`]).

pub mod api_error;
pub mod auth;
pub mod tracing;
