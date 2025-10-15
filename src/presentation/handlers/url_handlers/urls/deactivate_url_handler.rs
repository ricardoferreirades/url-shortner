use crate::application::dto::ErrorResponse;
use crate::presentation::handlers::ConcreteAppState;
use axum::{
    extract::State,
    http::HeaderMap,
    http::{header, StatusCode},
    Json,
};
use tracing::{info, warn};

/// Handler for deactivating a URL (soft delete)
#[utoipa::path(
    delete,
    path = "/urls/{id}",
    params(
        ("id" = i32, Path, description = "URL ID to deactivate")
    ),
    responses(
        (status = 204, description = "URL deactivated successfully"),
        (status = 404, description = "URL not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-management"
)]
pub async fn deactivate_url_handler(
    State(app_state): State<ConcreteAppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
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

    info!(
        "Received deactivate URL request for ID: {} (user: {})",
        id, user.id
    );

    match app_state
        .url_service
        .deactivate_url(id, Some(user.id))
        .await
    {
        Ok(true) => {
            info!("Successfully deactivated URL ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => {
            warn!("URL not found or not owned by user: {}", id);
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "URL not found or you don't have permission to deactivate it".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Failed to deactivate URL {}: {}", id, error);
            let error_response = ErrorResponse {
                error: "DEACTIVATE_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unauthorized_error() {
        let error = ErrorResponse {
            error: "UNAUTHORIZED".to_string(),
            message: "Missing or invalid Authorization header".to_string(),
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
        };
        assert_eq!(error.status_code, 401);
    }

    #[test]
    fn test_not_found_error() {
        let error = ErrorResponse {
            error: "NOT_FOUND".to_string(),
            message: "URL not found or you don't have permission to deactivate it".to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        };
        assert_eq!(error.status_code, 404);
    }

    #[test]
    fn test_deactivate_failed_error() {
        let error = ErrorResponse {
            error: "DEACTIVATE_FAILED".to_string(),
            message: "Failed to deactivate URL".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error.error, "DEACTIVATE_FAILED");
    }
}
