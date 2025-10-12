use crate::domain::entities::{ProfilePrivacy, User};
use thiserror::Error;

/// Privacy service for handling user profile privacy settings
pub struct PrivacyService;

/// Privacy service errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum PrivacyServiceError {
    #[error("Invalid privacy setting: {0}")]
    InvalidPrivacySetting(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Profile not found")]
    ProfileNotFound,
}

/// Privacy level for different types of data
#[derive(Debug, Clone, PartialEq)]
pub enum DataPrivacyLevel {
    Public,
    Private,
    FriendsOnly,
}

/// Privacy settings for different profile fields
#[derive(Debug, Clone)]
pub struct FieldPrivacySettings {
    pub first_name: DataPrivacyLevel,
    pub last_name: DataPrivacyLevel,
    pub bio: DataPrivacyLevel,
    pub avatar_url: DataPrivacyLevel,
    pub website: DataPrivacyLevel,
    pub location: DataPrivacyLevel,
    pub email: DataPrivacyLevel,
}

#[allow(dead_code)]
impl PrivacyService {
    /// Create a new privacy service
    pub fn new() -> Self {
        Self
    }

    /// Get default privacy settings for a new user
    pub fn get_default_privacy_settings() -> FieldPrivacySettings {
        FieldPrivacySettings {
            first_name: DataPrivacyLevel::Public,
            last_name: DataPrivacyLevel::Public,
            bio: DataPrivacyLevel::Public,
            avatar_url: DataPrivacyLevel::Public,
            website: DataPrivacyLevel::Public,
            location: DataPrivacyLevel::Public,
            email: DataPrivacyLevel::Private,
        }
    }

    /// Check if a user can view another user's profile
    pub fn can_view_profile(
        &self,
        viewer: Option<&User>,
        profile_owner: &User,
    ) -> Result<bool, PrivacyServiceError> {
        match profile_owner.privacy {
            ProfilePrivacy::Public => Ok(true),
            ProfilePrivacy::Private => {
                // Only the owner can view private profiles
                if let Some(viewer) = viewer {
                    Ok(viewer.id == profile_owner.id)
                } else {
                    Ok(false)
                }
            }
            ProfilePrivacy::FriendsOnly => {
                // For now, only the owner can view friends-only profiles
                // In a real implementation, you would check if the viewer is a friend
                if let Some(viewer) = viewer {
                    Ok(viewer.id == profile_owner.id)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Check if a specific field can be viewed by a user
    pub fn can_view_field(
        &self,
        viewer: Option<&User>,
        profile_owner: &User,
        field_privacy: DataPrivacyLevel,
    ) -> Result<bool, PrivacyServiceError> {
        // If the profile itself is private, check if viewer can view the profile
        if !self.can_view_profile(viewer, profile_owner)? {
            return Ok(false);
        }

        // Check field-specific privacy
        match field_privacy {
            DataPrivacyLevel::Public => Ok(true),
            DataPrivacyLevel::Private => {
                // Only the owner can view private fields
                if let Some(viewer) = viewer {
                    Ok(viewer.id == profile_owner.id)
                } else {
                    Ok(false)
                }
            }
            DataPrivacyLevel::FriendsOnly => {
                // For now, only the owner can view friends-only fields
                // In a real implementation, you would check if the viewer is a friend
                if let Some(viewer) = viewer {
                    Ok(viewer.id == profile_owner.id)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Filter user profile data based on privacy settings
    pub fn filter_profile_data(
        &self,
        viewer: Option<&User>,
        profile_owner: &User,
        field_settings: &FieldPrivacySettings,
    ) -> Result<FilteredProfileData, PrivacyServiceError> {
        let can_view_profile = self.can_view_profile(viewer, profile_owner)?;

        if !can_view_profile {
            return Err(PrivacyServiceError::AccessDenied(
                "Cannot view this profile".to_string(),
            ));
        }

        let first_name =
            if self.can_view_field(viewer, profile_owner, field_settings.first_name.clone())? {
                profile_owner.first_name.clone()
            } else {
                None
            };

        let last_name =
            if self.can_view_field(viewer, profile_owner, field_settings.last_name.clone())? {
                profile_owner.last_name.clone()
            } else {
                None
            };

        let bio = if self.can_view_field(viewer, profile_owner, field_settings.bio.clone())? {
            profile_owner.bio.clone()
        } else {
            None
        };

        let avatar_url =
            if self.can_view_field(viewer, profile_owner, field_settings.avatar_url.clone())? {
                profile_owner.avatar_url.clone()
            } else {
                None
            };

        let website =
            if self.can_view_field(viewer, profile_owner, field_settings.website.clone())? {
                profile_owner.website.clone()
            } else {
                None
            };

        let location =
            if self.can_view_field(viewer, profile_owner, field_settings.location.clone())? {
                profile_owner.location.clone()
            } else {
                None
            };

        let email = if self.can_view_field(viewer, profile_owner, field_settings.email.clone())? {
            Some(profile_owner.email.clone())
        } else {
            None
        };

        Ok(FilteredProfileData {
            id: profile_owner.id,
            username: profile_owner.username.clone(),
            email,
            first_name,
            last_name,
            bio,
            avatar_url,
            website,
            location,
            privacy: profile_owner.privacy.clone(),
            created_at: profile_owner.created_at,
            updated_at: profile_owner.updated_at,
        })
    }

    /// Validate privacy setting
    pub fn validate_privacy_setting(
        &self,
        privacy: &ProfilePrivacy,
    ) -> Result<(), PrivacyServiceError> {
        match privacy {
            ProfilePrivacy::Public | ProfilePrivacy::Private | ProfilePrivacy::FriendsOnly => {
                Ok(())
            }
        }
    }

    /// Get privacy description
    pub fn get_privacy_description(&self, privacy: &ProfilePrivacy) -> &'static str {
        match privacy {
            ProfilePrivacy::Public => "Profile is visible to everyone",
            ProfilePrivacy::Private => "Profile is only visible to you",
            ProfilePrivacy::FriendsOnly => "Profile is visible to friends only",
        }
    }

    /// Check if profile is searchable
    pub fn is_profile_searchable(&self, privacy: &ProfilePrivacy) -> bool {
        matches!(privacy, ProfilePrivacy::Public)
    }

    /// Get recommended privacy settings based on user preferences
    pub fn get_recommended_privacy_settings(
        &self,
        is_public_figure: bool,
        is_business_account: bool,
        is_personal_account: bool,
    ) -> FieldPrivacySettings {
        if is_public_figure || is_business_account {
            // More open settings for public figures and businesses
            FieldPrivacySettings {
                first_name: DataPrivacyLevel::Public,
                last_name: DataPrivacyLevel::Public,
                bio: DataPrivacyLevel::Public,
                avatar_url: DataPrivacyLevel::Public,
                website: DataPrivacyLevel::Public,
                location: DataPrivacyLevel::Public,
                email: DataPrivacyLevel::Private,
            }
        } else if is_personal_account {
            // More private settings for personal accounts
            FieldPrivacySettings {
                first_name: DataPrivacyLevel::Public,
                last_name: DataPrivacyLevel::FriendsOnly,
                bio: DataPrivacyLevel::FriendsOnly,
                avatar_url: DataPrivacyLevel::Public,
                website: DataPrivacyLevel::FriendsOnly,
                location: DataPrivacyLevel::FriendsOnly,
                email: DataPrivacyLevel::Private,
            }
        } else {
            // Default balanced settings
            Self::get_default_privacy_settings()
        }
    }
}

/// Filtered profile data based on privacy settings
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FilteredProfileData {
    pub id: i32,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub privacy: ProfilePrivacy,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for PrivacyService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_user(id: i32, username: &str, privacy: ProfilePrivacy) -> User {
        User::new_with_profile(
            id,
            username.to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            Utc::now(),
            Some("John".to_string()),
            Some("Doe".to_string()),
            Some("Software developer".to_string()),
            Some("https://example.com/avatar.jpg".to_string()),
            Some("https://johndoe.com".to_string()),
            Some("New York, NY".to_string()),
            privacy,
        )
    }

    #[test]
    fn test_can_view_profile() {
        let service = PrivacyService::new();
        let public_user = create_test_user(1, "public_user", ProfilePrivacy::Public);
        let private_user = create_test_user(2, "private_user", ProfilePrivacy::Private);
        let viewer = create_test_user(3, "viewer", ProfilePrivacy::Public);

        // Anyone can view public profiles
        assert!(service.can_view_profile(None, &public_user).unwrap());
        assert!(service
            .can_view_profile(Some(&viewer), &public_user)
            .unwrap());

        // Only owner can view private profiles
        assert!(!service.can_view_profile(None, &private_user).unwrap());
        assert!(!service
            .can_view_profile(Some(&viewer), &private_user)
            .unwrap());
        assert!(service
            .can_view_profile(Some(&private_user), &private_user)
            .unwrap());
    }

    #[test]
    fn test_filter_profile_data() {
        let service = PrivacyService::new();
        let public_user = create_test_user(1, "public_user", ProfilePrivacy::Public);
        let field_settings = FieldPrivacySettings {
            first_name: DataPrivacyLevel::Public,
            last_name: DataPrivacyLevel::Public,
            bio: DataPrivacyLevel::Public,
            avatar_url: DataPrivacyLevel::Public,
            website: DataPrivacyLevel::Public,
            location: DataPrivacyLevel::Public,
            email: DataPrivacyLevel::Private,
        };

        // Anonymous user can view public profile
        let filtered = service
            .filter_profile_data(None, &public_user, &field_settings)
            .unwrap();
        assert_eq!(filtered.first_name, Some("John".to_string()));
        assert_eq!(filtered.email, None); // Email is private

        // Owner can view all fields
        let filtered = service
            .filter_profile_data(Some(&public_user), &public_user, &field_settings)
            .unwrap();
        assert_eq!(filtered.first_name, Some("John".to_string()));
        assert_eq!(filtered.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_privacy_descriptions() {
        let service = PrivacyService::new();

        assert_eq!(
            service.get_privacy_description(&ProfilePrivacy::Public),
            "Profile is visible to everyone"
        );
        assert_eq!(
            service.get_privacy_description(&ProfilePrivacy::Private),
            "Profile is only visible to you"
        );
        assert_eq!(
            service.get_privacy_description(&ProfilePrivacy::FriendsOnly),
            "Profile is visible to friends only"
        );
    }

    #[test]
    fn test_recommended_privacy_settings() {
        let service = PrivacyService::new();

        // Business account settings
        let business_settings = service.get_recommended_privacy_settings(true, true, false);
        assert_eq!(business_settings.first_name, DataPrivacyLevel::Public);
        assert_eq!(business_settings.email, DataPrivacyLevel::Private);

        // Personal account settings
        let personal_settings = service.get_recommended_privacy_settings(false, false, true);
        assert_eq!(personal_settings.first_name, DataPrivacyLevel::Public);
        assert_eq!(personal_settings.last_name, DataPrivacyLevel::FriendsOnly);
        assert_eq!(personal_settings.email, DataPrivacyLevel::Private);
    }
}
