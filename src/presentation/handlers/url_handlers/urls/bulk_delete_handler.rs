use crate::application::dto::{requests::{ShortenUrlRequest, BulkShortenUrlsRequest, BatchUrlOperationRequest, BulkStatusUpdateRequest, BulkExpirationUpdateRequest, BulkDeleteRequest}, responses::{ShortenUrlResponse, BatchOperationResponse, BatchOperationResult, BulkOperationProgress}, ErrorResponse};
use crate::domain::repositories::UrlRepository;
use axum::{extract::State, http::{StatusCode, header}, response::Redirect, Json, http::HeaderMap};
use tracing::{info, warn};
use crate::presentation::handlers::app_state::AppState;

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
    tag = "url-shortener"
)]
pub async fn bulk_delete_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    headers: HeaderMap,
    Json(request): Json<BulkDeleteRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,{
    // Require Authorization: Bearer <token>
    let auth_header = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
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

    info!("Received bulk delete request for {} URLs (user: {}, force: {:?})", request.url_ids.len(), user.id, request.force);

    // Add safety check for bulk deletion
    if request.url_ids.len() > 100 && request.force != Some(true) {
        let error_response = ErrorResponse {
            error: "BULK_DELETE_LIMIT_EXCEEDED".to_string(),
            message: "Bulk deletion of more than 100 URLs requires force=true".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    match app_state.url_service.batch_delete_urls(&request.url_ids, Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "delete".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk delete completed: {} successful, {} failed", result.successful, result.failed);
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
