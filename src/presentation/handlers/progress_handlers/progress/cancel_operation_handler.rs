use crate::application::dto::responses::ErrorResponse;
use crate::domain::services::ProgressServiceError;
use crate::presentation::handlers::ConcreteAppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use tracing::{info, warn};

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
    tag = "bulk-operations"
)]
pub async fn cancel_bulk_operation_handler(
    State(app_state): State<ConcreteAppState>,
    Path(operation_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_not_found_error() {
        let error = ErrorResponse {
            error: "OPERATION_NOT_FOUND".to_string(),
            message: "Operation not found or may have expired".to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        };
        assert_eq!(error.status_code, 404);
    }

    #[test]
    fn test_internal_error() {
        let error = ErrorResponse {
            error: "INTERNAL_ERROR".to_string(),
            message: "Failed to cancel operation".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error.error, "INTERNAL_ERROR");
    }
}
