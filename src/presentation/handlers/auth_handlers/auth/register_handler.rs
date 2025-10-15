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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_deserialize() {
        let json =
            r#"{"username":"newuser","email":"test@example.com","password":"securepass123"}"#;
        let request: Result<RegisterRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.username, "newuser");
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "securepass123");
    }

    #[test]
    fn test_username_exists_error() {
        let error_response = ErrorResponse {
            error: "USERNAME_EXISTS".to_string(),
            message: "Username already exists".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error_response.error, "USERNAME_EXISTS");
        assert_eq!(error_response.status_code, 400);
    }

    #[test]
    fn test_email_exists_error() {
        let error_response = ErrorResponse {
            error: "EMAIL_EXISTS".to_string(),
            message: "Email already exists".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error_response.error, "EMAIL_EXISTS");
        assert_eq!(error_response.status_code, 400);
    }

    #[test]
    fn test_invalid_input_error() {
        let error_response = ErrorResponse {
            error: "INVALID_INPUT".to_string(),
            message: "Invalid email format".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error_response.error, "INVALID_INPUT");
    }

    #[test]
    fn test_token_generation_failed_error() {
        let error_response = ErrorResponse {
            error: "TOKEN_GENERATION_FAILED".to_string(),
            message: "User registered but failed to generate token".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error_response.status_code, 500);
    }

    #[test]
    fn test_auth_response_serialization() {
        use chrono::Utc;
        let auth_response = AuthResponse {
            token: "jwt_token_here".to_string(),
            user: UserResponse {
                id: 1,
                username: "newuser".to_string(),
                email: "test@example.com".to_string(),
                created_at: Utc::now().to_rfc3339(),
            },
        };
        let json = serde_json::to_string(&auth_response);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("newuser"));
    }
}
