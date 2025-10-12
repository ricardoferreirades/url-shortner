use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Privacy settings for user profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfilePrivacy {
    Public,
    Private,
    FriendsOnly,
}

impl Default for ProfilePrivacy {
    fn default() -> Self {
        ProfilePrivacy::Public
    }
}

/// Domain entity representing a User
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    // Profile fields
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub privacy: ProfilePrivacy,
    pub updated_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
impl User {
    /// Create a new User entity
    #[allow(dead_code)]
    pub fn new(
        id: i32,
        username: String,
        email: String,
        password_hash: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            created_at,
            first_name: None,
            last_name: None,
            bio: None,
            avatar_url: None,
            website: None,
            location: None,
            privacy: ProfilePrivacy::default(),
            updated_at: None,
        }
    }

    /// Create a new User with current timestamp
    pub fn new_with_timestamp(
        id: i32,
        username: String,
        email: String,
        password_hash: String,
    ) -> Self {
        Self::new(id, username, email, password_hash, Utc::now())
    }

    /// Create a new User with profile fields
    pub fn new_with_profile(
        id: i32,
        username: String,
        email: String,
        password_hash: String,
        created_at: DateTime<Utc>,
        first_name: Option<String>,
        last_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        website: Option<String>,
        location: Option<String>,
        privacy: ProfilePrivacy,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            created_at,
            first_name,
            last_name,
            bio,
            avatar_url,
            website,
            location,
            privacy,
            updated_at: None,
        }
    }

    /// Update profile fields
    pub fn update_profile(
        &mut self,
        first_name: Option<String>,
        last_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        website: Option<String>,
        location: Option<String>,
        privacy: Option<ProfilePrivacy>,
    ) {
        if let Some(name) = first_name {
            self.first_name = Some(name);
        }
        if let Some(name) = last_name {
            self.last_name = Some(name);
        }
        if let Some(bio) = bio {
            self.bio = Some(bio);
        }
        if let Some(url) = avatar_url {
            self.avatar_url = Some(url);
        }
        if let Some(site) = website {
            self.website = Some(site);
        }
        if let Some(loc) = location {
            self.location = Some(loc);
        }
        if let Some(privacy) = privacy {
            self.privacy = privacy;
        }
        self.updated_at = Some(Utc::now());
    }

    /// Get full name (first_name + last_name)
    pub fn full_name(&self) -> Option<String> {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => None,
        }
    }

    /// Check if profile is public
    pub fn is_profile_public(&self) -> bool {
        matches!(self.privacy, ProfilePrivacy::Public)
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User(id={}, username={}, email={}, created_at={}, privacy={:?})",
            self.id, self.username, self.email, self.created_at, self.privacy
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new_with_timestamp(
            1,
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert_eq!(user.privacy, ProfilePrivacy::Public);
        assert!(user.first_name.is_none());
        assert!(user.last_name.is_none());
    }

    #[test]
    fn test_user_with_profile() {
        let user = User::new_with_profile(
            1,
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Utc::now(),
            Some("John".to_string()),
            Some("Doe".to_string()),
            Some("Software developer".to_string()),
            Some("https://example.com/avatar.jpg".to_string()),
            Some("https://johndoe.com".to_string()),
            Some("New York, NY".to_string()),
            ProfilePrivacy::Public,
        );

        assert_eq!(user.first_name, Some("John".to_string()));
        assert_eq!(user.last_name, Some("Doe".to_string()));
        assert_eq!(user.bio, Some("Software developer".to_string()));
        assert_eq!(user.website, Some("https://johndoe.com".to_string()));
        assert_eq!(user.location, Some("New York, NY".to_string()));
        assert_eq!(user.privacy, ProfilePrivacy::Public);
    }

    #[test]
    fn test_full_name() {
        let mut user = User::new_with_timestamp(
            1,
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        // No name set
        assert_eq!(user.full_name(), None);

        // Only first name
        user.first_name = Some("John".to_string());
        assert_eq!(user.full_name(), Some("John".to_string()));

        // Only last name
        user.first_name = None;
        user.last_name = Some("Doe".to_string());
        assert_eq!(user.full_name(), Some("Doe".to_string()));

        // Both names
        user.first_name = Some("John".to_string());
        assert_eq!(user.full_name(), Some("John Doe".to_string()));
    }

    #[test]
    fn test_update_profile() {
        let mut user = User::new_with_timestamp(
            1,
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        user.update_profile(
            Some("Jane".to_string()),
            Some("Smith".to_string()),
            Some("Updated bio".to_string()),
            Some("https://example.com/new-avatar.jpg".to_string()),
            Some("https://janesmith.com".to_string()),
            Some("San Francisco, CA".to_string()),
            Some(ProfilePrivacy::Private),
        );

        assert_eq!(user.first_name, Some("Jane".to_string()));
        assert_eq!(user.last_name, Some("Smith".to_string()));
        assert_eq!(user.bio, Some("Updated bio".to_string()));
        assert_eq!(
            user.avatar_url,
            Some("https://example.com/new-avatar.jpg".to_string())
        );
        assert_eq!(user.website, Some("https://janesmith.com".to_string()));
        assert_eq!(user.location, Some("San Francisco, CA".to_string()));
        assert_eq!(user.privacy, ProfilePrivacy::Private);
        assert!(user.updated_at.is_some());
    }

    #[test]
    fn test_profile_privacy() {
        let public_user = User::new_with_profile(
            1,
            "publicuser".to_string(),
            "public@example.com".to_string(),
            "hash".to_string(),
            Utc::now(),
            None,
            None,
            None,
            None,
            None,
            None,
            ProfilePrivacy::Public,
        );

        let private_user = User::new_with_profile(
            2,
            "privateuser".to_string(),
            "private@example.com".to_string(),
            "hash".to_string(),
            Utc::now(),
            None,
            None,
            None,
            None,
            None,
            None,
            ProfilePrivacy::Private,
        );

        assert!(public_user.is_profile_public());
        assert!(!private_user.is_profile_public());
    }
}
