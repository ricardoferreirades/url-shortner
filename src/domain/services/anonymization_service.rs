use crate::domain::entities::User;
use chrono::Utc;
use thiserror::Error;

/// Service for anonymizing user data upon account deletion
#[derive(Clone, Default)]
pub struct AnonymizationService;

impl AnonymizationService {
    pub fn new() -> Self {
        Self
    }

    /// Anonymize user data for GDPR compliance
    /// Replaces personal information with anonymized values while preserving referential integrity
    pub fn anonymize_user_data(&self, user: &User) -> AnonymizedUserData {
        AnonymizedUserData {
            user_id: user.id,
            username: self.anonymize_username(user.id),
            email: self.anonymize_email(user.id),
            password_hash: self.generate_anonymized_hash(),
            deleted_at: Utc::now(),
        }
    }

    /// Generate anonymized username
    fn anonymize_username(&self, user_id: i32) -> String {
        format!("deleted_user_{}", user_id)
    }

    /// Generate anonymized email
    fn anonymize_email(&self, user_id: i32) -> String {
        format!("deleted_{}@anonymized.local", user_id)
    }

    /// Generate a placeholder password hash
    fn generate_anonymized_hash(&self) -> String {
        // Use a fixed hash to indicate deleted account
        "DELETED_ACCOUNT_HASH".to_string()
    }
}

/// Anonymized user data structure
#[derive(Debug, Clone)]
pub struct AnonymizedUserData {
    #[allow(dead_code)]
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[allow(dead_code)]
    pub deleted_at: chrono::DateTime<Utc>,
}

/// Anonymization errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AnonymizationError {
    #[error("User data is already anonymized")]
    AlreadyAnonymized,

    #[error("Cannot anonymize system user")]
    SystemUser,

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ProfilePrivacy;

    #[test]
    fn test_anonymize_user_data() {
        let service = AnonymizationService::new();
        let user = User::new_with_profile(
            123,
            "john_doe".to_string(),
            "john@example.com".to_string(),
            "original_hash".to_string(),
            Utc::now(),
            Some("John".to_string()),
            Some("Doe".to_string()),
            Some("Software developer".to_string()),
            Some("https://example.com/avatar.jpg".to_string()),
            Some("https://johndoe.com".to_string()),
            Some("New York".to_string()),
            ProfilePrivacy::Public,
        );

        let anonymized = service.anonymize_user_data(&user);

        assert_eq!(anonymized.user_id, 123);
        assert_eq!(anonymized.username, "deleted_user_123");
        assert_eq!(anonymized.email, "deleted_123@anonymized.local");
        assert_eq!(anonymized.password_hash, "DELETED_ACCOUNT_HASH");
        assert!(anonymized.deleted_at <= Utc::now());
    }

    #[test]
    fn test_anonymize_username() {
        let service = AnonymizationService::new();
        assert_eq!(service.anonymize_username(1), "deleted_user_1");
        assert_eq!(service.anonymize_username(999), "deleted_user_999");
    }

    #[test]
    fn test_anonymize_email() {
        let service = AnonymizationService::new();
        assert_eq!(service.anonymize_email(1), "deleted_1@anonymized.local");
        assert_eq!(service.anonymize_email(999), "deleted_999@anonymized.local");
    }

    #[test]
    fn test_generate_anonymized_hash() {
        let service = AnonymizationService::new();
        let hash = service.generate_anonymized_hash();
        assert_eq!(hash, "DELETED_ACCOUNT_HASH");
    }
}
