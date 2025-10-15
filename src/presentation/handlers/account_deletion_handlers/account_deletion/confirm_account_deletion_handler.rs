use crate::application::dto::requests::ConfirmAccountDeletionRequest;
use crate::application::dto::responses::{AccountDeletionConfirmationResponse, ErrorResponse};
use crate::domain::repositories::{AccountDeletionTokenRepository, UserRepository};
use crate::domain::services::AnonymizationService;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

/// Confirm account deletion with token
/// POST /api/account/deletion/confirm
#[utoipa::path(
    post,
    path = "/account/deletion/confirm",
    request_body = ConfirmAccountDeletionRequest,
    responses(
        (status = 200, description = "Account deleted successfully", body = AccountDeletionConfirmationResponse),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 404, description = "Token not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "account-deletion"
)]
pub async fn confirm_account_deletion(
    State(state): State<ConcreteAppState>,
    Json(request): Json<ConfirmAccountDeletionRequest>,
) -> Result<Json<AccountDeletionConfirmationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account_deletion_repo = &state.account_deletion_repository;

    // Find token
    let mut deletion_token = account_deletion_repo
        .find_by_token(&request.token)
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
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Token not found".to_string(),
                    message: "Invalid account deletion token".to_string(),
                    status_code: StatusCode::NOT_FOUND.as_u16(),
                }),
            )
        })?;

    // Validate token
    if !deletion_token.is_valid() {
        let error_msg = if deletion_token.is_expired() {
            "Account deletion token has expired"
        } else if deletion_token.is_confirmed {
            "Account deletion has already been confirmed"
        } else if deletion_token.is_cancelled {
            "Account deletion request has been cancelled"
        } else {
            "Invalid account deletion token"
        };

        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid token".to_string(),
                message: error_msg.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            }),
        ));
    }

    let user_id = deletion_token.user_id;

    // Get user before deletion
    let user = state
        .user_repository
        .find_by_id(user_id)
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
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                    message: "User account does not exist".to_string(),
                    status_code: StatusCode::NOT_FOUND.as_u16(),
                }),
            )
        })?;

    // Anonymize user data
    let anonymization_service = AnonymizationService::new();
    let anonymized_data = anonymization_service.anonymize_user_data(&user);

    // Anonymize account using repository method
    state
        .user_repository
        .anonymize_account(
            user_id,
            &anonymized_data.username,
            &anonymized_data.email,
            &anonymized_data.password_hash,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to anonymize user data".to_string(),
                    message: e.to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                }),
            )
        })?;

    // Mark token as confirmed
    deletion_token.mark_as_confirmed();
    account_deletion_repo
        .update_token(deletion_token)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update deletion token".to_string(),
                    message: e.to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                }),
            )
        })?;

    tracing::info!("Account deleted successfully for user_id: {}", user_id);

    Ok(Json(AccountDeletionConfirmationResponse {
        message:
            "Your account has been successfully deleted. All personal data has been anonymized."
                .to_string(),
        deleted: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_not_found_error() {
        let error = ErrorResponse {
            error: "Token not found".to_string(),
            message: "Invalid account deletion token".to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        };
        assert_eq!(error.status_code, 404);
    }

    #[test]
    fn test_invalid_token_error() {
        let error = ErrorResponse {
            error: "Invalid token".to_string(),
            message: "Account deletion token has expired".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.error, "Invalid token");
    }
}
