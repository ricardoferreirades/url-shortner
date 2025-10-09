use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Status of a URL - active or inactive (soft deleted)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum UrlStatus {
    /// URL is active and can be accessed
    Active,
    /// URL is inactive (soft deleted) and should not redirect
    Inactive,
}

impl UrlStatus {
    /// Check if the URL status allows access
    pub fn is_active(&self) -> bool {
        matches!(self, UrlStatus::Active)
    }
}

impl Default for UrlStatus {
    fn default() -> Self {
        UrlStatus::Active
    }
}

impl fmt::Display for UrlStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlStatus::Active => write!(f, "active"),
            UrlStatus::Inactive => write!(f, "inactive"),
        }
    }
}

/// Domain entity representing a URL record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Url {
    pub id: i32,
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub user_id: Option<i32>, // For future user association
    pub status: UrlStatus, // URL status (active/inactive)
}

#[allow(dead_code)]
impl Url {
    /// Create a new URL entity
    pub fn new(
        id: i32,
        short_code: String,
        original_url: String,
        created_at: DateTime<Utc>,
        expiration_date: Option<DateTime<Utc>>,
        user_id: Option<i32>,
        status: UrlStatus,
    ) -> Self {
        Self {
            id,
            short_code,
            original_url,
            created_at,
            expiration_date,
            user_id,
            status,
        }
    }

    /// Create a new URL with current timestamp
    pub fn new_with_timestamp(
        id: i32,
        short_code: String,
        original_url: String,
        expiration_date: Option<DateTime<Utc>>,
        user_id: Option<i32>,
        status: UrlStatus,
    ) -> Self {
        Self::new(id, short_code, original_url, Utc::now(), expiration_date, user_id, status)
    }

    /// Check if this URL belongs to a specific user
    pub fn belongs_to_user(&self, user_id: i32) -> bool {
        self.user_id.map_or(true, |uid| uid == user_id)
    }

    /// Get the short URL format
    pub fn short_url(&self, base_url: &str) -> String {
        format!("{}/{}", base_url.trim_end_matches('/'), self.short_code)
    }

    /// Check if the URL is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration_date {
            Utc::now() > expiration
        } else {
            false // No expiration date means never expires
        }
    }

    /// Check if the URL will expire within the given duration
    pub fn expires_within(&self, duration: chrono::Duration) -> bool {
        if let Some(expiration) = self.expiration_date {
            let now = Utc::now();
            let warning_time = expiration - duration;
            now >= warning_time && now < expiration
        } else {
            false
        }
    }

    /// Check if the URL is accessible (active and not expired)
    pub fn is_accessible(&self) -> bool {
        self.status.is_active() && !self.is_expired()
    }

    /// Deactivate the URL (soft delete)
    pub fn deactivate(&mut self) {
        self.status = UrlStatus::Inactive;
    }

    /// Reactivate the URL
    pub fn reactivate(&mut self) {
        self.status = UrlStatus::Active;
    }

    /// Check if the URL is deactivated
    pub fn is_deactivated(&self) -> bool {
        matches!(self.status, UrlStatus::Inactive)
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Url(id={}, short_code={}, original_url={}, created_at={}, expiration_date={:?}, status={})",
            self.id, self.short_code, self.original_url, self.created_at, self.expiration_date, self.status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_creation() {
        let url = Url::new_with_timestamp(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            None,
            None,
            UrlStatus::Active,
        );
        
        assert_eq!(url.id, 1);
        assert_eq!(url.short_code, "abc123");
        assert_eq!(url.original_url, "https://example.com");
        assert!(url.user_id.is_none());
        assert!(url.expiration_date.is_none());
        assert_eq!(url.status, UrlStatus::Active);
    }

    #[test]
    fn test_url_belongs_to_user() {
        let url = Url::new_with_timestamp(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            None,
            Some(42),
            UrlStatus::Active,
        );
        
        assert!(url.belongs_to_user(42));
        assert!(!url.belongs_to_user(43));
        
        let anonymous_url = Url::new_with_timestamp(
            2,
            "def456".to_string(),
            "https://example.org".to_string(),
            None,
            None,
            UrlStatus::Active,
        );
        
        assert!(anonymous_url.belongs_to_user(42)); // Anonymous URLs belong to everyone
    }

    #[test]
    fn test_short_url_generation() {
        let url = Url::new_with_timestamp(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            None,
            None,
            UrlStatus::Active,
        );
        
        assert_eq!(url.short_url("https://short.ly"), "https://short.ly/abc123");
        assert_eq!(url.short_url("https://short.ly/"), "https://short.ly/abc123");
    }

    #[test]
    fn test_url_expiration() {
        let now = Utc::now();
        let future = now + chrono::Duration::hours(1);
        let past = now - chrono::Duration::hours(1);

        // URL with no expiration (never expires)
        let url_no_expiry = Url::new_with_timestamp(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            None,
            None,
            UrlStatus::Active,
        );
        assert!(!url_no_expiry.is_expired());

        // URL with future expiration
        let url_future = Url::new_with_timestamp(
            2,
            "def456".to_string(),
            "https://example.org".to_string(),
            Some(future),
            None,
            UrlStatus::Active,
        );
        assert!(!url_future.is_expired());

        // URL with past expiration
        let url_past = Url::new_with_timestamp(
            3,
            "ghi789".to_string(),
            "https://example.net".to_string(),
            Some(past),
            None,
            UrlStatus::Active,
        );
        assert!(url_past.is_expired());
    }

    #[test]
    fn test_url_expires_within() {
        let now = Utc::now();
        let expires_in_30_min = now + chrono::Duration::minutes(30);
        let expires_in_2_hours = now + chrono::Duration::hours(2);

        // URL expiring in 30 minutes, warning period 1 hour
        let url_warning = Url::new_with_timestamp(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            Some(expires_in_30_min),
            None,
            UrlStatus::Active,
        );
        assert!(url_warning.expires_within(chrono::Duration::hours(1)));

        // URL expiring in 2 hours, warning period 1 hour
        let url_no_warning = Url::new_with_timestamp(
            2,
            "def456".to_string(),
            "https://example.org".to_string(),
            Some(expires_in_2_hours),
            None,
            UrlStatus::Active,
        );
        assert!(!url_no_warning.expires_within(chrono::Duration::hours(1)));
    }

    #[test]
    fn test_url_status_functionality() {
        let mut url = Url::new_with_timestamp(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            None,
            None,
            UrlStatus::Active,
        );

        // Test initial state
        assert!(url.status.is_active());
        assert!(url.is_accessible());
        assert!(!url.is_deactivated());

        // Test deactivation
        url.deactivate();
        assert!(url.is_deactivated());
        assert!(!url.is_accessible());
        assert_eq!(url.status, UrlStatus::Inactive);

        // Test reactivation
        url.reactivate();
        assert!(url.status.is_active());
        assert!(url.is_accessible());
        assert!(!url.is_deactivated());
    }

    #[test]
    fn test_url_status_display() {
        assert_eq!(UrlStatus::Active.to_string(), "active");
        assert_eq!(UrlStatus::Inactive.to_string(), "inactive");
    }
}
