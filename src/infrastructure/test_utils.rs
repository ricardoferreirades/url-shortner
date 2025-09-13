// Test utilities for integration tests
use crate::domain::entities::{ShortCode, Url};
use crate::domain::repositories::{RepositoryError, UrlRepository};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// Mock repository for testing
#[derive(Clone)]
pub struct MockUrlRepository {
    urls: Arc<Mutex<Vec<Url>>>,
}

impl MockUrlRepository {
    pub fn new() -> Self {
        Self {
            urls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl UrlRepository for MockUrlRepository {
    async fn create_url(&self, short_code: &ShortCode, original_url: &str, user_id: Option<i32>) -> Result<Url, RepositoryError> {
        let url = Url::new_with_timestamp(
            (self.urls.lock().unwrap().len() + 1) as i32,
            short_code.value().to_string(),
            original_url.to_string(),
            user_id,
        );
        let mut urls = self.urls.lock().unwrap();
        urls.push(url.clone());
        Ok(url)
    }

    async fn exists_by_short_code(&self, short_code: &ShortCode) -> Result<bool, RepositoryError> {
        let urls = self.urls.lock().unwrap();
        Ok(urls.iter().any(|url| url.short_code == short_code.value()))
    }

    async fn find_by_short_code(&self, short_code: &ShortCode) -> Result<Option<Url>, RepositoryError> {
        let urls = self.urls.lock().unwrap();
        Ok(urls.iter().find(|url| url.short_code == short_code.value()).cloned())
    }

    async fn find_by_user_id(&self, _user_id: i32) -> Result<Vec<Url>, RepositoryError> {
        let urls = self.urls.lock().unwrap();
        Ok(urls.clone())
    }

    async fn delete_by_id(&self, _id: i32, _user_id: Option<i32>) -> Result<bool, RepositoryError> {
        Ok(true)
    }

    async fn update_url(&self, url: &Url) -> Result<Url, RepositoryError> {
        Ok(url.clone())
    }

    async fn get_stats(&self, _user_id: Option<i32>) -> Result<crate::domain::repositories::UrlStats, RepositoryError> {
        Ok(crate::domain::repositories::UrlStats {
            total_urls: 0,
            total_clicks: 0,
            unique_short_codes: 0,
        })
    }
}
