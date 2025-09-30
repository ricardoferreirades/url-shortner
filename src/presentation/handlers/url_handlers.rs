use crate::application::dto::{requests::{ShortenUrlRequest, BulkShortenUrlsRequest, BatchUrlOperationRequest, BulkStatusUpdateRequest, BulkExpirationUpdateRequest, BulkDeleteRequest}, responses::{ShortenUrlResponse, BatchOperationResponse, BatchOperationResult, BulkOperationProgress}, ErrorResponse};
use crate::domain::repositories::UrlRepository;
use axum::{extract::State, http::{StatusCode, header}, response::Redirect, Json, http::HeaderMap};
use tracing::{info, warn};
use super::app_state::AppState;

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
pub async fn shorten_url_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<ShortenUrlRequest>,
) -> Result<(StatusCode, Json<ShortenUrlResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    info!("Received shorten URL request for: {} (user: {:?})", request.url, user_id);

    match app_state.shorten_url_use_case.execute(request, user_id).await {
        Ok(response) => {
            info!("Successfully shortened URL: {} -> {}", response.original_url, response.short_url);
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for batch URL operations
#[utoipa::path(
    post,
    path = "/urls/batch",
    request_body = BatchUrlOperationRequest,
    responses(
        (status = 200, description = "Batch operation completed", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received batch operation request: {:?} for {} URLs (user: {})", request.operation, request.url_ids.len(), user.id);

    match app_state.url_service.process_batch_operations(
        &request.operation,
        &request.url_ids,
        request.data.as_ref(),
        Some(user.id),
    ).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: format!("{:?}", request.operation),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Batch operation completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Batch operation failed: {}", error);
            let error_response = ErrorResponse {
                error: "BATCH_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
    tag = "url-shortener"
)]
pub async fn bulk_status_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkStatusUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk status update request: {} for {} URLs (user: {})", request.status, request.url_ids.len(), user.id);

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

    match app_state.url_service.batch_update_status(&request.url_ids, status, Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_status".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk status update completed: {} successful, {} failed", result.successful, result.failed);
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for bulk expiration updates
#[utoipa::path(
    patch,
    path = "/urls/bulk/expiration",
    request_body = BulkExpirationUpdateRequest,
    responses(
        (status = 200, description = "Expiration updated successfully", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn bulk_expiration_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkExpirationUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk expiration update request for {} URLs (user: {})", request.url_ids.len(), user.id);

    match app_state.url_service.batch_update_expiration(&request.url_ids, Some(request.expiration_date), Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_expiration".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk expiration update completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Bulk expiration update failed: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_UPDATE_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
pub async fn bulk_delete_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkDeleteRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for redirecting to original URL
#[utoipa::path(
    get,
    path = "/{short_code}",
    params(
        ("short_code" = String, Path, description = "Short code to redirect")
    ),
    responses(
        (status = 301, description = "Redirect to original URL"),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Short code not found", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn redirect_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    axum::extract::Path(short_code_str): axum::extract::Path<String>,
) -> Result<Redirect, (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
    info!("Received redirect request for short code: {}", short_code_str);

    // Parse and validate short code
    let short_code = match crate::domain::entities::ShortCode::new(short_code_str) {
        Ok(code) => code,
        Err(error) => {
            warn!("Invalid short code format: {}", error);
            let error_response = ErrorResponse {
                error: "INVALID_SHORT_CODE".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Find the URL with validation (checks expiration and status)
    match app_state.url_service.get_url_by_short_code_with_validation(&short_code).await {
        Ok(Some(url)) => {
            info!("Redirecting {} to {}", short_code.value(), url.original_url);
            Ok(Redirect::permanent(&url.original_url))
        }
        Ok(None) => {
            warn!("Short code not found or not accessible: {}", short_code.value());
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "Short code not found or no longer available".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Database error while looking up short code: {}", error);
            let error_response = ErrorResponse {
                error: "DATABASE_ERROR".to_string(),
                message: "Internal server error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for batch URL operations
#[utoipa::path(
    post,
    path = "/urls/batch",
    request_body = BatchUrlOperationRequest,
    responses(
        (status = 200, description = "Batch operation completed", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received batch operation request: {:?} for {} URLs (user: {})", request.operation, request.url_ids.len(), user.id);

    match app_state.url_service.process_batch_operations(
        &request.operation,
        &request.url_ids,
        request.data.as_ref(),
        Some(user.id),
    ).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: format!("{:?}", request.operation),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Batch operation completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Batch operation failed: {}", error);
            let error_response = ErrorResponse {
                error: "BATCH_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
    tag = "url-shortener"
)]
pub async fn bulk_status_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkStatusUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk status update request: {} for {} URLs (user: {})", request.status, request.url_ids.len(), user.id);

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

    match app_state.url_service.batch_update_status(&request.url_ids, status, Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_status".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk status update completed: {} successful, {} failed", result.successful, result.failed);
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for bulk expiration updates
#[utoipa::path(
    patch,
    path = "/urls/bulk/expiration",
    request_body = BulkExpirationUpdateRequest,
    responses(
        (status = 200, description = "Expiration updated successfully", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn bulk_expiration_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkExpirationUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk expiration update request for {} URLs (user: {})", request.url_ids.len(), user.id);

    match app_state.url_service.batch_update_expiration(&request.url_ids, Some(request.expiration_date), Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_expiration".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk expiration update completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Bulk expiration update failed: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_UPDATE_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
pub async fn bulk_delete_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkDeleteRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for bulk shortening URLs
#[utoipa::path(
    post,
    path = "/urls/bulk",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 201, description = "URLs shortened successfully", body = [ShortenUrlResponse]),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<Vec<ShortenUrlResponse>>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let mut responses: Vec<ShortenUrlResponse> = Vec::with_capacity(request.items.len());

    for item in request.items {
        let req = ShortenUrlRequest { url: item.url, custom_short_code: item.custom_short_code, expiration_date: item.expiration_date };
        match app_state.shorten_url_use_case.execute(req, user_id).await {
            Ok(resp) => responses.push(resp),
            Err(err) => {
                let error_response = ErrorResponse {
                    error: "SHORTEN_FAILED".to_string(),
                    message: err.to_string(),
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                };
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }
        }
    }

    Ok((StatusCode::CREATED, Json(responses)))
}

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
    tag = "url-shortener"
)]
pub async fn deactivate_url_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received deactivate URL request for ID: {} (user: {})", id, user.id);

    match app_state.url_service.deactivate_url(id, Some(user.id)).await {
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for batch URL operations
#[utoipa::path(
    post,
    path = "/urls/batch",
    request_body = BatchUrlOperationRequest,
    responses(
        (status = 200, description = "Batch operation completed", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received batch operation request: {:?} for {} URLs (user: {})", request.operation, request.url_ids.len(), user.id);

    match app_state.url_service.process_batch_operations(
        &request.operation,
        &request.url_ids,
        request.data.as_ref(),
        Some(user.id),
    ).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: format!("{:?}", request.operation),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Batch operation completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Batch operation failed: {}", error);
            let error_response = ErrorResponse {
                error: "BATCH_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
    tag = "url-shortener"
)]
pub async fn bulk_status_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkStatusUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk status update request: {} for {} URLs (user: {})", request.status, request.url_ids.len(), user.id);

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

    match app_state.url_service.batch_update_status(&request.url_ids, status, Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_status".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk status update completed: {} successful, {} failed", result.successful, result.failed);
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for bulk expiration updates
#[utoipa::path(
    patch,
    path = "/urls/bulk/expiration",
    request_body = BulkExpirationUpdateRequest,
    responses(
        (status = 200, description = "Expiration updated successfully", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn bulk_expiration_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkExpirationUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk expiration update request for {} URLs (user: {})", request.url_ids.len(), user.id);

    match app_state.url_service.batch_update_expiration(&request.url_ids, Some(request.expiration_date), Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_expiration".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk expiration update completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Bulk expiration update failed: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_UPDATE_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
pub async fn bulk_delete_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkDeleteRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for reactivating a URL
#[utoipa::path(
    patch,
    path = "/urls/{id}/reactivate",
    params(
        ("id" = i32, Path, description = "URL ID to reactivate")
    ),
    responses(
        (status = 204, description = "URL reactivated successfully"),
        (status = 404, description = "URL not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn reactivate_url_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received reactivate URL request for ID: {} (user: {})", id, user.id);

    match app_state.url_service.reactivate_url(id, Some(user.id)).await {
        Ok(true) => {
            info!("Successfully reactivated URL ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => {
            warn!("URL not found or not owned by user: {}", id);
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "URL not found or you don't have permission to reactivate it".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Failed to reactivate URL {}: {}", id, error);
            let error_response = ErrorResponse {
                error: "REACTIVATE_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for batch URL operations
#[utoipa::path(
    post,
    path = "/urls/batch",
    request_body = BatchUrlOperationRequest,
    responses(
        (status = 200, description = "Batch operation completed", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received batch operation request: {:?} for {} URLs (user: {})", request.operation, request.url_ids.len(), user.id);

    match app_state.url_service.process_batch_operations(
        &request.operation,
        &request.url_ids,
        request.data.as_ref(),
        Some(user.id),
    ).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: format!("{:?}", request.operation),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Batch operation completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Batch operation failed: {}", error);
            let error_response = ErrorResponse {
                error: "BATCH_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
    tag = "url-shortener"
)]
pub async fn bulk_status_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkStatusUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk status update request: {} for {} URLs (user: {})", request.status, request.url_ids.len(), user.id);

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

    match app_state.url_service.batch_update_status(&request.url_ids, status, Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_status".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk status update completed: {} successful, {} failed", result.successful, result.failed);
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for bulk expiration updates
#[utoipa::path(
    patch,
    path = "/urls/bulk/expiration",
    request_body = BulkExpirationUpdateRequest,
    responses(
        (status = 200, description = "Expiration updated successfully", body = BatchOperationResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn bulk_expiration_update_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkExpirationUpdateRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    info!("Received bulk expiration update request for {} URLs (user: {})", request.url_ids.len(), user.id);

    match app_state.url_service.batch_update_expiration(&request.url_ids, Some(request.expiration_date), Some(user.id)).await {
        Ok(result) => {
            let response = BatchOperationResponse {
                operation: "update_expiration".to_string(),
                total_processed: result.total_processed,
                successful: result.successful,
                failed: result.failed,
                results: result.results.into_iter().map(|r| BatchOperationResult {
                    url_id: r.url_id,
                    success: r.success,
                    error: r.error,
                }).collect(),
            };
            info!("Bulk expiration update completed: {} successful, {} failed", result.successful, result.failed);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(error) => {
            warn!("Bulk expiration update failed: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_UPDATE_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
pub async fn bulk_delete_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkDeleteRequest>,
) -> Result<(StatusCode, Json<BatchOperationResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

/// Handler for async bulk URL shortening with progress tracking
#[utoipa::path(
    post,
    path = "/urls/bulk/async",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 202, description = "Bulk operation started", body = BulkOperationProgress),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn async_bulk_shorten_urls_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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

    let user_id = Some(user.id);
    let total_items = request.items.len();
    
    // Create operation for progress tracking
    let operation_id = app_state.progress_service.create_operation(total_items).await;
    
    info!("Starting async bulk URL shortening for {} URLs (user: {}, operation: {})", total_items, user.id, operation_id);

    // Start background processing
    match app_state.bulk_processor.process_bulk_url_creation(
        operation_id.clone(),
        request.items,
        user_id,
    ).await {
        Ok(_) => {
            let progress = app_state.progress_service.get_progress(&operation_id).await.unwrap();
            info!("Started async bulk URL shortening operation: {}", operation_id);
            Ok((StatusCode::ACCEPTED, Json(progress)))
        }
        Err(error) => {
            warn!("Failed to start async bulk URL shortening: {}", error);
            let error_response = ErrorResponse {
                error: "BULK_OPERATION_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

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
pub async fn async_batch_url_operations_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<BatchUrlOperationRequest>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
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
