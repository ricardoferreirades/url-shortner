use crate::application::dto::responses::{BulkOperationProgress, ErrorResponse};
use crate::domain::services::ProgressServiceError;
use crate::presentation::handlers::app_state::AppState;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use tracing::{info, warn};

/// Handler for getting bulk operation progress
#[utoipa::path(
    get,
    path = "/urls/bulk/progress/{operation_id}",
    params(
        ("operation_id" = String, Path, description = "Operation ID to check progress for")
    ),
    responses(
        (status = 200, description = "Progress retrieved successfully", body = BulkOperationProgress),
        (status = 404, description = "Operation not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn get_bulk_operation_progress_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    Path(operation_id): Path<String>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)>
where
    R: crate::domain::repositories::UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,
{
    let progress_service = &app_state.progress_service;
    info!("Getting progress for operation: {}", operation_id);

    match progress_service.get_progress(&operation_id).await {
        Ok(progress) => {
            info!("Retrieved progress for operation {}: {}% complete", operation_id, progress.progress_percentage);
            Ok((StatusCode::OK, Json(progress)))
        }
        Err(ProgressServiceError::OperationNotFound) => {
            warn!("Operation not found: {}", operation_id);
            let error_response = ErrorResponse {
                error: "OPERATION_NOT_FOUND".to_string(),
                message: "Operation not found or may have expired".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Error retrieving progress for operation {}: {}", operation_id, error);
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "Failed to retrieve operation progress".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Handler for cancelling a bulk operation
#[utoipa::path(
    delete,
    path = "/urls/bulk/progress/{operation_id}",
    params(
        ("operation_id" = String, Path, description = "Operation ID to cancel")
    ),
    responses(
        (status = 204, description = "Operation cancelled successfully"),
        (status = 404, description = "Operation not found", body = ErrorResponse),
        (status = 400, description = "Operation cannot be cancelled", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn cancel_bulk_operation_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    Path(operation_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    R: crate::domain::repositories::UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,
{
    let progress_service = &app_state.progress_service;
    info!("Cancelling operation: {}", operation_id);

    match progress_service.cancel_operation(&operation_id).await {
        Ok(_) => {
            info!("Successfully cancelled operation: {}", operation_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(ProgressServiceError::OperationNotFound) => {
            warn!("Operation not found for cancellation: {}", operation_id);
            let error_response = ErrorResponse {
                error: "OPERATION_NOT_FOUND".to_string(),
                message: "Operation not found or may have expired".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Error cancelling operation {}: {}", operation_id, error);
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "Failed to cancel operation".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Handler for getting all operations for a user
#[utoipa::path(
    get,
    path = "/urls/bulk/operations",
    responses(
        (status = 200, description = "Operations retrieved successfully", body = [BulkOperationProgress]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn get_user_operations_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    // In a real implementation, you'd extract user_id from the auth token
    // For now, we'll use a placeholder user_id
) -> Result<(StatusCode, Json<Vec<BulkOperationProgress>>), (StatusCode, Json<ErrorResponse>)>
where
    R: crate::domain::repositories::UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,
{
    let progress_service = &app_state.progress_service;
    let user_id = 1; // This should come from authentication
    info!("Getting operations for user: {}", user_id);

    match progress_service.get_user_operations(user_id).await {
        Ok(operations) => {
            info!("Retrieved {} operations for user {}", operations.len(), user_id);
            Ok((StatusCode::OK, Json(operations)))
        }
        Err(error) => {
            warn!("Error retrieving operations for user {}: {}", user_id, error);
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "Failed to retrieve user operations".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
