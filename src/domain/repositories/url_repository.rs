use crate::domain::entities::{ShortCode, Url, UrlStatus};
use async_trait::async_trait;

/// Repository trait for URL operations
/// This defines the contract for URL data access without depending on specific implementations
#[async_trait]
#[allow(dead_code)]
pub trait UrlRepository: Send + Sync {
    /// Create a new URL record
    async fn create_url(
        &self,
        short_code: &ShortCode,
        original_url: &str,
        expiration_date: Option<chrono::DateTime<chrono::Utc>>,
        user_id: Option<i32>,
        status: UrlStatus,
    ) -> Result<Url, RepositoryError>;

    /// Find a URL by short code
    async fn find_by_short_code(
        &self,
        short_code: &ShortCode,
    ) -> Result<Option<Url>, RepositoryError>;

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

    /// Find URLs that are expiring soon
    async fn find_urls_expiring_soon(
        &self,
        duration: chrono::Duration,
    ) -> Result<Vec<Url>, RepositoryError>;

    /// Find expired URLs
    async fn find_expired_urls(&self) -> Result<Vec<Url>, RepositoryError>;

    /// Delete expired URLs
    async fn delete_expired_urls(&self) -> Result<u64, RepositoryError>;

    /// Soft delete a URL by setting status to inactive
    async fn soft_delete_by_id(
        &self,
        id: i32,
        user_id: Option<i32>,
    ) -> Result<bool, RepositoryError>;

    /// Reactivate a URL by setting status to active
    async fn reactivate_by_id(
        &self,
        id: i32,
        user_id: Option<i32>,
    ) -> Result<bool, RepositoryError>;

    /// Find URLs by status
    async fn find_by_status(
        &self,
        status: UrlStatus,
        user_id: Option<i32>,
    ) -> Result<Vec<Url>, RepositoryError>;

    /// Batch deactivate URLs by IDs
    async fn batch_deactivate_urls(
        &self,
        url_ids: &[i32],
        user_id: Option<i32>,
    ) -> Result<BatchOperationResult, RepositoryError>;

    /// Batch reactivate URLs by IDs
    async fn batch_reactivate_urls(
        &self,
        url_ids: &[i32],
        user_id: Option<i32>,
    ) -> Result<BatchOperationResult, RepositoryError>;

    /// Batch delete URLs by IDs
    async fn batch_delete_urls(
        &self,
        url_ids: &[i32],
        user_id: Option<i32>,
    ) -> Result<BatchOperationResult, RepositoryError>;

    /// Batch update URL status
    async fn batch_update_status(
        &self,
        url_ids: &[i32],
        status: UrlStatus,
        user_id: Option<i32>,
    ) -> Result<BatchOperationResult, RepositoryError>;

    /// Batch update URL expiration dates
    async fn batch_update_expiration(
        &self,
        url_ids: &[i32],
        expiration_date: Option<chrono::DateTime<chrono::Utc>>,
        user_id: Option<i32>,
    ) -> Result<BatchOperationResult, RepositoryError>;
}

/// Statistics about URLs  
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UrlStats {
    pub total_urls: i64,
    pub total_clicks: i64,
    pub unique_short_codes: i64,
}

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct BatchOperationResult {
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult>,
}

/// Individual result for a batch operation item
#[derive(Debug, Clone)]
pub struct BatchItemResult {
    pub url_id: i32,
    pub success: bool,
    pub error: Option<String>,
}

/// Repository errors
#[allow(dead_code)]
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

#[cfg(test)]
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
            expiration_date: Option<chrono::DateTime<chrono::Utc>>,
            user_id: Option<i32>,
            status: UrlStatus,
        ) -> Result<Url, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let id = (urls.len() + 1) as i32;
            let url = Url::new_with_timestamp(
                id,
                short_code.value().to_string(),
                original_url.to_string(),
                expiration_date,
                user_id,
                status,
            );
            urls.push(url.clone());
            Ok(url)
        }

        async fn find_by_short_code(
            &self,
            short_code: &ShortCode,
        ) -> Result<Option<Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            Ok(urls
                .iter()
                .find(|u| u.short_code == short_code.value())
                .cloned())
        }

        async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            Ok(urls
                .iter()
                .filter(|u| u.user_id == Some(user_id))
                .cloned()
                .collect())
        }

        async fn exists_by_short_code(
            &self,
            short_code: &ShortCode,
        ) -> Result<bool, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            Ok(urls.iter().any(|u| u.short_code == short_code.value()))
        }

        async fn delete_by_id(
            &self,
            id: i32,
            user_id: Option<i32>,
        ) -> Result<bool, RepositoryError> {
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

        async fn find_urls_expiring_soon(
            &self,
            duration: chrono::Duration,
        ) -> Result<Vec<Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            let now = chrono::Utc::now();
            let warning_time = now + duration;

            let expiring_soon: Vec<Url> = urls
                .iter()
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

        async fn find_expired_urls(&self) -> Result<Vec<Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();

            let expired: Vec<Url> = urls
                .iter()
                .filter(|url| url.is_expired())
                .cloned()
                .collect();

            Ok(expired)
        }

        async fn delete_expired_urls(&self) -> Result<u64, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let initial_count = urls.len();

            urls.retain(|url| !url.is_expired());

            let deleted_count = initial_count - urls.len();
            Ok(deleted_count as u64)
        }

        async fn soft_delete_by_id(
            &self,
            id: i32,
            user_id: Option<i32>,
        ) -> Result<bool, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            if let Some(url) = urls.iter_mut().find(|u| u.id == id && u.user_id == user_id) {
                url.deactivate();
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn reactivate_by_id(
            &self,
            id: i32,
            user_id: Option<i32>,
        ) -> Result<bool, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            if let Some(url) = urls.iter_mut().find(|u| u.id == id && u.user_id == user_id) {
                url.reactivate();
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn find_by_status(
            &self,
            status: UrlStatus,
            user_id: Option<i32>,
        ) -> Result<Vec<Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            let filtered_urls: Vec<Url> = urls
                .iter()
                .filter(|url| url.status == status && (user_id.is_none() || url.user_id == user_id))
                .cloned()
                .collect();
            Ok(filtered_urls)
        }

        async fn batch_deactivate_urls(
            &self,
            url_ids: &[i32],
            user_id: Option<i32>,
        ) -> Result<BatchOperationResult, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();
            let mut successful = 0;
            let mut failed = 0;

            for &url_id in url_ids {
                if let Some(url) = urls
                    .iter_mut()
                    .find(|u| u.id == url_id && (user_id.is_none() || u.user_id == user_id))
                {
                    url.deactivate();
                    results.push(BatchItemResult {
                        url_id,
                        success: true,
                        error: None,
                    });
                    successful += 1;
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or permission denied".to_string()),
                    });
                    failed += 1;
                }
            }

            Ok(BatchOperationResult {
                total_processed: url_ids.len(),
                successful,
                failed,
                results,
            })
        }

        async fn batch_reactivate_urls(
            &self,
            url_ids: &[i32],
            user_id: Option<i32>,
        ) -> Result<BatchOperationResult, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();
            let mut successful = 0;
            let mut failed = 0;

            for &url_id in url_ids {
                if let Some(url) = urls
                    .iter_mut()
                    .find(|u| u.id == url_id && (user_id.is_none() || u.user_id == user_id))
                {
                    url.reactivate();
                    results.push(BatchItemResult {
                        url_id,
                        success: true,
                        error: None,
                    });
                    successful += 1;
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or permission denied".to_string()),
                    });
                    failed += 1;
                }
            }

            Ok(BatchOperationResult {
                total_processed: url_ids.len(),
                successful,
                failed,
                results,
            })
        }

        async fn batch_delete_urls(
            &self,
            url_ids: &[i32],
            user_id: Option<i32>,
        ) -> Result<BatchOperationResult, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();
            let mut successful = 0;
            let mut failed = 0;

            for &url_id in url_ids {
                if let Some(pos) = urls
                    .iter()
                    .position(|u| u.id == url_id && (user_id.is_none() || u.user_id == user_id))
                {
                    urls.remove(pos);
                    results.push(BatchItemResult {
                        url_id,
                        success: true,
                        error: None,
                    });
                    successful += 1;
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or permission denied".to_string()),
                    });
                    failed += 1;
                }
            }

            Ok(BatchOperationResult {
                total_processed: url_ids.len(),
                successful,
                failed,
                results,
            })
        }

        async fn batch_update_status(
            &self,
            url_ids: &[i32],
            status: UrlStatus,
            user_id: Option<i32>,
        ) -> Result<BatchOperationResult, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();
            let mut successful = 0;
            let mut failed = 0;

            for &url_id in url_ids {
                if let Some(url) = urls
                    .iter_mut()
                    .find(|u| u.id == url_id && (user_id.is_none() || u.user_id == user_id))
                {
                    url.status = status;
                    results.push(BatchItemResult {
                        url_id,
                        success: true,
                        error: None,
                    });
                    successful += 1;
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or permission denied".to_string()),
                    });
                    failed += 1;
                }
            }

            Ok(BatchOperationResult {
                total_processed: url_ids.len(),
                successful,
                failed,
                results,
            })
        }

        async fn batch_update_expiration(
            &self,
            url_ids: &[i32],
            expiration_date: Option<chrono::DateTime<chrono::Utc>>,
            user_id: Option<i32>,
        ) -> Result<BatchOperationResult, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();
            let mut successful = 0;
            let mut failed = 0;

            for &url_id in url_ids {
                if let Some(url) = urls
                    .iter_mut()
                    .find(|u| u.id == url_id && (user_id.is_none() || u.user_id == user_id))
                {
                    url.expiration_date = expiration_date;
                    results.push(BatchItemResult {
                        url_id,
                        success: true,
                        error: None,
                    });
                    successful += 1;
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or permission denied".to_string()),
                    });
                    failed += 1;
                }
            }

            Ok(BatchOperationResult {
                total_processed: url_ids.len(),
                successful,
                failed,
                results,
            })
        }
    }

    #[tokio::test]
    async fn test_mock_repository_create_and_find() {
        let repo = MockUrlRepository::new();
        let short_code = ShortCode::new("abc123".to_string()).unwrap();

        let url = repo
            .create_url(
                &short_code,
                "https://example.com",
                None,
                None,
                UrlStatus::Active,
            )
            .await
            .unwrap();
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

        repo.create_url(
            &short_code,
            "https://example.com",
            None,
            None,
            UrlStatus::Active,
        )
        .await
        .unwrap();

        assert!(repo.exists_by_short_code(&short_code).await.unwrap());
    }
}
