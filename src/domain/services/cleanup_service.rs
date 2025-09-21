use crate::domain::repositories::UrlRepository;
use crate::domain::services::NotificationService;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, error};

/// Service for handling background cleanup tasks
#[derive(Clone)]
pub struct CleanupService<R>
where
    R: UrlRepository + Clone,
{
    url_repository: R,
    notification_service: NotificationService,
}

impl<R> CleanupService<R>
where
    R: UrlRepository + Clone,
{
    pub fn new(url_repository: R) -> Self {
        Self { 
            url_repository,
            notification_service: NotificationService::new(),
        }
    }

    /// Start the cleanup service with the specified interval
    pub async fn start_cleanup_service(&self, cleanup_interval_hours: u64) {
        let mut interval = interval(Duration::from_secs(cleanup_interval_hours * 3600));
        
        info!("Starting URL cleanup service with {} hour interval", cleanup_interval_hours);
        
        loop {
            interval.tick().await;
            
            // Send warnings for URLs expiring soon (7 days)
            if let Err(e) = self.send_expiration_warnings(7).await {
                error!("Failed to send expiration warnings: {}", e);
            }
            
            // Clean up expired URLs
            match self.cleanup_expired_urls().await {
                Ok(deleted_count) => {
                    if deleted_count > 0 {
                        info!("Cleaned up {} expired URLs", deleted_count);
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup expired URLs: {}", e);
                }
            }
        }
    }

    /// Clean up expired URLs
    pub async fn cleanup_expired_urls(&self) -> Result<u64, CleanupError> {
        let deleted_count = self.url_repository.delete_expired_urls().await
            .map_err(CleanupError::Repository)?;
        
        Ok(deleted_count)
    }

    /// Get URLs that are expiring soon for notification purposes
    pub async fn get_urls_expiring_soon(&self, warning_duration: chrono::Duration) -> Result<Vec<crate::domain::entities::Url>, CleanupError> {
        self.url_repository.find_urls_expiring_soon(warning_duration).await
            .map_err(CleanupError::Repository)
    }

    /// Get all expired URLs
    pub async fn get_expired_urls(&self) -> Result<Vec<crate::domain::entities::Url>, CleanupError> {
        self.url_repository.find_expired_urls().await
            .map_err(CleanupError::Repository)
    }

    /// Send expiration warnings for URLs expiring soon
    pub async fn send_expiration_warnings(&self, warning_days: u32) -> Result<(), CleanupError> {
        let duration = chrono::Duration::days(warning_days as i64);
        let expiring_urls = self.url_repository.find_urls_expiring_soon(duration).await
            .map_err(CleanupError::Repository)?;

        if !expiring_urls.is_empty() {
            info!("Found {} URLs expiring within {} days", expiring_urls.len(), warning_days);
            
            // Send bulk warnings
            self.notification_service.send_bulk_expiration_warnings(&expiring_urls, warning_days as i64).await
                .map_err(|e| CleanupError::TaskError(format!("Failed to send notifications: {}", e)))?;
        }

        Ok(())
    }
}

/// Cleanup service errors
#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("Repository error: {0}")]
    Repository(#[from] crate::domain::repositories::RepositoryError),
    
    #[error("Cleanup task error: {0}")]
    TaskError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::UrlRepository;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // Mock repository for testing
    #[derive(Clone)]
    struct MockUrlRepository {
        urls: Arc<Mutex<Vec<crate::domain::entities::Url>>>,
    }

    impl MockUrlRepository {
        fn new() -> Self {
            Self {
                urls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl UrlRepository for MockUrlRepository {
        async fn create_url(
            &self,
            _short_code: &crate::domain::entities::ShortCode,
            _original_url: &str,
            _expiration_date: Option<chrono::DateTime<chrono::Utc>>,
            _user_id: Option<i32>,
            _status: crate::domain::entities::UrlStatus,
        ) -> Result<crate::domain::entities::Url, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn find_by_short_code(&self, _short_code: &crate::domain::entities::ShortCode) -> Result<Option<crate::domain::entities::Url>, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn find_by_user_id(&self, _user_id: i32) -> Result<Vec<crate::domain::entities::Url>, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn exists_by_short_code(&self, _short_code: &crate::domain::entities::ShortCode) -> Result<bool, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn delete_by_id(&self, _id: i32, _user_id: Option<i32>) -> Result<bool, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn update_url(&self, _url: &crate::domain::entities::Url) -> Result<crate::domain::entities::Url, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn get_stats(&self, _user_id: Option<i32>) -> Result<crate::domain::repositories::UrlStats, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn find_urls_expiring_soon(&self, _duration: chrono::Duration) -> Result<Vec<crate::domain::entities::Url>, crate::domain::repositories::RepositoryError> {
            let urls = self.urls.lock().unwrap();
            let now = chrono::Utc::now();
            let warning_time = now + _duration;
            
            let expiring_soon: Vec<crate::domain::entities::Url> = urls.iter()
                .filter(|url| {
                    if let Some(expiration) = url.expiration_date {
                        now < expiration && expiration <= warning_time
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
            
            Ok(expiring_soon)
        }

        async fn find_expired_urls(&self) -> Result<Vec<crate::domain::entities::Url>, crate::domain::repositories::RepositoryError> {
            let urls = self.urls.lock().unwrap();
            let expired: Vec<crate::domain::entities::Url> = urls.iter()
                .filter(|url| url.is_expired())
                .cloned()
                .collect();
            
            Ok(expired)
        }

        async fn delete_expired_urls(&self) -> Result<u64, crate::domain::repositories::RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let initial_count = urls.len();
            
            urls.retain(|url| !url.is_expired());
            
            let deleted_count = initial_count - urls.len();
            Ok(deleted_count as u64)
        }

        async fn soft_delete_by_id(&self, _id: i32, _user_id: Option<i32>) -> Result<bool, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn reactivate_by_id(&self, _id: i32, _user_id: Option<i32>) -> Result<bool, crate::domain::repositories::RepositoryError> {
            todo!()
        }

        async fn find_by_status(&self, _status: crate::domain::entities::UrlStatus, _user_id: Option<i32>) -> Result<Vec<crate::domain::entities::Url>, crate::domain::repositories::RepositoryError> {
            todo!()
        }
    }

    #[tokio::test]
    async fn test_cleanup_expired_urls() {
        let repo = MockUrlRepository::new();
        let service = CleanupService::new(repo);
        
        // Test with no expired URLs
        let deleted_count = service.cleanup_expired_urls().await.unwrap();
        assert_eq!(deleted_count, 0);
    }
}
