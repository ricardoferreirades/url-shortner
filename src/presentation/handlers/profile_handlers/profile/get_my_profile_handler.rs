use super::utils::user_to_profile_response;
use crate::application::dto::responses::{ErrorResponse, UserProfileResponse};
use crate::domain::repositories::UserRepository;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};

/// Get current user's profile
/// GET /api/profile
#[utoipa::path(
    get,
    path = "/profile",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = UserProfileResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "profile"
)]
pub async fn get_my_profile(
    State(state): State<ConcreteAppState>,
    // In a real implementation, you would extract user from JWT token
    // For now, we'll use a placeholder user_id
) -> Result<Json<UserProfileResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    match state.user_repository.get_profile(user_id).await {
        Ok(Some(user)) => Ok(Json(user_to_profile_response(user))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "User not found".to_string(),
                message: "User profile not found".to_string(),
                status_code: 404,
            }),
        )),
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
    fn test_user_not_found_error() {
        let error = ErrorResponse {
            error: "User not found".to_string(),
            message: "User profile not found".to_string(),
            status_code: 404,
        };
        assert_eq!(error.status_code, 404);
    }

    #[test]
    fn test_database_error_response() {
        let error = ErrorResponse {
            error: "Database error".to_string(),
            message: "Connection failed".to_string(),
            status_code: 500,
        };
        assert_eq!(error.status_code, 500);
    }
}
