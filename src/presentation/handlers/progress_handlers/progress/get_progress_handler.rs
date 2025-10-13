use crate::application::dto::responses::{BulkOperationProgress, ErrorResponse};
use crate::domain::services::ProgressServiceError;
use crate::presentation::handlers::ConcreteAppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
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
    tag = "bulk-operations"
)]
pub async fn get_bulk_operation_progress_handler(
    State(app_state): State<ConcreteAppState>,
    Path(operation_id): Path<String>,
) -> Result<(StatusCode, Json<BulkOperationProgress>), (StatusCode, Json<ErrorResponse>)> {
    let progress_service = &app_state.progress_service;
    info!("Getting progress for operation: {}", operation_id);

    match progress_service.get_progress(&operation_id).await {
        Ok(progress) => {
            info!(
                "Retrieved progress for operation {}: {}% complete",
                operation_id, progress.progress_percentage
            );
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
            warn!(
                "Error retrieving progress for operation {}: {}",
                operation_id, error
            );
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "Failed to retrieve operation progress".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
