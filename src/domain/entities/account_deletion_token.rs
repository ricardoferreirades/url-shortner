use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Domain entity representing an Account Deletion Confirmation Token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountDeletionToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub is_confirmed: bool,
    pub is_cancelled: bool,
}

#[allow(dead_code)]
impl AccountDeletionToken {
    /// Create a new Account Deletion Token
    pub fn new(
        id: i32,
        user_id: i32,
        token: String,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            token,
            created_at,
            expires_at,
            confirmed_at: None,
            is_confirmed: false,
            is_cancelled: false,
        }
    }

    /// Create a new Account Deletion Token with current timestamp
    pub fn new_with_timestamp(
        id: i32,
        user_id: i32,
        token: String,
        expiration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(expiration_hours);
        
        Self::new(id, user_id, token, now, expires_at)
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token is valid (not expired, not confirmed, not cancelled)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_confirmed && !self.is_cancelled
    }

    /// Mark the token as confirmed
    pub fn mark_as_confirmed(&mut self) {
        self.is_confirmed = true;
        self.confirmed_at = Some(Utc::now());
    }

    /// Mark the token as cancelled
    pub fn mark_as_cancelled(&mut self) {
        self.is_cancelled = true;
    }

    /// Get time until expiration
    pub fn time_until_expiration(&self) -> Option<Duration> {
        let now = Utc::now();
        if now < self.expires_at {
            Some(self.expires_at - now)
        } else {
            None
        }
    }

    /// Get time since creation
    pub fn time_since_creation(&self) -> Duration {
        Utc::now() - self.created_at
    }
}

impl fmt::Display for AccountDeletionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountDeletionToken(id={}, user_id={}, token={}..., expires_at={}, is_confirmed={})",
            self.id,
            self.user_id,
            &self.token[..8.min(self.token.len())], // Only show first 8 characters for security
            self.expires_at,
            self.is_confirmed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_deletion_token_creation() {
        let token = AccountDeletionToken::new_with_timestamp(
            1,
            123,
            "test_token_123".to_string(),
            24, // 24 hours
        );
        
        assert_eq!(token.id, 1);
        assert_eq!(token.user_id, 123);
        assert_eq!(token.token, "test_token_123");
        assert!(!token.is_confirmed);
        assert!(!token.is_cancelled);
        assert!(token.confirmed_at.is_none());
    }

    #[test]
    fn test_token_expiration() {
        let token = AccountDeletionToken::new_with_timestamp(
            1,
            123,
            "test_token".to_string(),
            -1, // Expired 1 hour ago
        );
        
        assert!(token.is_expired());
        assert!(!token.is_valid());
    }

    #[test]
    fn test_token_validation() {
        let token = AccountDeletionToken::new_with_timestamp(
            1,
            123,
            "test_token".to_string(),
            24, // Valid for 24 hours
        );
        
        assert!(!token.is_expired());
        assert!(token.is_valid());
        
        // Mark as confirmed
        let mut confirmed_token = token.clone();
        confirmed_token.mark_as_confirmed();
        
        assert!(!confirmed_token.is_valid());
        assert!(confirmed_token.is_confirmed);
        assert!(confirmed_token.confirmed_at.is_some());
    }

    #[test]
    fn test_token_cancellation() {
        let mut token = AccountDeletionToken::new_with_timestamp(
            1,
            123,
            "test_token".to_string(),
            24,
        );
        
        assert!(token.is_valid());
        
        token.mark_as_cancelled();
        
        assert!(!token.is_valid());
        assert!(token.is_cancelled);
    }

    #[test]
    fn test_time_calculations() {
        let token = AccountDeletionToken::new_with_timestamp(
            1,
            123,
            "test_token".to_string(),
            24, // 24 hours
        );
        
        // Should have time until expiration
        assert!(token.time_until_expiration().is_some());
        
        // Should have time since creation (should be very small)
        let time_since = token.time_since_creation();
        assert!(time_since.num_seconds() >= 0);
    }
}

