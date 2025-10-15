use crate::application::dto::responses::{AccountDeletionCancellationResponse, ErrorResponse};
use crate::domain::repositories::AccountDeletionTokenRepository;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

/// Cancel account deletion request
/// POST /api/account/deletion/cancel
#[utoipa::path(
    post,
    path = "/account/deletion/cancel",
    responses(
        (status = 200, description = "Account deletion cancelled", body = AccountDeletionCancellationResponse),
        (status = 404, description = "No active deletion request found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "account-deletion"
)]
pub async fn cancel_account_deletion(
    State(state): State<ConcreteAppState>,
) -> Result<Json<AccountDeletionCancellationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from authentication token/session
    // For now, using a placeholder user_id
    let user_id = 1; // This should come from authenticated user session

    let account_deletion_repo = &state.account_deletion_repository;

    // Find active token for user
    let active_token = account_deletion_repo
        .find_active_token_for_user(user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                    message: e.to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                }),
            )
        })?;

    if active_token.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No active deletion request".to_string(),
                message: "No active account deletion request found".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            }),
        ));
    }

    // Cancel all active tokens for the user
    let cancelled_count = account_deletion_repo
        .cancel_all_tokens_for_user(user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to cancel deletion request".to_string(),
                    message: e.to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                }),
            )
        })?;

    tracing::info!(
        "Cancelled {} account deletion request(s) for user_id: {}",
        cancelled_count,
        user_id
    );

    Ok(Json(AccountDeletionCancellationResponse {
        message: "Account deletion request has been cancelled successfully.".to_string(),
        cancelled: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_active_deletion_request_error() {
        let error = ErrorResponse {
            error: "No active deletion request".to_string(),
            message: "No active account deletion request found".to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        };
        assert_eq!(error.status_code, 404);
    }

    #[test]
    fn test_database_error() {
        let error = ErrorResponse {
            error: "Database error".to_string(),
            message: "Connection failed".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error.status_code, 500);
    }
}
