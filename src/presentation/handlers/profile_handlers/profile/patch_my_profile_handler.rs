use super::utils::{convert_privacy_request, user_to_profile_response};
use crate::application::dto::{
    requests::UpdateProfileRequest,
    responses::{ErrorResponse, UserProfileResponse},
};
use crate::domain::entities::ProfilePrivacy;
use crate::domain::repositories::UserRepository;
use crate::domain::services::ProfileValidationService;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

/// Partially update current user's profile
/// PATCH /api/profile
#[utoipa::path(
    patch,
    path = "/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserProfileResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "profile"
)]
pub async fn patch_my_profile(
    State(state): State<ConcreteAppState>,
    Json(request): Json<UpdateProfileRequest>,
    // In a real implementation, you would extract user from JWT token
    // For now, we'll use a placeholder user_id
) -> Result<Json<UserProfileResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    // Convert privacy request to domain enum
    let privacy = request
        .privacy
        .map(convert_privacy_request)
        .unwrap_or(ProfilePrivacy::Public);

    // Validate and sanitize profile data
    let validation_service = ProfileValidationService::new();
    let validated_data = validation_service
        .validate_profile_data(
            request.first_name,
            request.last_name,
            request.bio,
            request.avatar_url,
            request.website,
            request.location,
            privacy,
        )
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

    match state
        .user_repository
        .update_profile(
            user_id,
            validated_data.first_name.as_deref(),
            validated_data.last_name.as_deref(),
            validated_data.bio.as_deref(),
            validated_data.avatar_url.as_deref(),
            validated_data.website.as_deref(),
            validated_data.location.as_deref(),
            Some(validated_data.privacy),
        )
        .await
    {
        Ok(user) => Ok(Json(user_to_profile_response(user))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
                message: e.to_string(),
                status_code: 500,
            }),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let error = ErrorResponse {
            error: "Validation error".to_string(),
            message: "Invalid profile data".to_string(),
            status_code: 400,
        };
        assert_eq!(error.status_code, 400);
    }

    #[test]
    fn test_database_error() {
        let error = ErrorResponse {
            error: "Database error".to_string(),
            message: "Failed to update profile".to_string(),
            status_code: 500,
        };
        assert_eq!(error.status_code, 500);
    }
}
