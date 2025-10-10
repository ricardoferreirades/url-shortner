use super::ConcreteAppState;
use crate::application::dto::{
    requests::{UpdateProfileRequest, ProfilePrivacyRequest, DeleteAccountRequest},
    responses::{UserProfileResponse, PublicUserProfileResponse, ProfilePrivacyResponse, ErrorResponse},
};
use crate::domain::entities::{User, ProfilePrivacy};
use crate::domain::repositories::user_repository::UserRepository;
use crate::domain::services::ProfileValidationService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use bcrypt::verify;

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

/// Convert User entity to UserProfileResponse
fn user_to_profile_response(user: User) -> UserProfileResponse {
    let full_name = user.full_name();
    UserProfileResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        full_name,
        bio: user.bio,
        avatar_url: user.avatar_url,
        website: user.website,
        location: user.location,
        privacy: convert_privacy_response(user.privacy),
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.map(|dt| dt.to_rfc3339()),
    }
}

/// Convert User entity to PublicUserProfileResponse
fn user_to_public_profile_response(user: User) -> PublicUserProfileResponse {
    let full_name = user.full_name();
    PublicUserProfileResponse {
        id: user.id,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        full_name,
        bio: user.bio,
        avatar_url: user.avatar_url,
        website: user.website,
        location: user.location,
        created_at: user.created_at.to_rfc3339(),
    }
}

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

/// Get public profile by user ID
/// GET /api/profile/{user_id}
#[utoipa::path(
    get,
    path = "/profile/{user_id}",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = PublicUserProfileResponse),
        (status = 403, description = "Profile is private", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("user_id" = i32, Path, description = "User ID")
    ),
    tag = "profile"
)]
pub async fn get_public_profile(
    State(state): State<ConcreteAppState>,
    Path(user_id): Path<i32>,
) -> Result<Json<PublicUserProfileResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.user_repository.get_profile(user_id).await {
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

/// Update current user's profile
/// PUT /api/profile
#[utoipa::path(
    put,
    path = "/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserProfileResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "profile"
)]
pub async fn update_my_profile(
    State(state): State<ConcreteAppState>,
    Json(request): Json<UpdateProfileRequest>,
    // In a real implementation, you would extract user from JWT token
    // For now, we'll use a placeholder user_id
) -> Result<Json<UserProfileResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    // Convert privacy request to domain enum
    let privacy = request.privacy.map(convert_privacy_request).unwrap_or(ProfilePrivacy::Public);

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
    let privacy = request.privacy.map(convert_privacy_request).unwrap_or(ProfilePrivacy::Public);

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

/// Delete current user's account
/// DELETE /api/profile/delete
#[utoipa::path(
    delete,
    path = "/profile/delete",
    request_body = DeleteAccountRequest,
    responses(
        (status = 204, description = "Account deleted successfully"),
        (status = 401, description = "Invalid password", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "profile"
)]
pub async fn delete_account(
    State(state): State<ConcreteAppState>,
    Json(request): Json<DeleteAccountRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    // Get user from repository
    let user = match state.user_repository.find_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                    message: "User account not found".to_string(),
                    status_code: 404,
                }),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                    message: e.to_string(),
                    status_code: 500,
                }),
            ))
        }
    };

    // Verify password
    let is_valid = match verify(&request.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Password verification error".to_string(),
                    message: e.to_string(),
                    status_code: 500,
                }),
            ))
        }
    };

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid password".to_string(),
                message: "The password provided is incorrect".to_string(),
                status_code: 401,
            }),
        ));
    }

    // Delete account
    match state.user_repository.delete_account(user_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Deletion failed".to_string(),
                message: e.to_string(),
                status_code: 500,
            }),
        )),
    }
}
