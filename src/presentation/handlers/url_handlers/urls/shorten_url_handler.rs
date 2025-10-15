use crate::application::dto::{
    requests::ShortenUrlRequest, responses::ShortenUrlResponse, ErrorResponse,
};
use crate::presentation::handlers::ConcreteAppState;
use axum::{
    extract::State,
    http::HeaderMap,
    http::{header, StatusCode},
    Json,
};
use tracing::{info, warn};

/// Handler for shortening URLs
#[utoipa::path(
    post,
    path = "/shorten",
    request_body = ShortenUrlRequest,
    responses(
        (status = 201, description = "URL shortened successfully", body = ShortenUrlResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn shorten_url_handler(
    State(app_state): State<ConcreteAppState>,
    headers: HeaderMap,
    Json(request): Json<ShortenUrlRequest>,
) -> Result<(StatusCode, Json<ShortenUrlResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Require Authorization: Bearer <token>
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let token = match auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) if !t.is_empty() => t,
        _ => {
            let error_response = ErrorResponse {
                error: "UNAUTHORIZED".to_string(),
                message: "Missing or invalid Authorization header".to_string(),
                status_code: StatusCode::UNAUTHORIZED.as_u16(),
            };
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    // Verify token and get user
    let user = match app_state.auth_service.verify_token(token).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Token verification failed: {}", e);
            let error_response = ErrorResponse {
                error: "INVALID_TOKEN".to_string(),
                message: "Invalid or expired token".to_string(),
                status_code: StatusCode::UNAUTHORIZED.as_u16(),
            };
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    let user_id = Some(user.id);
    info!(
        "Received shorten URL request for: {} (user: {:?})",
        request.url, user_id
    );

    match app_state
        .shorten_url_use_case
        .execute(request, user_id)
        .await
    {
        Ok(response) => {
            info!(
                "Successfully shortened URL: {} -> {}",
                response.original_url, response.short_url
            );
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(error) => {
            warn!("Failed to shorten URL: {}", error);
            let error_response = ErrorResponse {
                error: "SHORTEN_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_request_deserialize() {
        let json = r#"{"url":"https://example.com"}"#;
        let request: Result<ShortenUrlRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.url, "https://example.com");
    }

    #[test]
    fn test_shorten_request_with_custom_code() {
        let json = r#"{"url":"https://example.com","custom_short_code":"mycustom"}"#;
        let request: Result<ShortenUrlRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.custom_short_code, Some("mycustom".to_string()));
    }

    #[test]
    fn test_unauthorized_error_response() {
        let error = ErrorResponse {
            error: "UNAUTHORIZED".to_string(),
            message: "Missing or invalid Authorization header".to_string(),
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
        };
        assert_eq!(error.status_code, 401);
    }

    #[test]
    fn test_invalid_token_error_response() {
        let error = ErrorResponse {
            error: "INVALID_TOKEN".to_string(),
            message: "Invalid or expired token".to_string(),
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
        };
        assert_eq!(error.error, "INVALID_TOKEN");
    }

    #[test]
    fn test_shorten_failed_error_response() {
        let error = ErrorResponse {
            error: "SHORTEN_FAILED".to_string(),
            message: "Invalid URL".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.status_code, 400);
    }

    #[test]
    fn test_shorten_response_serialization() {
        use chrono::Utc;
        let response = ShortenUrlResponse {
            short_url: "http://short.url/abc123".to_string(),
            original_url: "https://example.com".to_string(),
            short_code: "abc123".to_string(),
            created_at: Utc::now().to_rfc3339(),
            expiration_date: None,
        };
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("abc123"));
    }
}
