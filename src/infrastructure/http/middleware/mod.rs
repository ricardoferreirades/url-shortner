// HTTP Middleware implementations
// This allows us to organize middleware by functionality

pub mod cors_middleware;
pub mod logging_middleware;
pub mod error_middleware;

// Future: pub mod auth_middleware;
// Future: pub mod metrics_middleware;

pub use cors_middleware::*;
pub use logging_middleware::*;
pub use error_middleware::*;
