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

