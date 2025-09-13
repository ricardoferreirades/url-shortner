use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request DTO for shortening a URL
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ShortenUrlRequest {
    pub url: String,
    pub custom_short_code: Option<String>,
}

/// Request DTO for updating a URL
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUrlRequest {
    pub original_url: Option<String>,
    pub custom_short_code: Option<String>,
}

/// Request DTO for user authentication
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Request DTO for user registration
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}
