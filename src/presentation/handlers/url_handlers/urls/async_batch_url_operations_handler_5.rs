use crate::application::dto::{requests::{ShortenUrlRequest, BulkShortenUrlsRequest, BatchUrlOperationRequest, BulkStatusUpdateRequest, BulkExpirationUpdateRequest, BulkDeleteRequest}, responses::{ShortenUrlResponse, BatchOperationResponse, BatchOperationResult, BulkOperationProgress}, ErrorResponse};
use crate::domain::repositories::UrlRepository;
use axum::{extract::State, http::{StatusCode, header}, response::Redirect, Json, http::HeaderMap};
use tracing::{info, warn};
use crate::presentation::handlers::app_state::AppState;

/// Handler for async batch URL operations with progress tracking
#[utoipa::path(
    post,
    path = "/urls/batch/async",
    request_body = BatchUrlOperationRequest,
    responses(
        (status = 202, description = "Batch operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_batch_url_operations_handler_5<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
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

    let total_items = request.url_ids.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async batch operation {:?} for {} URLs (user: {}, operation: {})", request.operation, total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_operation(
        operation_id.clone(),
        request.operation,
        request.url_ids,
        request.data,
        Some(user.id),
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async batch operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async batch operation: {}", error);
            let error_response = ErrorResponse {
                error: "BATCH_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}
