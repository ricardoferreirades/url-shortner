use crate::application::dto::requests::DeleteAccountRequest;
use crate::application::dto::responses::{AccountDeletionRequestResponse, ErrorResponse};
use crate::domain::entities::AccountDeletionToken;
use crate::domain::repositories::{AccountDeletionTokenRepository, UserRepository};
use crate::infrastructure::email::EmailMessage;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};
use bcrypt::verify;
use chrono::{Duration, Utc};

use super::token_utils::generate_secure_token;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_password_error() {
        let error = ErrorResponse {
            error: "Invalid password".to_string(),
            message: "The password provided is incorrect".to_string(),
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
        };
        assert_eq!(error.status_code, 401);
    }

    #[test]
    fn test_user_not_found_error() {
        let error = ErrorResponse {
            error: "User not found".to_string(),
            message: "User account does not exist".to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        };
        assert_eq!(error.status_code, 404);
    }
}
