use crate::application::dto::responses::{BulkOperationProgress, ErrorResponse};
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, Json};
use tracing::{info, warn};

/// Handler for getting all operations for a user
#[utoipa::path(
    get,
    path = "/urls/bulk/operations",
    responses(
        (status = 200, description = "Operations retrieved successfully", body = [BulkOperationProgress]),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "bulk-operations"
)]
pub async fn get_user_operations_handler(
    State(app_state): State<ConcreteAppState>,
    // In a real implementation, you'd extract user_id from the auth token
    // For now, we'll use a placeholder user_id
) -> Result<(StatusCode, Json<Vec<BulkOperationProgress>>), (StatusCode, Json<ErrorResponse>)> {
    let progress_service = &app_state.progress_service;
    let user_id = 1; // This should come from authentication
    info!("Getting operations for user: {}", user_id);

    match progress_service.get_user_operations(user_id).await {
        Ok(operations) => {
            info!(
                "Retrieved {} operations for user {}",
                operations.len(),
                user_id
            );
            Ok((StatusCode::OK, Json(operations)))
        }
        Err(error) => {
            warn!(
                "Error retrieving operations for user {}: {}",
                user_id, error
            );
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "Failed to retrieve user operations".to_string(),
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
    fn test_internal_error() {
        let error = ErrorResponse {
            error: "INTERNAL_ERROR".to_string(),
            message: "Failed to retrieve user operations".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error.status_code, 500);
    }
}
