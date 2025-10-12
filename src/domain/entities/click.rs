use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Domain entity representing a click/access event for analytics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Click {
    pub id: i32,
    pub url_id: i32,
    pub clicked_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub country_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl Click {
    /// Create a new Click entity
    pub fn new(
        id: i32,
        url_id: i32,
        clicked_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        referer: Option<String>,
        country_code: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            url_id,
            clicked_at,
            ip_address,
            user_agent,
            referer,
            country_code,
            created_at,
        }
    }

    /// Create a new Click with current timestamp
    pub fn new_with_timestamp(
        id: i32,
        url_id: i32,
        ip_address: Option<String>,
        user_agent: Option<String>,
        referer: Option<String>,
        country_code: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self::new(
            id,
            url_id,
            now,
            ip_address,
            user_agent,
            referer,
            country_code,
            now,
        )
    }

    /// Create a new Click for tracking (without ID, for database insertion)
    pub fn new_for_tracking(
        url_id: i32,
        ip_address: Option<String>,
        user_agent: Option<String>,
        referer: Option<String>,
        country_code: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self::new(
            0,
            url_id,
            now,
            ip_address,
            user_agent,
            referer,
            country_code,
            now,
        )
    }

    /// Check if this click has geographic information
    pub fn has_geographic_data(&self) -> bool {
        self.country_code.is_some()
    }

    /// Get a sanitized user agent (first 100 characters)
    pub fn sanitized_user_agent(&self) -> Option<String> {
        self.user_agent.as_ref().map(|ua| {
            if ua.len() > 100 {
                format!("{}...", &ua[..97])
            } else {
                ua.clone()
            }
        })
    }
}

impl fmt::Display for Click {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Click(id={}, url_id={}, clicked_at={}, ip={})",
            self.id,
            self.url_id,
            self.clicked_at,
            self.ip_address.as_deref().unwrap_or("unknown")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_creation() {
        let click = Click::new_with_timestamp(
            1,
            42,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0...".to_string()),
            Some("https://google.com".to_string()),
            Some("US".to_string()),
        );

        assert_eq!(click.id, 1);
        assert_eq!(click.url_id, 42);
        assert_eq!(click.ip_address, Some("192.168.1.1".to_string()));
        assert!(click.has_geographic_data());
    }

    #[test]
    fn test_click_for_tracking() {
        let click = Click::new_for_tracking(
            42,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0...".to_string()),
            None,
            None,
        );

        assert_eq!(click.id, 0); // ID will be set by database
        assert_eq!(click.url_id, 42);
        assert!(!click.has_geographic_data());
    }

    #[test]
    fn test_sanitized_user_agent() {
        let long_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let click = Click::new_with_timestamp(1, 42, None, Some(long_ua.to_string()), None, None);

        let sanitized = click.sanitized_user_agent().unwrap();
        assert!(sanitized.len() <= 100);
        assert!(sanitized.ends_with("..."));
    }

    #[test]
    fn test_geographic_data_detection() {
        let click_with_geo =
            Click::new_with_timestamp(1, 42, None, None, None, Some("US".to_string()));

        let click_without_geo = Click::new_with_timestamp(2, 42, None, None, None, None);

        assert!(click_with_geo.has_geographic_data());
        assert!(!click_without_geo.has_geographic_data());
    }
}
