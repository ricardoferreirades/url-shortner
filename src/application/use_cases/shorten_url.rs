use crate::application::dto::{requests::ShortenUrlRequest, responses::ShortenUrlResponse};
use crate::domain::entities::ShortCode;
use crate::domain::repositories::UrlRepository;
use crate::domain::services::{ServiceError, UrlService};

/// Use case for shortening URLs
#[derive(Clone)]
pub struct ShortenUrlUseCase<R>
where
    R: UrlRepository + Clone,
{
    url_service: UrlService<R>,
    base_url: String,
}

impl<R> ShortenUrlUseCase<R>
where
    R: UrlRepository + Clone,
{
    pub fn new(url_service: UrlService<R>, base_url: String) -> Self {
        Self {
            url_service,
            base_url,
        }
    }

    /// Execute the shorten URL use case
    pub async fn execute(
        &self,
        request: ShortenUrlRequest,
        user_id: Option<i32>,
    ) -> Result<ShortenUrlResponse, UseCaseError> {
        // Validate the input URL
        self.validate_url(&request.url)?;

        // Create custom short code if provided
        let custom_short_code = if let Some(code_str) = request.custom_short_code {
            Some(
                ShortCode::new(code_str)
                    .map_err(|e| UseCaseError::InvalidShortCode(e.to_string()))?,
            )
        } else {
            None
        };

        // Create the URL using the domain service
        let url = self
            .url_service
            .create_url(
                &request.url,
                custom_short_code,
                request.expiration_date,
                user_id,
            )
            .await
            .map_err(UseCaseError::Service)?;

        // Convert to response DTO
        Ok(ShortenUrlResponse {
            short_url: url.short_url(&self.base_url),
            original_url: url.original_url,
            short_code: url.short_code,
            created_at: url.created_at.to_rfc3339(),
            expiration_date: url.expiration_date.map(|d| d.to_rfc3339()),
        })
    }

    /// Validate URL format
    fn validate_url(&self, url: &str) -> Result<(), UseCaseError> {
        if url.is_empty() {
            return Err(UseCaseError::Validation("URL cannot be empty".to_string()));
        }

        if url.len() > 2048 {
            return Err(UseCaseError::Validation(
                "URL is too long (max 2048 characters)".to_string(),
            ));
        }

        // Basic URL format validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(UseCaseError::Validation(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        Ok(())
    }
}

/// Use case errors
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    #[error("Invalid short code: {0}")]
    InvalidShortCode(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::UrlStatus,
        repositories::{RepositoryError, UrlRepository},
    };
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
            short_code: &ShortCode,
            original_url: &str,
            expiration_date: Option<chrono::DateTime<chrono::Utc>>,
            user_id: Option<i32>,
            status: crate::domain::entities::UrlStatus,
        ) -> Result<crate::domain::entities::Url, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            let id = (urls.len() + 1) as i32;
            let url = crate::domain::entities::Url::new_with_timestamp(
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
        ) -> Result<Option<crate::domain::entities::Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            Ok(urls
                .iter()
                .find(|u| u.short_code == short_code.value())
                .cloned())
        }

        async fn find_by_user_id(
            &self,
            user_id: i32,
        ) -> Result<Vec<crate::domain::entities::Url>, RepositoryError> {
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

        async fn update_url(
            &self,
            url: &crate::domain::entities::Url,
        ) -> Result<crate::domain::entities::Url, RepositoryError> {
            let mut urls = self.urls.lock().unwrap();
            if let Some(existing) = urls.iter_mut().find(|u| u.id == url.id) {
                *existing = url.clone();
                Ok(existing.clone())
            } else {
                Err(RepositoryError::NotFound)
            }
        }

        async fn get_stats(
            &self,
            user_id: Option<i32>,
        ) -> Result<crate::domain::repositories::UrlStats, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            let filtered_urls: Vec<_> = if let Some(uid) = user_id {
                urls.iter().filter(|u| u.user_id == Some(uid)).collect()
            } else {
                urls.iter().collect()
            };

            Ok(crate::domain::repositories::UrlStats {
                total_urls: filtered_urls.len() as i64,
                total_clicks: 0,
                unique_short_codes: filtered_urls.len() as i64,
            })
        }

        async fn find_urls_expiring_soon(
            &self,
            _duration: chrono::Duration,
        ) -> Result<Vec<crate::domain::entities::Url>, RepositoryError> {
            Ok(vec![])
        }

        async fn find_expired_urls(
            &self,
        ) -> Result<Vec<crate::domain::entities::Url>, RepositoryError> {
            Ok(vec![])
        }

        async fn delete_expired_urls(&self) -> Result<u64, RepositoryError> {
            Ok(0)
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
            status: crate::domain::entities::UrlStatus,
            user_id: Option<i32>,
        ) -> Result<Vec<crate::domain::entities::Url>, RepositoryError> {
            let urls = self.urls.lock().unwrap();
            let filtered_urls: Vec<crate::domain::entities::Url> = urls
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
        ) -> Result<crate::domain::repositories::url_repository::BatchOperationResult, RepositoryError>
        {
            self.batch_update_status(url_ids, UrlStatus::Inactive, user_id)
                .await
        }

        async fn batch_reactivate_urls(
            &self,
            url_ids: &[i32],
            user_id: Option<i32>,
        ) -> Result<crate::domain::repositories::url_repository::BatchOperationResult, RepositoryError>
        {
            self.batch_update_status(url_ids, UrlStatus::Active, user_id)
                .await
        }

        async fn batch_delete_urls(
            &self,
            url_ids: &[i32],
            user_id: Option<i32>,
        ) -> Result<crate::domain::repositories::url_repository::BatchOperationResult, RepositoryError>
        {
            use crate::domain::repositories::url_repository::{
                BatchItemResult, BatchOperationResult,
            };

            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();

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
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or unauthorized".to_string()),
                    });
                }
            }

            let successful = results.iter().filter(|r| r.success).count();
            let failed = results.len() - successful;

            Ok(BatchOperationResult {
                total_processed: results.len(),
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
        ) -> Result<crate::domain::repositories::url_repository::BatchOperationResult, RepositoryError>
        {
            use crate::domain::repositories::url_repository::{
                BatchItemResult, BatchOperationResult,
            };

            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();

            for &url_id in url_ids {
                if let Some(url) = urls
                    .iter_mut()
                    .find(|u| u.id == url_id && (user_id.is_none() || u.user_id == user_id))
                {
                    url.status = status.clone();
                    results.push(BatchItemResult {
                        url_id,
                        success: true,
                        error: None,
                    });
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or unauthorized".to_string()),
                    });
                }
            }

            let successful = results.iter().filter(|r| r.success).count();
            let failed = results.len() - successful;

            Ok(BatchOperationResult {
                total_processed: results.len(),
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
        ) -> Result<crate::domain::repositories::url_repository::BatchOperationResult, RepositoryError>
        {
            use crate::domain::repositories::url_repository::{
                BatchItemResult, BatchOperationResult,
            };

            let mut urls = self.urls.lock().unwrap();
            let mut results = Vec::new();

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
                } else {
                    results.push(BatchItemResult {
                        url_id,
                        success: false,
                        error: Some("URL not found or unauthorized".to_string()),
                    });
                }
            }

            let successful = results.iter().filter(|r| r.success).count();
            let failed = results.len() - successful;

            Ok(BatchOperationResult {
                total_processed: results.len(),
                successful,
                failed,
                results,
            })
        }
    }

    #[tokio::test]
    async fn test_shorten_url_success() {
        let repo = MockUrlRepository::new();
        let url_service = UrlService::new(repo);
        let use_case = ShortenUrlUseCase::new(url_service, "https://short.ly".to_string());

        let request = ShortenUrlRequest {
            url: "https://example.com".to_string(),
            custom_short_code: None,
            expiration_date: None,
        };

        let response = use_case.execute(request, None).await.unwrap();
        assert_eq!(response.original_url, "https://example.com");
        assert!(!response.short_code.is_empty());
        assert!(response.short_url.starts_with("https://short.ly/"));
    }

    #[tokio::test]
    async fn test_shorten_url_with_custom_code() {
        let repo = MockUrlRepository::new();
        let url_service = UrlService::new(repo);
        let use_case = ShortenUrlUseCase::new(url_service, "https://short.ly".to_string());

        let request = ShortenUrlRequest {
            url: "https://example.com".to_string(),
            custom_short_code: Some("mycode".to_string()),
            expiration_date: None,
        };

        let response = use_case.execute(request, None).await.unwrap();
        assert_eq!(response.short_code, "mycode");
        assert_eq!(response.short_url, "https://short.ly/mycode");
    }

    #[tokio::test]
    async fn test_shorten_url_validation_empty() {
        let repo = MockUrlRepository::new();
        let url_service = UrlService::new(repo);
        let use_case = ShortenUrlUseCase::new(url_service, "https://short.ly".to_string());

        let request = ShortenUrlRequest {
            url: "".to_string(),
            custom_short_code: None,
            expiration_date: None,
        };

        let result = use_case.execute(request, None).await;
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn test_shorten_url_validation_invalid_protocol() {
        let repo = MockUrlRepository::new();
        let url_service = UrlService::new(repo);
        let use_case = ShortenUrlUseCase::new(url_service, "https://short.ly".to_string());

        let request = ShortenUrlRequest {
            url: "ftp://example.com".to_string(),
            custom_short_code: None,
            expiration_date: None,
        };

        let result = use_case.execute(request, None).await;
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}
