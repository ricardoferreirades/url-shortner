use super::dtos::{AuthResponse, ErrorResponse, LoginRequest, UserResponse};
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, Json};
use tracing::{info, warn};

/// Handler for user login
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    ),
    tag = "authentication"
)]
pub async fn login_handler(
    State(app_state): State<ConcreteAppState>,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("Received login request for username: {}", request.username);

    match app_state
        .auth_service
        .login(&request.username, &request.password)
        .await
    {
        Ok(token) => {
            // Get user details for response
            match app_state.auth_service.verify_token(&token).await {
                Ok(user) => {
                    info!("Successfully logged in user: {}", user.username);
                    let response = AuthResponse {
                        token,
                        user: UserResponse {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            created_at: user.created_at.to_rfc3339(),
                        },
                    };
                    Ok((StatusCode::OK, Json(response)))
                }
                Err(e) => {
                    warn!("Failed to verify token after login: {}", e);
                    let error_response = ErrorResponse {
                        error: "TOKEN_VERIFICATION_FAILED".to_string(),
                        message: "Login successful but failed to verify token".to_string(),
                        status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    };
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(error) => {
            warn!("Failed to login user: {}", error);
            let error_response = ErrorResponse {
                error: "INVALID_CREDENTIALS".to_string(),
                message: "Invalid username or password".to_string(),
                status_code: StatusCode::UNAUTHORIZED.as_u16(),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_deserialize() {
        let json = r#"{"username":"testuser","password":"testpass"}"#;
        let request: Result<LoginRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
    }

    #[test]
    fn test_error_response_format() {
        let error = ErrorResponse {
            error: "INVALID_CREDENTIALS".to_string(),
            message: "Invalid username or password".to_string(),
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
        };
        assert_eq!(error.error, "INVALID_CREDENTIALS");
        assert_eq!(error.status_code, 401);
    }

    #[test]
    fn test_user_response_serialization() {
        use chrono::Utc;
        let user_response = UserResponse {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        let json = serde_json::to_string(&user_response);
        assert!(json.is_ok());
    }

    #[test]
    fn test_auth_response_structure() {
        use chrono::Utc;
        let auth_response = AuthResponse {
            token: "test_token".to_string(),
            user: UserResponse {
                id: 1,
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                created_at: Utc::now().to_rfc3339(),
            },
        };
        assert_eq!(auth_response.token, "test_token");
        assert_eq!(auth_response.user.username, "testuser");
    }

    #[test]
    fn test_invalid_credentials_error_response() {
        let error_response = ErrorResponse {
            error: "INVALID_CREDENTIALS".to_string(),
            message: "Invalid username or password".to_string(),
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
        };
        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("INVALID_CREDENTIALS"));
        assert!(json.contains("Invalid username or password"));
    }

    #[test]
    fn test_token_verification_failed_error() {
        let error_response = ErrorResponse {
            error: "TOKEN_VERIFICATION_FAILED".to_string(),
            message: "Login successful but failed to verify token".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error_response.status_code, 500);
    }
}
