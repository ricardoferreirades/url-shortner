use tower_http::cors::{Any, CorsLayer};

/// CORS middleware configuration
pub struct CorsMiddleware;

impl CorsMiddleware {
    /// Create a CORS layer with development-friendly settings
    pub fn development() -> CorsLayer {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    }

    /// Create a CORS layer with production settings
    pub fn production(allowed_origins: Vec<String>) -> CorsLayer {
        // For now, use the same as development - in production you'd want to be more specific
        // TODO: Implement proper origin validation
        Self::development()
    }
}
