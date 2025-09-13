use crate::domain::entities::{ShortCode, Url};
use crate::domain::repositories::{RepositoryError, UrlRepository};
use seahash::SeaHasher;
use std::hash::{Hash, Hasher};

/// Domain service for URL operations
/// Contains business logic that doesn't belong to a specific entity
#[derive(Clone)]
pub struct UrlService<R>
where
    R: UrlRepository + Clone,
{
    repository: R,
}

impl<R> UrlService<R>
where
    R: UrlRepository + Clone,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Generate a unique short code for a URL
    pub async fn generate_short_code(&self, original_url: &str) -> Result<ShortCode, ServiceError> {
        // Start with a hash-based approach
        let mut hasher = SeaHasher::new();
        original_url.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert to base62-like encoding
        let short_code = self.hash_to_short_code(hash);
        let short_code = ShortCode::new(short_code)?;
        
        // Check if it already exists, if so, append a suffix
        if self.repository.exists_by_short_code(&short_code).await? {
            self.generate_unique_short_code(&short_code).await
        } else {
            Ok(short_code)
        }
    }

    /// Generate a unique short code with collision handling
    async fn generate_unique_short_code(&self, base_code: &ShortCode) -> Result<ShortCode, ServiceError> {
        let mut counter = 1;
        let base_value = base_code.value();
        
        loop {
            let candidate = if counter < 10 {
                format!("{}{}", base_value, counter)
            } else {
                // Use a more sophisticated approach for higher numbers
                let mut hasher = SeaHasher::new();
                (base_value, counter).hash(&mut hasher);
                let hash = hasher.finish();
                self.hash_to_short_code(hash)
            };
            
            let candidate_code = ShortCode::new(candidate)?;
            
            if !self.repository.exists_by_short_code(&candidate_code).await? {
                return Ok(candidate_code);
            }
            
            counter += 1;
            if counter > 1000 {
                return Err(ServiceError::TooManyCollisions);
            }
        }
    }

    /// Convert a hash to a short code string
    fn hash_to_short_code(&self, hash: u64) -> String {
        const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let mut result = String::new();
        let mut value = hash;
        
        // Generate 6-8 character code
        for _ in 0..6 {
            result.push(CHARS[(value % 62) as usize] as char);
            value /= 62;
        }
        
        result
    }

    /// Create a URL with auto-generated short code
    pub async fn create_url(
        &self,
        original_url: &str,
        custom_short_code: Option<ShortCode>,
        user_id: Option<i32>,
    ) -> Result<Url, ServiceError> {
        let short_code = match custom_short_code {
            Some(code) => {
                // Validate custom short code
                if self.repository.exists_by_short_code(&code).await? {
                    return Err(ServiceError::ShortCodeAlreadyExists);
                }
                code
            }
            None => self.generate_short_code(original_url).await?,
        };

        self.repository.create_url(&short_code, original_url, user_id).await
            .map_err(ServiceError::from)
    }

    /// Get URL by short code
    pub async fn get_url_by_short_code(&self, short_code: &ShortCode) -> Result<Option<Url>, ServiceError> {
        self.repository.find_by_short_code(short_code).await
            .map_err(ServiceError::from)
    }

    /// Get URLs for a specific user
    pub async fn get_urls_for_user(&self, user_id: i32) -> Result<Vec<Url>, ServiceError> {
        self.repository.find_by_user_id(user_id).await
            .map_err(ServiceError::from)
    }

    /// Delete a URL (with ownership check)
    pub async fn delete_url(&self, id: i32, user_id: Option<i32>) -> Result<bool, ServiceError> {
        self.repository.delete_by_id(id, user_id).await
            .map_err(ServiceError::from)
    }

    /// Update a URL (with ownership check)
    pub async fn update_url(&self, url: &Url) -> Result<Url, ServiceError> {
        self.repository.update_url(url).await
            .map_err(ServiceError::from)
    }
}

/// Service errors
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    
    #[error("Invalid short code: {0}")]
    InvalidShortCode(String),
    
    #[error("Short code already exists")]
    ShortCodeAlreadyExists,
    
    #[error("Too many collisions while generating short code")]
    TooManyCollisions,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

impl From<crate::domain::entities::ShortCodeError> for ServiceError {
    fn from(err: crate::domain::entities::ShortCodeError) -> Self {
        ServiceError::InvalidShortCode(err.to_string())
    }
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
        urls: Arc<Mutex<Vec<Url>>>,
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

        async fn get_stats(&self, user_id: Option<i32>) -> Result<crate::domain::repositories::UrlStats, RepositoryError> {
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
    }

    #[tokio::test]
    async fn test_generate_short_code() {
        let repo = MockUrlRepository::new();
        let service = UrlService::new(repo);
        
        let short_code = service.generate_short_code("https://example.com").await.unwrap();
        assert!(!short_code.value().is_empty());
        assert!(short_code.value().len() >= 6);
    }

    #[tokio::test]
    async fn test_create_url_with_generated_code() {
        let repo = MockUrlRepository::new();
        let service = UrlService::new(repo);
        
        let url = service.create_url("https://example.com", None, None).await.unwrap();
        assert_eq!(url.original_url, "https://example.com");
        assert!(!url.short_code.is_empty());
    }

    #[tokio::test]
    async fn test_create_url_with_custom_code() {
        let repo = MockUrlRepository::new();
        let service = UrlService::new(repo);
        let custom_code = ShortCode::new("mycode".to_string()).unwrap();
        
        let url = service.create_url("https://example.com", Some(custom_code.clone()), None).await.unwrap();
        assert_eq!(url.short_code, custom_code.value());
        assert_eq!(url.original_url, "https://example.com");
    }
}
