use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use governor::{
    clock::{Clock, DefaultClock},
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use tower_http::{compression::CompressionLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::warn;

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub max_request_size: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,       // 60 requests per minute per IP
            burst_size: 10,                // Allow bursts of up to 10 requests
            max_request_size: 1024 * 1024, // 1MB max request size
        }
    }
}

/// Rate limiter instance
pub type AppRateLimiter = RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

/// Create rate limiter
pub fn create_rate_limiter(config: &RateLimitConfig) -> AppRateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(config.requests_per_minute).unwrap())
        .allow_burst(NonZeroU32::new(config.burst_size).unwrap());

    let state = DefaultKeyedStateStore::new();
    let clock = DefaultClock::default();

    RateLimiter::new(quota, state, &clock)
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, axum::Json<RateLimitError>)> {
    // For now, we'll implement a simple per-IP rate limiter
    // In a real implementation, you'd extract the client IP from headers or connection info

    // Extract IP from request (simplified - in production use proper IP extraction)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|header| header.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Get rate limiter from application state (we'll add this to the app state)
    // For now, we'll create a temporary one
    let rate_limiter = create_rate_limiter(&RateLimitConfig::default());

    match rate_limiter.check_key(&client_ip) {
        Ok(_) => {
            // Rate limit OK, continue
            Ok(next.run(request).await)
        }
        Err(negative) => {
            // Rate limit exceeded
            let retry_after = negative
                .wait_time_from(DefaultClock::default().now())
                .as_secs();
            warn!(
                "Rate limit exceeded for IP: {}, retry after {} seconds",
                client_ip, retry_after
            );

            Err(handle_rate_limit_error(retry_after))
        }
    }
}

/// Create request size limiting middleware
pub fn create_request_size_limiter(max_size: usize) -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(max_size)
}

/// Create compression middleware
pub fn create_compression_layer() -> CompressionLayer {
    CompressionLayer::new().br(true).gzip(true).deflate(true)
}

/// Create tracing middleware
pub fn create_tracing_layer(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Add security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    // Add HSTS header for HTTPS
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );

    // Add Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'".parse().unwrap(),
    );

    Ok(response)
}

/// Rate limiting error response
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct RateLimitError {
    pub error: String,
    pub message: String,
    pub retry_after: u64,
}

/// Handle rate limiting errors
pub fn handle_rate_limit_error(retry_after: u64) -> (StatusCode, axum::Json<RateLimitError>) {
    warn!("Rate limit exceeded, retry after {} seconds", retry_after);

    (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(RateLimitError {
            error: "rate_limit_exceeded".to_string(),
            message: "Too many requests. Please try again later.".to_string(),
            retry_after,
        }),
    )
}

/// Create middleware layers individually
pub fn create_request_size_layer(config: &RateLimitConfig) -> RequestBodyLimitLayer {
    create_request_size_limiter(config.max_request_size)
}

pub fn create_tracing_layer_simple(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    create_tracing_layer()
}

pub fn create_compression_layer_simple() -> CompressionLayer {
    create_compression_layer()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.burst_size, 10);
        assert_eq!(config.max_request_size, 1024 * 1024);
    }

    #[tokio::test]
    async fn test_security_headers_middleware() {
        let app = Router::new()
            .route("/", get(|| async { "test" }))
            .layer(axum::middleware::from_fn(security_headers_middleware));

        let request = Request::builder().uri("/").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();

        // Check security headers are present
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-XSS-Protection"));
        assert!(headers.contains_key("Referrer-Policy"));
        assert!(headers.contains_key("Permissions-Policy"));
        assert!(headers.contains_key("Strict-Transport-Security"));
        assert!(headers.contains_key("Content-Security-Policy"));
    }

    #[test]
    fn test_rate_limit_error_response() {
        let error = handle_rate_limit_error(60);
        assert_eq!(error.0, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(error.1.retry_after, 60);
        assert_eq!(error.1.error, "rate_limit_exceeded");
    }

    #[test]
    fn test_create_rate_limiter() {
        let config = RateLimitConfig::default();
        let rate_limiter = create_rate_limiter(&config);
        // Rate limiter should be created successfully
        assert!(rate_limiter.check_key(&"test-ip".to_string()).is_ok());
    }
}
