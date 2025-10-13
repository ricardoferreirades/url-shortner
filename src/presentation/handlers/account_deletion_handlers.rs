use super::ConcreteAppState;
use crate::application::dto::requests::{ConfirmAccountDeletionRequest, DeleteAccountRequest};
use crate::application::dto::responses::{
    AccountDeletionCancellationResponse, AccountDeletionConfirmationResponse,
    AccountDeletionRequestResponse, ErrorResponse,
};
use crate::domain::entities::AccountDeletionToken;
use crate::domain::repositories::{AccountDeletionTokenRepository, UserRepository};
use crate::domain::services::AnonymizationService;
use crate::infrastructure::email::EmailMessage;
use axum::{extract::State, http::StatusCode, response::Json};
use bcrypt::verify;
use chrono::{Duration, Utc};
use rand::Rng;

/// Request account deletion (send confirmation email)
/// POST /api/account/deletion/request
#[utoipa::path(
    post,
    path = "/account/deletion/request",
    request_body = DeleteAccountRequest,
    responses(
        (status = 200, description = "Account deletion confirmation email sent", body = AccountDeletionRequestResponse),
        (status = 401, description = "Invalid password", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "account-deletion"
)]
pub async fn request_account_deletion(
    State(state): State<ConcreteAppState>,
    Json(request): Json<DeleteAccountRequest>,
) -> Result<Json<AccountDeletionRequestResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from authentication token/session
    // For now, using a placeholder user_id
    let user_id = 1; // This should come from authenticated user session

    // Verify user exists and password is correct
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

    // Verify password
    let is_valid = verify(&request.password, &user.password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Authentication error".to_string(),
                message: e.to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            }),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid password".to_string(),
                message: "The password provided is incorrect".to_string(),
                status_code: StatusCode::UNAUTHORIZED.as_u16(),
            }),
        ));
    }

    // Cancel any existing active tokens for this user
    let account_deletion_repo = &state.account_deletion_repository;
    account_deletion_repo
        .cancel_all_tokens_for_user(user_id)
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

    // Generate secure random token
    let token = generate_secure_token();
    let now = Utc::now();
    let expires_at = now + Duration::hours(24); // Token valid for 24 hours

    // Create account deletion token
    let deletion_token = AccountDeletionToken::new(0, user_id, token.clone(), now, expires_at);

    let created_token = account_deletion_repo
        .create_token(deletion_token)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create deletion token".to_string(),
                    message: e.to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                }),
            )
        })?;

    // Send account deletion confirmation email
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let confirmation_link = format!("{}/account/deletion/confirm?token={}", base_url, token);

    let email_message = EmailMessage::account_deletion_confirmation(
        user.email.clone(),
        confirmation_link,
        24, // 24 hours expiration
    );

    // Send email (if email sender is configured)
    if let Some(email_sender) = state.email_sender.as_ref() {
        if let Err(e) = email_sender.send_email(email_message).await {
            tracing::error!("Failed to send account deletion confirmation email: {}", e);
            // Don't fail the request if email sending fails
            // Log the error and continue
        }
    } else {
        tracing::warn!("Email sender not configured, account deletion email not sent");
        tracing::info!("Account deletion token for user {}: {}", user_id, token);
    }

    Ok(Json(AccountDeletionRequestResponse {
        message: "Account deletion confirmation email has been sent. Please check your email to confirm this action.".to_string(),
        expires_at: created_token.expires_at.to_rfc3339(),
    }))
}

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

/// Generate a secure random token for account deletion
fn generate_secure_token() -> String {
    let mut rng = rand::thread_rng();
    let token: String = (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            
            match idx {
                0..=9 => (b'0' + idx) as char,
                10..=35 => (b'a' + idx - 10) as char,
                36..=61 => (b'A' + idx - 36) as char,
                _ => unreachable!(),
            }
        })
        .collect();
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_secure_token_uniqueness() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();
        assert_ne!(token1, token2);
    }
}
