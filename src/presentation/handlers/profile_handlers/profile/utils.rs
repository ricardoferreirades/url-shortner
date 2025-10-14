use crate::application::dto::{
    requests::ProfilePrivacyRequest,
    responses::{ProfilePrivacyResponse, PublicUserProfileResponse, UserProfileResponse},
};
use crate::domain::entities::{ProfilePrivacy, User};

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

/// Convert User entity to UserProfileResponse
pub fn user_to_profile_response(user: User) -> UserProfileResponse {
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
pub fn user_to_public_profile_response(user: User) -> PublicUserProfileResponse {
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
