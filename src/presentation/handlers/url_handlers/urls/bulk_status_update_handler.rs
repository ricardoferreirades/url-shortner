use crate::application::dto::{
    requests::BulkStatusUpdateRequest,
    responses::{BatchOperationResponse, BatchOperationResult},
    ErrorResponse,
};
use crate::presentation::handlers::ConcreteAppState;
use axum::{
    extract::State,
    http::HeaderMap,
    http::{header, StatusCode},
    Json,
};
use tracing::{info, warn};

/// Handler for bulk status updates
#[utoipa::path(
    patch,
    path = "/urls/bulk/status",
    request_body = BulkStatusUpdateRequest,
    responses(
        (status = 200, description = "Status updated successfully", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "bulk-operations"
)]
pub async fn bulk_status_update_handler(
    State(app_state): State<ConcreteAppState>,
    headers: HeaderMap,
    Json(request): Json<BulkStatusUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)> {
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
        "Received bulk status update request: {} for {} URLs (user: {})",
        request.status,
        request.url_ids.len(),
        user.id
    );

    let status = match request.status.as_str() {
        "active" => crate::domain::entities::UrlStatus::Active,
        "inactive" => crate::domain::entities::UrlStatus::Inactive,
        _ => {
            let error_response = ErrorResponse {
                error: "INVALID_STATUS".to_string(),
                message: "Invalid status. Must be 'active' or 'inactive'".to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    match app_state
        .url_service
        .batch_update_status(&request.url_ids, status, Some(user.id))
        .await
    {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_status".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result
                    .results
                    .into_iter()
                    .map(|r| BatchOperationResult {
                        url_id: r.url_id,
                        success: r.success,
                        error: r.error,
                    })
                    .collect(),
            };
            info!(
                "Bulk status update completed: {} successful, {} failed",
                result.successful, result.failed
            );
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Bulk status update failed: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_UPDATE_FAILED".to_string(),
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
    fn test_bulk_status_update_request_deserialize() {
        let json = r#"{"url_ids":[1,2,3],"status":"active"}"#;
        let request: Result<BulkStatusUpdateRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
    }

    #[test]
    fn test_invalid_status_error() {
        let error = ErrorResponse {
            error: "INVALID_STATUS".to_string(),
            message: "Invalid status. Must be 'active' or 'inactive'".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.error, "INVALID_STATUS");
    }

    #[test]
    fn test_bulk_update_failed_error() {
        let error = ErrorResponse {
            error: "BULK_UPDATE_FAILED".to_string(),
            message: "Failed to update status".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.error, "BULK_UPDATE_FAILED");
    }
}
