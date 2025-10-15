use super::dtos::{
    convert_data_privacy_response, convert_privacy_request, convert_privacy_response,
    FieldPrivacySettingsResponse, PrivacySettingsResponse, UpdatePrivacyRequest,
};
use crate::application::dto::responses::ErrorResponse;
use crate::domain::repositories::UserRepository;
use crate::domain::services::PrivacyService;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let error = ErrorResponse {
            error: "Validation error".to_string(),
            message: "Invalid privacy setting".to_string(),
            status_code: 400,
        };
        assert_eq!(error.status_code, 400);
    }

    #[test]
    fn test_database_error() {
        let error = ErrorResponse {
            error: "Database error".to_string(),
            message: "Failed to update privacy settings".to_string(),
            status_code: 500,
        };
        assert_eq!(error.status_code, 500);
    }
}
