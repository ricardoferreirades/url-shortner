#![allow(dead_code)]
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Redirect},
    Json,
};
use tracing::info;

use crate::application::dto::{
    requests::ShortenUrlRequest, responses::ShortenUrlResponse, ErrorResponse,
};
use crate::domain::repositories::UrlRepository;
use crate::presentation::handlers::ConcreteAppState;

/// Axum-specific controller for URL operations
/// This separates HTTP framework concerns from business logic
pub struct AxumUrlController;

impl AxumUrlController {
    /// Handle URL shortening requests
    pub async fn shorten_url(
        State(app_state): State<ConcreteAppState>,
        Json(request): Json<ShortenUrlRequest>,
    ) -> Result<(StatusCode, Json<ShortenUrlResponse>), (StatusCode, Json<ErrorResponse>)> {
        info!(
            "Axum controller: Received shorten URL request for: {}",
            request.url
        );

        match app_state.shorten_url_use_case.execute(request, None).await {
            Ok(response) => {
                info!(
                    "Axum controller: Successfully shortened URL: {} -> {}",
                    response.original_url, response.short_url
                );
                Ok((StatusCode::CREATED, Json(response)))
            }
            Err(error) => {
                let error_response = ErrorResponse {
                    error: "SHORTEN_FAILED".to_string(),
                    message: error.to_string(),
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                };
                Err((StatusCode::BAD_REQUEST, Json(error_response)))
            }
        }
    }

    /// Handle URL redirects
    pub async fn redirect(
        State(app_state): State<ConcreteAppState>,
        axum::extract::Path(short_code_str): axum::extract::Path<String>,
    ) -> Result<Redirect, (StatusCode, Json<ErrorResponse>)> {
        info!(
            "Axum controller: Received redirect request for short code: {}",
            short_code_str
        );

        // Parse and validate short code
        let short_code = match crate::domain::entities::ShortCode::new(short_code_str) {
            Ok(code) => code,
            Err(error) => {
                let error_response = ErrorResponse {
                    error: "INVALID_SHORT_CODE".to_string(),
                    message: error.to_string(),
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                };
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }
        };

        // Find the URL
        match app_state
            .url_repository
            .find_by_short_code(&short_code)
            .await
        {
            Ok(Some(url)) => {
                info!(
                    "Axum controller: Redirecting {} to {}",
                    short_code.value(),
                    url.original_url
                );
                Ok(Redirect::permanent(&url.original_url))
            }
            Ok(None) => {
                let error_response = ErrorResponse {
                    error: "NOT_FOUND".to_string(),
                    message: "Short code not found".to_string(),
                    status_code: StatusCode::NOT_FOUND.as_u16(),
                };
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            }
            Err(_error) => {
                let error_response = ErrorResponse {
                    error: "INTERNAL_SERVER_ERROR".to_string(),
                    message: "An internal server error occurred".to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                };
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            }
        }
    }

    /// Handle welcome page
    pub async fn welcome() -> Html<&'static str> {
        Html(
            r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>URL Shortener</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; }
                .container { max-width: 600px; margin: 0 auto; }
                .api-docs { margin-top: 20px; }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🔗 URL Shortener API</h1>
                <p>Welcome to the URL Shortener service!</p>
                <div class="api-docs">
                    <h2>API Documentation</h2>
                    <p><a href="/docs">📚 Swagger UI Documentation</a></p>
                    <p><a href="/health">🏥 Health Check</a></p>
                </div>
            </div>
        </body>
        </html>
        "#,
        )
    }

    /// Handle health checks
    pub async fn health_check() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": "url-shortener"
        }))
    }
}
