use crate::domain::entities::{ShortCode, Url};
use async_trait::async_trait;

/// Repository trait for URL operations
/// This defines the contract for URL data access without depending on specific implementations
#[async_trait]
pub trait UrlRepository: Send + Sync {
    /// Create a new URL record
    async fn create_url(
        &self,
        short_code: &ShortCode,
        original_url: &str,
        user_id: Option<i32>,
    ) -> Result<Url, RepositoryError>;

    /// Find a URL by short code
    async fn find_by_short_code(&self, short_code: &ShortCode) -> Result<Option<Url>, RepositoryError>;

    /// Find URLs by user ID
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Url>, RepositoryError>;

    /// Check if a short code already exists
    async fn exists_by_short_code(&self, short_code: &ShortCode) -> Result<bool, RepositoryError>;

    /// Delete a URL by ID
    async fn delete_by_id(&self, id: i32, user_id: Option<i32>) -> Result<bool, RepositoryError>;

    /// Update a URL
    async fn update_url(&self, url: &Url) -> Result<Url, RepositoryError>;

    /// Get URL statistics
    async fn get_stats(&self, user_id: Option<i32>) -> Result<UrlStats, RepositoryError>;
}

/// Statistics about URLs
#[derive(Debug, Clone)]
pub struct UrlStats {
    pub total_urls: i64,
    pub total_clicks: i64,
    pub unique_short_codes: i64,
}

/// Repository errors
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    
    #[error("URL not found")]
    NotFound,
    
    #[error("Short code already exists")]
    DuplicateShortCode,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use super::*;
    use crate::domain::entities::ShortCode;

    // Mock repository for testing
    struct MockUrlRepository {
        urls: std::sync::Mutex<Vec<Url>>,
    }

    impl MockUrlRepository {
        fn new() -> Self {
            Self {
                urls: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl UrlRepository for MockUrlRepository {
        async fn create_url(
            &self,
            short_code: &ShortCode,
            original_url: &str,
            user_id: Option<i32>,
        ) -> Result<Url, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let id = (urls.len() + 1) as i32;
            let url = Url::new_with_timestamp(
                id,
                short_code.value().to_string(),
                original_url.to_string(),
                user_id,
            );
            urls.push(url.clone());
            Ok(url)
        }

        async fn find_by_short_code(&self, short_code: &ShortCode) -> Result<Option<Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            Ok(urls.iter().find(|u| u.short_code == short_code.value()).cloned())
        }

        async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            Ok(urls.iter().filter(|u| u.user_id == Some(user_id)).cloned().collect())
        }

        async fn exists_by_short_code(&self, short_code: &ShortCode) -> Result<bool, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            Ok(urls.iter().any(|u| u.short_code == short_code.value()))
        }

        async fn delete_by_id(&self, id: i32, user_id: Option<i32>) -> Result<bool, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            if let Some(pos) = urls.iter().position(|u| u.id == id && u.user_id == user_id) {
                urls.remove(pos);
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn update_url(&self, url: &Url) -> Result<Url, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            if let Some(existing) = urls.iter_mut().find(|u| u.id == url.id) {
                *existing = url.clone();
                Ok(existing.clone())
            } else {
                Err(RepositoryError::NotFound)
            }
        }

        async fn get_stats(&self, user_id: Option<i32>) -> Result<UrlStats, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            let filtered_urls: Vec<_> = if let Some(uid) = user_id {
                urls.iter().filter(|u| u.user_id == Some(uid)).collect()
            } else {
                urls.iter().collect()
            };
            
            Ok(UrlStats {
                total_urls: filtered_urls.len() as i64,
                total_clicks: 0, // Mock value
                unique_short_codes: filtered_urls.len() as i64,
            })
        }
    }

    #[tokio::test]
    async fn test_mock_repository_create_and_find() {
        let repo = MockUrlRepository::new();
        let short_code = ShortCode::new("abc123".to_string()).unwrap();
        
        let url = repo.create_url(&short_code, "https://example.com", None).await.unwrap();
        assert_eq!(url.short_code, "abc123");
        assert_eq!(url.original_url, "https://example.com");
        
        let found = repo.find_by_short_code(&short_code).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, url.id);
    }

    #[tokio::test]
    async fn test_mock_repository_exists_check() {
        let repo = MockUrlRepository::new();
        let short_code = ShortCode::new("abc123".to_string()).unwrap();
        
        assert!(!repo.exists_by_short_code(&short_code).await.unwrap());
        
        repo.create_url(&short_code, "https://example.com", None).await.unwrap();
        
        assert!(repo.exists_by_short_code(&short_code).await.unwrap());
    }
}
