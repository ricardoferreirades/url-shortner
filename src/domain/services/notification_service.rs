#![allow(dead_code)]
use crate::domain::entities::Url;
use tracing::{info, warn};

/// Service for handling notifications and warnings
#[derive(Clone)]
pub struct NotificationService;

impl NotificationService {
    pub fn new() -> Self {
        Self
    }

    /// Send expiration warning for a URL
    pub async fn send_expiration_warning(
        &self,
        url: &Url,
        days_until_expiry: i64,
    ) -> Result<(), NotificationError> {
        info!(
            "URL expiration warning: {} expires in {} days (expires at: {:?})",
            url.short_code, days_until_expiry, url.expiration_date
        );

        // TODO: In a real implementation, this would:
        // 1. Send email to the user who created the URL
        // 2. Send push notification if user has mobile app
        // 3. Send webhook notification to external systems
        // 4. Log to monitoring systems

        Ok(())
    }

    /// Send notification that a URL has expired
    pub async fn send_expiration_notification(&self, url: &Url) -> Result<(), NotificationError> {
        warn!(
            "URL has expired: {} (expired at: {:?})",
            url.short_code, url.expiration_date
        );

        // TODO: In a real implementation, this would:
        // 1. Send email to the user who created the URL
        // 2. Send push notification if user has mobile app
        // 3. Send webhook notification to external systems
        // 4. Log to monitoring systems

        Ok(())
    }

    /// Send bulk expiration warnings for multiple URLs
    pub async fn send_bulk_expiration_warnings(
        &self,
        urls: &[Url],
        days_until_expiry: i64,
    ) -> Result<(), NotificationError> {
        for url in urls {
            self.send_expiration_warning(url, days_until_expiry).await?;
        }
        Ok(())
    }
}

/// Notification service errors
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Email service error: {0}")]
    EmailService(String),

    #[error("Push notification error: {0}")]
    PushNotification(String),

    #[error("Webhook error: {0}")]
    Webhook(String),

    #[error("Internal notification error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::UrlStatus;
    use chrono::Utc;

    #[tokio::test]
    async fn test_send_expiration_warning() {
        let service = NotificationService::new();
        let url = Url::new_with_timestamp(
            1,
            "test123".to_string(),
            "https://example.com".to_string(),
            Some(Utc::now() + chrono::Duration::days(3)),
            Some(1),
            UrlStatus::Active,
        );

        let result = service.send_expiration_warning(&url, 3).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_expiration_notification() {
        let service = NotificationService::new();
        let url = Url::new_with_timestamp(
            1,
            "test123".to_string(),
            "https://example.com".to_string(),
            Some(Utc::now() - chrono::Duration::days(1)),
            Some(1),
            UrlStatus::Active,
        );

        let result = service.send_expiration_notification(&url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_bulk_expiration_warnings() {
        let service = NotificationService::new();
        let urls = vec![
            Url::new_with_timestamp(
                1,
                "test1".to_string(),
                "https://example1.com".to_string(),
                Some(Utc::now() + chrono::Duration::days(2)),
                Some(1),
                UrlStatus::Active,
            ),
            Url::new_with_timestamp(
                2,
                "test2".to_string(),
                "https://example2.com".to_string(),
                Some(Utc::now() + chrono::Duration::days(2)),
                Some(1),
                UrlStatus::Active,
            ),
        ];

        let result = service.send_bulk_expiration_warnings(&urls, 2).await;
        assert!(result.is_ok());
    }
}
