use crate::domain::entities::ProfilePrivacy;
use crate::domain::services::DataPrivacyLevel;
use serde::{Deserialize, Serialize};

/// Request DTO for updating privacy settings
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePrivacyRequest {
    pub profile_privacy: Option<ProfilePrivacyRequest>,
    pub field_settings: Option<FieldPrivacySettingsRequest>,
}

/// Privacy settings for profile requests
#[derive(Debug, Deserialize, Serialize)]
pub enum ProfilePrivacyRequest {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "friends_only")]
    FriendsOnly,
}

/// Field privacy settings request
#[derive(Debug, Deserialize, Serialize)]
pub struct FieldPrivacySettingsRequest {
    pub first_name: Option<DataPrivacyLevelRequest>,
    pub last_name: Option<DataPrivacyLevelRequest>,
    pub bio: Option<DataPrivacyLevelRequest>,
    pub avatar_url: Option<DataPrivacyLevelRequest>,
    pub website: Option<DataPrivacyLevelRequest>,
    pub location: Option<DataPrivacyLevelRequest>,
    pub email: Option<DataPrivacyLevelRequest>,
}

/// Data privacy level request
#[derive(Debug, Deserialize, Serialize)]
pub enum DataPrivacyLevelRequest {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "friends_only")]
    FriendsOnly,
}

/// Response DTO for privacy settings
#[derive(Debug, Serialize)]
pub struct PrivacySettingsResponse {
    pub profile_privacy: ProfilePrivacyResponse,
    pub field_settings: FieldPrivacySettingsResponse,
    pub is_searchable: bool,
    pub privacy_description: String,
}

/// Profile privacy response
#[derive(Debug, Serialize)]
pub enum ProfilePrivacyResponse {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "friends_only")]
    FriendsOnly,
}

/// Field privacy settings response
#[derive(Debug, Serialize)]
pub struct FieldPrivacySettingsResponse {
    pub first_name: DataPrivacyLevelResponse,
    pub last_name: DataPrivacyLevelResponse,
    pub bio: DataPrivacyLevelResponse,
    pub avatar_url: DataPrivacyLevelResponse,
    pub website: DataPrivacyLevelResponse,
    pub location: DataPrivacyLevelResponse,
    pub email: DataPrivacyLevelResponse,
}

/// Data privacy level response
#[derive(Debug, Serialize)]
pub enum DataPrivacyLevelResponse {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "friends_only")]
    FriendsOnly,
}

/// Convert ProfilePrivacyRequest to ProfilePrivacy
pub fn convert_privacy_request(privacy: ProfilePrivacyRequest) -> ProfilePrivacy {
    match privacy {
        ProfilePrivacyRequest::Public => ProfilePrivacy::Public,
        ProfilePrivacyRequest::Private => ProfilePrivacy::Private,
        ProfilePrivacyRequest::FriendsOnly => ProfilePrivacy::FriendsOnly,
    }
}

/// Convert ProfilePrivacy to ProfilePrivacyResponse
pub fn convert_privacy_response(privacy: ProfilePrivacy) -> ProfilePrivacyResponse {
    match privacy {
        ProfilePrivacy::Public => ProfilePrivacyResponse::Public,
        ProfilePrivacy::Private => ProfilePrivacyResponse::Private,
        ProfilePrivacy::FriendsOnly => ProfilePrivacyResponse::FriendsOnly,
    }
}

/// Convert DataPrivacyLevelRequest to DataPrivacyLevel
#[allow(dead_code)]
pub fn convert_data_privacy_request(level: DataPrivacyLevelRequest) -> DataPrivacyLevel {
    match level {
        DataPrivacyLevelRequest::Public => DataPrivacyLevel::Public,
        DataPrivacyLevelRequest::Private => DataPrivacyLevel::Private,
        DataPrivacyLevelRequest::FriendsOnly => DataPrivacyLevel::FriendsOnly,
    }
}

/// Convert DataPrivacyLevel to DataPrivacyLevelResponse
pub fn convert_data_privacy_response(level: DataPrivacyLevel) -> DataPrivacyLevelResponse {
    match level {
        DataPrivacyLevel::Public => DataPrivacyLevelResponse::Public,
        DataPrivacyLevel::Private => DataPrivacyLevelResponse::Private,
        DataPrivacyLevel::FriendsOnly => DataPrivacyLevelResponse::FriendsOnly,
    }
}
