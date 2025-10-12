use super::ConcreteAppState;
use crate::application::dto::responses::ErrorResponse;
use crate::domain::entities::ProfilePrivacy;
use crate::domain::repositories::UserRepository;
use crate::domain::services::{DataPrivacyLevel, PrivacyService};
use axum::{extract::State, http::StatusCode, response::Json};
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
fn convert_privacy_request(privacy: ProfilePrivacyRequest) -> ProfilePrivacy {
    match privacy {
        ProfilePrivacyRequest::Public => ProfilePrivacy::Public,
        ProfilePrivacyRequest::Private => ProfilePrivacy::Private,
        ProfilePrivacyRequest::FriendsOnly => ProfilePrivacy::FriendsOnly,
    }
}

/// Convert ProfilePrivacy to ProfilePrivacyResponse
fn convert_privacy_response(privacy: ProfilePrivacy) -> ProfilePrivacyResponse {
    match privacy {
        ProfilePrivacy::Public => ProfilePrivacyResponse::Public,
        ProfilePrivacy::Private => ProfilePrivacyResponse::Private,
        ProfilePrivacy::FriendsOnly => ProfilePrivacyResponse::FriendsOnly,
    }
}

/// Convert DataPrivacyLevelRequest to DataPrivacyLevel
#[allow(dead_code)]
fn convert_data_privacy_request(level: DataPrivacyLevelRequest) -> DataPrivacyLevel {
    match level {
        DataPrivacyLevelRequest::Public => DataPrivacyLevel::Public,
        DataPrivacyLevelRequest::Private => DataPrivacyLevel::Private,
        DataPrivacyLevelRequest::FriendsOnly => DataPrivacyLevel::FriendsOnly,
    }
}

/// Convert DataPrivacyLevel to DataPrivacyLevelResponse
fn convert_data_privacy_response(level: DataPrivacyLevel) -> DataPrivacyLevelResponse {
    match level {
        DataPrivacyLevel::Public => DataPrivacyLevelResponse::Public,
        DataPrivacyLevel::Private => DataPrivacyLevelResponse::Private,
        DataPrivacyLevel::FriendsOnly => DataPrivacyLevelResponse::FriendsOnly,
    }
}

/// Get current user's privacy settings
/// GET /api/profile/privacy
#[utoipa::path(
    get,
    path = "/profile/privacy",
    responses(
        (status = 200, description = "Privacy settings retrieved successfully", body = PrivacySettingsResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "privacy"
)]
pub async fn get_privacy_settings(
    State(state): State<ConcreteAppState>,
    // In a real implementation, you would extract user from JWT token
    // For now, we'll use a placeholder user_id
) -> Result<Json<PrivacySettingsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    let user = match state.user_repository.get_profile(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                    message: "User profile not found".to_string(),
                    status_code: 404,
                }),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                    message: e.to_string(),
                    status_code: 500,
                }),
            ));
        }
    };

    let privacy_service = PrivacyService::new();
    let field_settings = PrivacyService::get_default_privacy_settings();

    Ok(Json(PrivacySettingsResponse {
        profile_privacy: convert_privacy_response(user.privacy.clone()),
        field_settings: FieldPrivacySettingsResponse {
            first_name: convert_data_privacy_response(field_settings.first_name),
            last_name: convert_data_privacy_response(field_settings.last_name),
            bio: convert_data_privacy_response(field_settings.bio),
            avatar_url: convert_data_privacy_response(field_settings.avatar_url),
            website: convert_data_privacy_response(field_settings.website),
            location: convert_data_privacy_response(field_settings.location),
            email: convert_data_privacy_response(field_settings.email),
        },
        is_searchable: privacy_service.is_profile_searchable(&user.privacy),
        privacy_description: privacy_service
            .get_privacy_description(&user.privacy)
            .to_string(),
    }))
}

/// Update privacy settings
/// PUT /api/profile/privacy
#[utoipa::path(
    put,
    path = "/profile/privacy",
    request_body = UpdatePrivacyRequest,
    responses(
        (status = 200, description = "Privacy settings updated successfully", body = PrivacySettingsResponse),
        (status = 400, description = "Invalid privacy settings", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "privacy"
)]
pub async fn update_privacy_settings(
    State(state): State<ConcreteAppState>,
    Json(request): Json<UpdatePrivacyRequest>,
    // In a real implementation, you would extract user from JWT token
    // For now, we'll use a placeholder user_id
) -> Result<Json<PrivacySettingsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    let privacy_service = PrivacyService::new();

    // Update profile privacy if provided
    if let Some(profile_privacy) = request.profile_privacy {
        let privacy = convert_privacy_request(profile_privacy);
        privacy_service
            .validate_privacy_setting(&privacy)
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Validation error".to_string(),
                        message: e.to_string(),
                        status_code: 400,
                    }),
                )
            })?;

        // Update user's privacy setting
        match state
            .user_repository
            .update_profile(
                user_id,
                None, // first_name
                None, // last_name
                None, // bio
                None, // avatar_url
                None, // website
                None, // location
                Some(privacy),
            )
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Database error".to_string(),
                        message: e.to_string(),
                        status_code: 500,
                    }),
                ));
            }
        }
    }

    // Get updated user profile
    let user = match state.user_repository.get_profile(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                    message: "User profile not found".to_string(),
                    status_code: 404,
                }),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                    message: e.to_string(),
                    status_code: 500,
                }),
            ));
        }
    };

    let field_settings = PrivacyService::get_default_privacy_settings();

    Ok(Json(PrivacySettingsResponse {
        profile_privacy: convert_privacy_response(user.privacy.clone()),
        field_settings: FieldPrivacySettingsResponse {
            first_name: convert_data_privacy_response(field_settings.first_name),
            last_name: convert_data_privacy_response(field_settings.last_name),
            bio: convert_data_privacy_response(field_settings.bio),
            avatar_url: convert_data_privacy_response(field_settings.avatar_url),
            website: convert_data_privacy_response(field_settings.website),
            location: convert_data_privacy_response(field_settings.location),
            email: convert_data_privacy_response(field_settings.email),
        },
        is_searchable: privacy_service.is_profile_searchable(&user.privacy),
        privacy_description: privacy_service
            .get_privacy_description(&user.privacy)
            .to_string(),
    }))
}

/// Get recommended privacy settings
/// GET /api/profile/privacy/recommendations
#[utoipa::path(
    get,
    path = "/profile/privacy/recommendations",
    responses(
        (status = 200, description = "Recommended privacy settings retrieved successfully", body = PrivacySettingsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "privacy"
)]
pub async fn get_privacy_recommendations(// In a real implementation, you would extract user preferences from JWT token or request
) -> Result<Json<PrivacySettingsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let privacy_service = PrivacyService::new();

    // For demo purposes, return business account recommendations
    // In a real implementation, you would determine this based on user preferences
    let field_settings = privacy_service.get_recommended_privacy_settings(
        false, // is_public_figure
        true,  // is_business_account
        false, // is_personal_account
    );

    Ok(Json(PrivacySettingsResponse {
        profile_privacy: ProfilePrivacyResponse::Public,
        field_settings: FieldPrivacySettingsResponse {
            first_name: convert_data_privacy_response(field_settings.first_name),
            last_name: convert_data_privacy_response(field_settings.last_name),
            bio: convert_data_privacy_response(field_settings.bio),
            avatar_url: convert_data_privacy_response(field_settings.avatar_url),
            website: convert_data_privacy_response(field_settings.website),
            location: convert_data_privacy_response(field_settings.location),
            email: convert_data_privacy_response(field_settings.email),
        },
        is_searchable: true,
        privacy_description: "Recommended settings for business accounts".to_string(),
    }))
}
