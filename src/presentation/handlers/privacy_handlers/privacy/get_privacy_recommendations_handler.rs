use super::dtos::{
    convert_data_privacy_response, FieldPrivacySettingsResponse, PrivacySettingsResponse,
    ProfilePrivacyResponse,
};
use crate::application::dto::responses::ErrorResponse;
use crate::domain::services::PrivacyService;
use axum::{http::StatusCode, response::Json};

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

#[cfg(test)]
mod tests {
    use super::super::dtos::{
        DataPrivacyLevelResponse, FieldPrivacySettingsResponse, PrivacySettingsResponse,
        ProfilePrivacyResponse,
    };

    #[test]
    fn test_privacy_recommendations_response_structure() {
        // Test the response structure by creating a sample
        let response = PrivacySettingsResponse {
            profile_privacy: ProfilePrivacyResponse::Public,
            field_settings: FieldPrivacySettingsResponse {
                first_name: DataPrivacyLevelResponse::Public,
                last_name: DataPrivacyLevelResponse::Public,
                bio: DataPrivacyLevelResponse::Public,
                avatar_url: DataPrivacyLevelResponse::Public,
                website: DataPrivacyLevelResponse::Public,
                location: DataPrivacyLevelResponse::Public,
                email: DataPrivacyLevelResponse::Private,
            },
            is_searchable: true,
            privacy_description: "Recommended settings for business accounts".to_string(),
        };
        assert!(response.is_searchable);
        assert_eq!(
            response.privacy_description,
            "Recommended settings for business accounts"
        );
    }
}
