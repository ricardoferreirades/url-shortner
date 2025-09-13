use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Domain entity representing a URL record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Url {
    pub id: i32,
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub user_id: Option<i32>, // For future user association
}

impl Url {
    /// Create a new URL entity
    pub fn new(
        id: i32,
        short_code: String,
        original_url: String,
        created_at: DateTime<Utc>,
        user_id: Option<i32>,
    ) -> Self {
        Self {
            id,
            short_code,
            original_url,
            created_at,
            user_id,
        }
    }

    /// Create a new URL with current timestamp
    pub fn new_with_timestamp(
        id: i32,
        short_code: String,
        original_url: String,
        user_id: Option<i32>,
    ) -> Self {
        Self::new(id, short_code, original_url, Utc::now(), user_id)
    }

    /// Check if this URL belongs to a specific user
    pub fn belongs_to_user(&self, user_id: i32) -> bool {
        self.user_id.map_or(true, |uid| uid == user_id)
    }

    /// Get the short URL format
    pub fn short_url(&self, base_url: &str) -> String {
        format!("{}/{}", base_url.trim_end_matches('/'), self.short_code)
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Url(id={}, short_code={}, original_url={}, created_at={})",
            self.id, self.short_code, self.original_url, self.created_at
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
        );
        
        assert_eq!(url.id, 1);
        assert_eq!(url.short_code, "abc123");
        assert_eq!(url.original_url, "https://example.com");
        assert!(url.user_id.is_none());
    }

    #[test]
    fn test_url_belongs_to_user() {
        let url = Url::new_with_timestamp(
            1,
            "abc123".to_string(),
            "https://example.com".to_string(),
            Some(42),
        );
        
        assert!(url.belongs_to_user(42));
        assert!(!url.belongs_to_user(43));
        
        let anonymous_url = Url::new_with_timestamp(
            2,
            "def456".to_string(),
            "https://example.org".to_string(),
            None,
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
        );
        
        assert_eq!(url.short_url("https://short.ly"), "https://short.ly/abc123");
        assert_eq!(url.short_url("https://short.ly/"), "https://short.ly/abc123");
    }
}
