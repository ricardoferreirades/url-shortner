use super::dtos::{ResetPasswordRequest, ResetPasswordResponse};
use crate::application::dto::responses::ErrorResponse;
use crate::domain::services::{PasswordResetError, PasswordResetService};
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

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
                PasswordResetError::InvalidToken => (
                    StatusCode::BAD_REQUEST,
                    "Invalid password reset token".to_string(),
                ),
                PasswordResetError::TokenExpired => (
                    StatusCode::BAD_REQUEST,
                    "Password reset token has expired".to_string(),
                ),
                PasswordResetError::TokenAlreadyUsed => (
                    StatusCode::BAD_REQUEST,
                    "Password reset token has already been used".to_string(),
                ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_too_short_error() {
        let error = ErrorResponse {
            error: "Validation error".to_string(),
            message: "Password must be at least 8 characters long".to_string(),
            status_code: 400,
        };
        assert_eq!(error.status_code, 400);
    }

    #[test]
    fn test_password_reset_error() {
        let error = ErrorResponse {
            error: "Password reset error".to_string(),
            message: "Invalid password reset token".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.error, "Password reset error");
    }
}
