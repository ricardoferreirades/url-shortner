use super::ConcreteAppState;
use crate::application::dto::responses::ErrorResponse;
use crate::domain::repositories::password_reset_repository::PasswordResetRepository;
use crate::domain::repositories::user_repository::UserRepository;
use crate::domain::services::{PasswordResetService, PasswordResetError, TokenValidationService};
use crate::infrastructure::email::{EmailSender, EmailMessage};
use crate::presentation::handlers::app_state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request DTO for password reset request
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RequestPasswordResetRequest {
    pub email: String,
}

/// Response DTO for password reset request
#[derive(Debug, Serialize, ToSchema)]
pub struct RequestPasswordResetResponse {
    pub message: String,
    pub email: String,
}

/// Request DTO for password reset confirmation
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

/// Response DTO for password reset confirmation
#[derive(Debug, Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    pub message: String,
    pub success: bool,
}

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
    state.password_reset_rate_limiter
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
                            message: "Too many password reset requests. Please try again later.".to_string(),
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
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
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
        tracing::info!("Password reset token for {}: {}", reset_request.email, reset_request.token);
    }

    Ok(Json(RequestPasswordResetResponse {
        message: "If the email exists in our system, a password reset link has been sent.".to_string(),
        email: request.email,
    }))
}

/// Reset password with token
/// POST /api/auth/password-reset/confirm
#[utoipa::path(
    post,
    path = "/auth/password-reset/confirm",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = ResetPasswordResponse),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "password-reset"
)]
pub async fn reset_password(
    State(state): State<ConcreteAppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate password strength (basic validation)
    if request.new_password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Validation error".to_string(),
                message: "Password must be at least 8 characters long".to_string(),
                status_code: 400,
            }),
        ));
    }

    // Create password reset service
    let password_reset_service = PasswordResetService::new_default(
        state.password_reset_repository.clone(),
        state.user_repository.clone(),
    );

    // Reset password
    password_reset_service
        .reset_password(&request.token, &request.new_password)
        .await
        .map_err(|e| {
            let (status, message) = match e {
                PasswordResetError::InvalidToken => {
                    (StatusCode::BAD_REQUEST, "Invalid password reset token".to_string())
                }
                PasswordResetError::TokenExpired => {
                    (StatusCode::BAD_REQUEST, "Password reset token has expired".to_string())
                }
                PasswordResetError::TokenAlreadyUsed => {
                    (StatusCode::BAD_REQUEST, "Password reset token has already been used".to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            };
            (
                status,
                Json(ErrorResponse {
                    error: "Password reset error".to_string(),
                    message,
                    status_code: status.as_u16(),
                }),
            )
        })?;

    Ok(Json(ResetPasswordResponse {
        message: "Password has been reset successfully".to_string(),
        success: true,
    }))
}

/// Validate password reset token
/// GET /api/auth/password-reset/validate/{token}
#[utoipa::path(
    get,
    path = "/auth/password-reset/validate/{token}",
    responses(
        (status = 200, description = "Token is valid", body = serde_json::Value),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("token" = String, Path, description = "Password reset token")
    ),
    tag = "password-reset"
)]
pub async fn validate_reset_token(
    State(state): State<ConcreteAppState>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Create validation service
    let validation_service = TokenValidationService::new_default();

    // Validate token format first
    validation_service.validate_token_format(&token).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Token format error".to_string(),
                message: e.to_string(),
                status_code: 400,
            }),
        )
    })?;

    // Create password reset service
    let password_reset_service = PasswordResetService::new_default(
        state.password_reset_repository.clone(),
        state.user_repository.clone(),
    );

    // Validate token
    let reset_token = password_reset_service
        .validate_token(&token)
        .await
        .map_err(|e| {
            let (status, message) = match e {
                PasswordResetError::InvalidToken => {
                    (StatusCode::BAD_REQUEST, "Invalid password reset token".to_string())
                }
                PasswordResetError::TokenExpired => {
                    (StatusCode::BAD_REQUEST, "Password reset token has expired".to_string())
                }
                PasswordResetError::TokenAlreadyUsed => {
                    (StatusCode::BAD_REQUEST, "Password reset token has already been used".to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            };
            (
                status,
                Json(ErrorResponse {
                    error: "Token validation error".to_string(),
                    message,
                    status_code: status.as_u16(),
                }),
            )
        })?;

    // Get comprehensive validation result
    let validation_result = validation_service.validate(&token, &reset_token).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Validation error".to_string(),
                message: e.to_string(),
                status_code: 400,
            }),
        )
    })?;

    // Check if token will expire soon
    let will_expire_soon = validation_service.will_expire_soon(&reset_token, 1);

    Ok(Json(serde_json::json!({
        "valid": validation_result.is_valid,
        "message": "Token is valid",
        "expires_at": reset_token.expires_at,
        "time_until_expiration_hours": validation_result.time_until_expiration.map(|d| d.num_hours()),
        "will_expire_soon": will_expire_soon,
        "token_strength_score": validation_service.get_token_strength_score(&token),
    })))
}
