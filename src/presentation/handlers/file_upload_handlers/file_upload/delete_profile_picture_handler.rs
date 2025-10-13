use crate::application::dto::responses::ErrorResponse;
use crate::domain::repositories::UserRepository;
use crate::domain::services::FileUploadService;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::Value;

/// Delete profile picture
/// DELETE /api/profile/avatar
#[utoipa::path(
    delete,
    path = "/profile/avatar",
    responses(
        (status = 200, description = "Avatar deleted successfully", body = serde_json::Value),
        (status = 404, description = "No avatar found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "profile"
)]
pub async fn delete_profile_picture(
    State(state): State<ConcreteAppState>,
    // In a real implementation, you would extract user from JWT token
    // For now, we'll use a placeholder user_id
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    // Get current user profile
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

    // Check if user has an avatar
    let avatar_url = match user.avatar_url {
        Some(url) => url,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "No avatar".to_string(),
                    message: "User has no avatar to delete".to_string(),
                    status_code: 404,
                }),
            ));
        }
    };

    // Extract filename from URL
    let filename = avatar_url.split('/').next_back().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid avatar URL".to_string(),
                message: "Cannot extract filename from avatar URL".to_string(),
                status_code: 400,
            }),
        )
    })?;

    // Create file upload service
    let upload_service =
        FileUploadService::new_profile_picture_service("uploads/avatars".to_string());

    // Delete file from filesystem
    if let Err(e) = upload_service.delete_file(filename).await {
        // Log error but don't fail the request
        tracing::warn!("Failed to delete avatar file {}: {}", filename, e);
    }

    // Update user profile to remove avatar URL
    match state
        .user_repository
        .update_profile(
            user_id,
            None,     // first_name
            None,     // last_name
            None,     // bio
            Some(""), // avatar_url (empty string to clear it)
            None,     // website
            None,     // location
            None,     // privacy
        )
        .await
    {
        Ok(_) => Ok(Json(serde_json::json!({
            "message": "Avatar deleted successfully"
        }))),
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

