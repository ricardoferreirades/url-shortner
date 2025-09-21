// Test utilities for integration tests
use crate::domain::entities::{ShortCode, Url, UrlStatus};
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
    async fn create_url(&self, short_code: &ShortCode, original_url: &str, expiration_date: Option<chrono::DateTime<chrono::Utc>>, user_id: Option<i32>, status: UrlStatus) -> Result<Url, RepositoryError> {
        let url = Url::new_with_timestamp(
            (self.urls.lock().unwrap().len() + 1) as i32,
            short_code.value().to_string(),
            original_url.to_string(),
            expiration_date,
            user_id,
            status,
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

    async fn find_urls_expiring_soon(&self, _duration: chrono::Duration) -> Result<Vec<Url>, RepositoryError> {
        Ok(vec![])
    }

    async fn find_expired_urls(&self) -> Result<Vec<Url>, RepositoryError> {
        Ok(vec![])
    }

    async fn delete_expired_urls(&self) -> Result<u64, RepositoryError> {
        Ok(0)
    }

    async fn soft_delete_by_id(&self, id: i32, user_id: Option<i32>) -> Result<bool, RepositoryError> {
        let mut urls = self.urls.lock().unwrap();
        if let Some(url) = urls.iter_mut().find(|u| u.id == id && u.user_id == user_id) {
            url.deactivate();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn reactivate_by_id(&self, id: i32, user_id: Option<i32>) -> Result<bool, RepositoryError> {
        let mut urls = self.urls.lock().unwrap();
        if let Some(url) = urls.iter_mut().find(|u| u.id == id && u.user_id == user_id) {
            url.reactivate();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn find_by_status(&self, status: UrlStatus, user_id: Option<i32>) -> Result<Vec<Url>, RepositoryError> {
        let urls = self.urls.lock().unwrap();
        let filtered_urls: Vec<Url> = urls.iter()
            .filter(|url| {
                url.status == status && 
                (user_id.is_none() || url.user_id == user_id)
            })
            .cloned()
            .collect();
        Ok(filtered_urls)
    }
}
