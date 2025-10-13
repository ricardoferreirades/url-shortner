use serde::{Deserialize, Serialize};

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
