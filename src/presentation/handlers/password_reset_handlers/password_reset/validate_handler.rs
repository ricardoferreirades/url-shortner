use crate::application::dto::responses::ErrorResponse;
use crate::domain::services::{PasswordResetError, PasswordResetService, TokenValidationService};
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

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
    validation_service
        .validate_token_format(&token)
        .map_err(|e| {
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
                    error: "Token validation error".to_string(),
                    message,
                    status_code: status.as_u16(),
                }),
            )
        })?;

    // Get comprehensive validation result
    let validation_result = validation_service
        .validate(&token, &reset_token)
        .map_err(|e| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_token_error() {
        let error = ErrorResponse {
            error: "Token validation error".to_string(),
            message: "Invalid password reset token".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.status_code, 400);
    }

    #[test]
    fn test_token_expired_error() {
        let error = ErrorResponse {
            error: "Token validation error".to_string(),
            message: "Password reset token has expired".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert!(error.message.contains("expired"));
    }
}
