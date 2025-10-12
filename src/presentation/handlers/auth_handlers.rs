use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::domain::repositories::user_repository::UserRepository;
use crate::domain::services::AuthServiceError;
use crate::presentation::handlers::app_state::AppState;

/// Request DTO for user registration
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Request DTO for user login
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response DTO for authentication
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

/// Response DTO for user information
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

/// Error response DTO
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

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
pub async fn register_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: crate::domain::repositories::UrlRepository + Send + Sync + Clone,
    U: UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,
{
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
pub async fn login_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: crate::domain::repositories::UrlRepository + Send + Sync + Clone,
    U: UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,
{
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
