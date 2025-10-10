use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request DTO for shortening a URL
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ShortenUrlRequest {
    pub url: String,
    pub custom_short_code: Option<String>,
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request DTO for updating a URL
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUrlRequest {
    pub original_url: Option<String>,
    pub custom_short_code: Option<String>,
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
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

/// Request DTO for setting URL expiration
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SetExpirationRequest {
    pub expiration_date: chrono::DateTime<chrono::Utc>,
}

/// Request DTO for extending URL expiration
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtendExpirationRequest {
    pub additional_days: u32,
}

/// Request DTO for bulk URL shortening
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BulkShortenUrlsRequest {
    pub items: Vec<ShortenUrlRequest>,
}

/// Request DTO for batch URL operations
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BatchUrlOperationRequest {
    pub operation: BatchOperationType,
    pub url_ids: Vec<i32>,
    pub data: Option<BatchOperationData>,
}

/// Types of batch operations that can be performed
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub enum BatchOperationType {
    #[serde(rename = "deactivate")]
    Deactivate,
    #[serde(rename = "reactivate")]
    Reactivate,
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "update_status")]
    UpdateStatus,
    #[serde(rename = "update_expiration")]
    UpdateExpiration,
}

/// Data for batch operations
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BatchOperationData {
    pub status: Option<String>,
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request DTO for bulk URL status updates
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BulkStatusUpdateRequest {
    pub url_ids: Vec<i32>,
    pub status: String,
}

/// Request DTO for bulk URL expiration updates
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BulkExpirationUpdateRequest {
    pub url_ids: Vec<i32>,
    pub expiration_date: chrono::DateTime<chrono::Utc>,
}

/// Request DTO for bulk URL deletion
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BulkDeleteRequest {
    pub url_ids: Vec<i32>,
    pub force: Option<bool>,
}

/// Request DTO for updating user profile
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub privacy: Option<ProfilePrivacyRequest>,
}

/// Privacy settings for profile requests
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub enum ProfilePrivacyRequest {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "friends_only")]
    FriendsOnly,
}

/// Request DTO for account deletion
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteAccountRequest {
    /// User's current password for confirmation
    pub password: String,
}
