use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response DTO for successful URL shortening
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShortenUrlResponse {
    pub short_url: String,
    pub original_url: String,
    pub short_code: String,
    pub created_at: String,
    pub expiration_date: Option<String>,
}

/// Response DTO for URL information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UrlInfoResponse {
    pub id: i32,
    pub short_code: String,
    pub original_url: String,
    pub short_url: String,
    pub created_at: String,
    pub expiration_date: Option<String>,
    pub is_expired: bool,
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

/// Response DTO for user profile information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProfileResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub privacy: ProfilePrivacyResponse,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// Privacy settings for profile responses
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum ProfilePrivacyResponse {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "friends_only")]
    FriendsOnly,
}

/// Response DTO for public user profile (limited fields)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicUserProfileResponse {
    pub id: i32,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
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

/// Response DTO for expiration information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExpirationInfoResponse {
    pub expiration_date: Option<String>,
    pub is_expired: bool,
    pub expires_in_days: Option<i64>,
}

/// Response DTO for URLs expiring soon
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExpiringUrlsResponse {
    pub urls: Vec<UrlInfoResponse>,
    pub total_count: i64,
    pub warning_period_days: u32,
}

/// Response DTO for batch operation results
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchOperationResponse {
    pub operation: String,
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BatchOperationResult>,
}

/// Individual result for a batch operation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchOperationResult {
    pub url_id: i32,
    pub success: bool,
    pub error: Option<String>,
}

/// Response DTO for bulk operation progress
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BulkOperationProgress {
    pub operation_id: String,
    pub status: BulkOperationStatus,
    pub total_items: usize,
    pub processed_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub progress_percentage: f32,
}

/// Status of a bulk operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum BulkOperationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}
