#![allow(dead_code)]
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Logging middleware configuration
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    /// Create a trace layer for request/response logging
    pub fn create_trace_layer() -> impl tower::Layer<axum::Router> + Clone {
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO))
    }

    /// Create a simple trace layer for development
    pub fn simple() -> impl tower::Layer<axum::Router> + Clone {
        Self::create_trace_layer()
    }
}
