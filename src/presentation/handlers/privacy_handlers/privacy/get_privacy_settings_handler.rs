use super::dtos::{
    convert_data_privacy_response, convert_privacy_response, FieldPrivacySettingsResponse,
    PrivacySettingsResponse,
};
use crate::application::dto::responses::ErrorResponse;
use crate::domain::repositories::UserRepository;
use crate::domain::services::PrivacyService;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

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
