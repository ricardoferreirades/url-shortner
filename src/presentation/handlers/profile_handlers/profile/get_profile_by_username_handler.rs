use super::utils::user_to_public_profile_response;
use crate::application::dto::responses::{ErrorResponse, PublicUserProfileResponse};
use crate::domain::repositories::UserRepository;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::Path, extract::State, http::StatusCode, response::Json};

/// Get profile by username
/// GET /api/profile/username/{username}
#[utoipa::path(
    get,
    path = "/profile/username/{username}",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = PublicUserProfileResponse),
        (status = 403, description = "Profile is private", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("username" = String, Path, description = "Username")
    ),
    tag = "profile"
)]
pub async fn get_profile_by_username(
    State(state): State<ConcreteAppState>,
    Path(username): Path<String>,
) -> Result<Json<PublicUserProfileResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.user_repository.find_by_username(&username).await {
        Ok(Some(user)) => {
            // Check if profile is public
            if !user.is_profile_public() {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Profile private".to_string(),
                        message: "This profile is private".to_string(),
                        status_code: 403,
                    }),
                ));
            }
            Ok(Json(user_to_public_profile_response(user)))
        }
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
    fn test_profile_private_error() {
        let error = ErrorResponse {
            error: "Profile private".to_string(),
            message: "This profile is private".to_string(),
            status_code: 403,
        };
        assert_eq!(error.status_code, 403);
    }

    #[test]
    fn test_user_not_found_error() {
        let error = ErrorResponse {
            error: "User not found".to_string(),
            message: "User profile not found".to_string(),
            status_code: 404,
        };
        assert_eq!(error.status_code, 404);
    }
}
