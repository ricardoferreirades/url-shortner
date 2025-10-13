use super::dtos::{AuthResponse, ErrorResponse, RegisterRequest, UserResponse};
use crate::domain::services::AuthServiceError;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, Json};
use tracing::{info, warn};

/// Handler for user registration
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid input or user already exists", body = ErrorResponse)
    ),
    tag = "authentication"
)]
pub async fn register_handler(
    State(app_state): State<ConcreteAppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Received registration request for username: {}",
        request.username
    );

    match app_state
        .auth_service
        .register(&request.username, &request.email, &request.password)
        .await
    {
        Ok(user) => {
            info!("Successfully registered user: {}", user.username);

            // Generate token for the newly registered user
            match app_state
                .auth_service
                .login(&user.username, &request.password)
                .await
            {
                Ok(token) => {
                    let response = AuthResponse {
                        token,
                        user: UserResponse {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            created_at: user.created_at.to_rfc3339(),
                        },
                    };
                    Ok((StatusCode::CREATED, Json(response)))
                }
                Err(e) => {
                    warn!("Failed to generate token for registered user: {}", e);
                    let error_response = ErrorResponse {
                        error: "TOKEN_GENERATION_FAILED".to_string(),
                        message: "User registered but failed to generate token".to_string(),
                        status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    };
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(error) => {
            warn!("Failed to register user: {}", error);
            let (error_code, error_message) = match error {
                AuthServiceError::UsernameAlreadyExists => {
                    ("USERNAME_EXISTS", "Username already exists".to_string())
                }
                AuthServiceError::EmailAlreadyExists => {
                    ("EMAIL_EXISTS", "Email already exists".to_string())
                }
                AuthServiceError::InvalidInput(msg) => ("INVALID_INPUT", msg),
                _ => ("REGISTRATION_FAILED", "Registration failed".to_string()),
            };

            let error_response = ErrorResponse {
                error: error_code.to_string(),
                message: error_message,
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}
