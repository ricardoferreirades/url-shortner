use super::dtos::{RequestPasswordResetRequest, RequestPasswordResetResponse};
use crate::application::dto::responses::ErrorResponse;
use crate::domain::services::{PasswordResetError, PasswordResetService};
use crate::infrastructure::email::EmailMessage;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

/// Request password reset (send reset email)
/// POST /api/auth/password-reset/request
#[utoipa::path(
    post,
    path = "/auth/password-reset/request",
    request_body = RequestPasswordResetRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = RequestPasswordResetResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 429, description = "Too many requests", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "password-reset"
)]
pub async fn request_password_reset(
    State(state): State<ConcreteAppState>,
    Json(request): Json<RequestPasswordResetRequest>,
) -> Result<Json<RequestPasswordResetResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get client IP (in production, extract from headers)
    let client_ip = "127.0.0.1"; // Placeholder - should extract from request headers

    // Check rate limits
    state
        .password_reset_rate_limiter
        .check_all_limits(client_ip, &request.email)
        .await
        .map_err(|e| {
            let status = StatusCode::TOO_MANY_REQUESTS;
            (
                status,
                Json(ErrorResponse {
                    error: "Rate limit exceeded".to_string(),
                    message: e.to_string(),
                    status_code: status.as_u16(),
                }),
            )
        })?;

    // Create password reset service
    let password_reset_service = PasswordResetService::new_default(
        state.password_reset_repository.clone(),
        state.user_repository.clone(),
    );

    // Create reset request and generate token
    let reset_request = match password_reset_service
        .create_reset_request(&request.email)
        .await
    {
        Ok(reset_req) => reset_req,
        Err(e) => {
            match e {
                PasswordResetError::UserNotFound => {
                    // For security, don't reveal if user exists or not
                    // Return success anyway
                    return Ok(Json(RequestPasswordResetResponse {
                        message: "If the email exists in our system, a password reset link has been sent.".to_string(),
                        email: request.email.clone(),
                    }));
                }
                PasswordResetError::TooManyRequests => {
                    return Err((
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(ErrorResponse {
                            error: "Password reset error".to_string(),
                            message: "Too many password reset requests. Please try again later."
                                .to_string(),
                            status_code: StatusCode::TOO_MANY_REQUESTS.as_u16(),
                        }),
                    ));
                }
                _ => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Password reset error".to_string(),
                            message: e.to_string(),
                            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        }),
                    ));
                }
            }
        }
    };

    // Send password reset email
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let reset_link = format!("{}/reset-password?token={}", base_url, reset_request.token);

    let email_message = EmailMessage::password_reset(
        reset_request.email.clone(),
        reset_link,
        24, // 24 hours expiration
    );

    // Send email (if email sender is configured)
    if let Some(email_sender) = state.email_sender.as_ref() {
        if let Err(e) = email_sender.send_email(email_message).await {
            tracing::error!("Failed to send password reset email: {}", e);
            // Don't fail the request if email sending fails
            // Log the error and continue
        }
    } else {
        tracing::warn!("Email sender not configured, password reset email not sent");
        tracing::info!(
            "Password reset token for {}: {}",
            reset_request.email,
            reset_request.token
        );
    }

    Ok(Json(RequestPasswordResetResponse {
        message: "If the email exists in our system, a password reset link has been sent."
            .to_string(),
        email: request.email,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_error() {
        let error = ErrorResponse {
            error: "Rate limit exceeded".to_string(),
            message: "Too many requests".to_string(),
            status_code: StatusCode::TOO_MANY_REQUESTS.as_u16(),
        };
        assert_eq!(error.status_code, 429);
    }

    #[test]
    fn test_password_reset_error() {
        let error = ErrorResponse {
            error: "Password reset error".to_string(),
            message: "Too many password reset requests. Please try again later.".to_string(),
            status_code: StatusCode::TOO_MANY_REQUESTS.as_u16(),
        };
        assert_eq!(error.error, "Password reset error");
    }
}
