use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response DTO for successful URL shortening
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShortenUrlResponse {
    pub short_url: String,
    pub original_url: String,
    pub short_code: String,
    pub created_at: String,
}

/// Response DTO for URL information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UrlInfoResponse {
    pub id: i32,
    pub short_code: String,
    pub original_url: String,
    pub short_url: String,
    pub created_at: String,
    pub click_count: Option<i64>,
}

/// Response DTO for user URLs list
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserUrlsResponse {
    pub urls: Vec<UrlInfoResponse>,
    pub total_count: i64,
}

/// Response DTO for authentication
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

/// Response DTO for user information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub created_at: String,
}

/// Generic error response DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

/// Success response DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SuccessResponse {
    pub message: String,
    pub status_code: u16,
}
