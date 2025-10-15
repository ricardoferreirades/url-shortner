use crate::application::dto::{
    requests::BulkDeleteRequest,
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

/// Handler for bulk URL deletion
#[utoipa::path(
    delete,
    path = "/urls/bulk",
    request_body = BulkDeleteRequest,
    responses(
        (status = 200, description = "URLs deleted successfully", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "bulk-operations"
)]
pub async fn bulk_delete_handler(
    State(app_state): State<ConcreteAppState>,
    headers: HeaderMap,
    Json(request): Json<BulkDeleteRequest>,
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
        "Received bulk delete request for {} URLs (user: {}, force: {:?})",
        request.url_ids.len(),
        user.id,
        request.force
    );

    // Add safety check for bulk deletion
    if request.url_ids.len() > 100 && request.force != Some(true) {
        let error_response = ErrorResponse {
            error: "BULK_DELETE_LIMIT_EXCEEDED".to_string(),
            message: "Bulk deletion of more than 100 URLs requires force=true".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    match app_state
        .url_service
        .batch_delete_urls(&request.url_ids, Some(user.id))
        .await
    {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "delete".to_string(),
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
                "Bulk delete completed: {} successful, {} failed",
                result.successful, result.failed
            );
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Bulk delete failed: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_DELETE_FAILED".to_string(),
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
    fn test_bulk_delete_request_deserialize() {
        let json = r#"{"url_ids":[1,2,3]}"#;
        let request: Result<BulkDeleteRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.url_ids.len(), 3);
    }

    #[test]
    fn test_bulk_delete_limit_error() {
        let error = ErrorResponse {
            error: "BULK_DELETE_LIMIT_EXCEEDED".to_string(),
            message: "Bulk deletion of more than 100 URLs requires force=true".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.error, "BULK_DELETE_LIMIT_EXCEEDED");
    }

    #[test]
    fn test_bulk_delete_failed_error() {
        let error = ErrorResponse {
            error: "BULK_DELETE_FAILED".to_string(),
            message: "Failed to delete URLs".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.error, "BULK_DELETE_FAILED");
    }

    #[test]
    fn test_batch_operation_response_structure() {
        let response = BatchOperationResponse {
            operation: "delete".to_string(),
            total_processed: 3,
            successful: 2,
            failed: 1,
            results: vec![],
        };
        assert_eq!(response.operation, "delete");
        assert_eq!(response.total_processed, 3);
    }
}
