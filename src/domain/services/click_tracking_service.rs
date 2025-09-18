use crate::domain::entities::Click;
use crate::domain::repositories::{ClickRepository, ClickStats, ClickRepositoryError};
use tokio::sync::{mpsc, oneshot};
use tokio::task;

/// Click tracking information extracted from HTTP request
#[derive(Debug, Clone)]
pub struct ClickInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub country_code: Option<String>,
}

/// Click tracking service for recording and analyzing URL clicks
/// Uses async processing to avoid blocking URL resolution
#[derive(Clone)]
pub struct ClickTrackingService<R>
where
    R: ClickRepository + Clone + Send + Sync + 'static,
{
    repository: R,
    sender: mpsc::UnboundedSender<ClickTrackingTask>,
}

/// Internal task for async click processing
#[derive(Debug)]
enum ClickTrackingTask {
    RecordClick { url_id: i32, click_info: ClickInfo },
    GetStats { url_id: i32, response_sender: oneshot::Sender<Result<ClickStats, ClickTrackingError>> },
    GetUserStats { user_id: i32, response_sender: oneshot::Sender<Result<ClickStats, ClickTrackingError>> },
}

impl<R> ClickTrackingService<R>
where
    R: ClickRepository + Clone + Send + Sync + 'static,
{
    /// Create a new click tracking service
    pub fn new(repository: R) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        // Spawn background task for processing click events
        let repo_clone = repository.clone();
        task::spawn(async move {
            while let Some(task) = receiver.recv().await {
                match task {
                    ClickTrackingTask::RecordClick { url_id, click_info } => {
                        let click = Click::new_for_tracking(
                            url_id,
                            click_info.ip_address,
                            click_info.user_agent,
                            click_info.referer,
                            click_info.country_code,
                        );
                        
                        if let Err(e) = repo_clone.record_click(&click).await {
                            tracing::warn!("Failed to record click for URL {}: {}", url_id, e);
                        }
                    }
                    ClickTrackingTask::GetStats { url_id, response_sender } => {
                        let result = repo_clone.get_url_click_stats(url_id).await;
                        let _ = response_sender.send(result.map_err(ClickTrackingError::from));
                    }
                    ClickTrackingTask::GetUserStats { user_id, response_sender } => {
                        let result = repo_clone.get_user_click_stats(user_id).await;
                        let _ = response_sender.send(result.map_err(ClickTrackingError::from));
                    }
                }
            }
        });
        
        Self { repository, sender }
    }
    
    /// Record a click event asynchronously (non-blocking)
    pub fn record_click(&self, url_id: i32, click_info: ClickInfo) -> Result<(), ClickTrackingError> {
        self.sender
            .send(ClickTrackingTask::RecordClick { url_id, click_info })
            .map_err(|_| ClickTrackingError::ServiceUnavailable)?;
        Ok(())
    }
    
    /// Get click statistics for a URL
    pub async fn get_url_stats(&self, url_id: i32) -> Result<ClickStats, ClickTrackingError> {
        let (response_sender, response_receiver) = oneshot::channel();
        
        self.sender
            .send(ClickTrackingTask::GetStats { url_id, response_sender })
            .map_err(|_| ClickTrackingError::ServiceUnavailable)?;
        
        response_receiver.await
            .map_err(|_| ClickTrackingError::ServiceUnavailable)?
    }
    
    /// Get click statistics for a user
    pub async fn get_user_stats(&self, user_id: i32) -> Result<ClickStats, ClickTrackingError> {
        let (response_sender, response_receiver) = oneshot::channel();
        
        self.sender
            .send(ClickTrackingTask::GetUserStats { user_id, response_sender })
            .map_err(|_| ClickTrackingError::ServiceUnavailable)?;
        
        response_receiver.await
            .map_err(|_| ClickTrackingError::ServiceUnavailable)?
    }
    
    /// Get click count for a URL (synchronous, for response enhancement)
    pub async fn get_click_count(&self, url_id: i32) -> Result<i64, ClickTrackingError> {
        self.repository.get_click_count(url_id).await
            .map_err(ClickTrackingError::from)
    }
    
    /// Get clicks for a URL within a time range
    pub async fn get_clicks_for_url(
        &self,
        url_id: i32,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Click>, ClickTrackingError> {
        self.repository.get_clicks_for_url(url_id, start_date, end_date).await
            .map_err(ClickTrackingError::from)
    }
    
    /// Get clicks for a user within a time range
    pub async fn get_clicks_for_user(
        &self,
        user_id: i32,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Click>, ClickTrackingError> {
        self.repository.get_clicks_for_user(user_id, start_date, end_date).await
            .map_err(ClickTrackingError::from)
    }
}

/// Click tracking service errors
#[derive(Debug, thiserror::Error)]
pub enum ClickTrackingError {
    #[error("Repository error: {0}")]
    Repository(#[from] ClickRepositoryError),
    
    #[error("Service unavailable")]
    ServiceUnavailable,
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::ClickRepository;
    use std::sync::{Arc, Mutex};

    // Mock repository for testing
    #[derive(Clone)]
    struct MockClickRepository {
        clicks: Arc<Mutex<Vec<Click>>>,
    }

    impl MockClickRepository {
        fn new() -> Self {
            Self {
                clicks: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl ClickRepository for MockClickRepository {
        async fn record_click(&self, click: &Click) -> Result<Click, ClickRepositoryError> {
            let mut clicks = self.clicks.lock().unwrap();
            let mut new_click = click.clone();
            new_click.id = (clicks.len() + 1) as i32;
            clicks.push(new_click.clone());
            Ok(new_click)
        }

        async fn get_click_count(&self, url_id: i32) -> Result<i64, ClickRepositoryError> {
            let clicks = self.clicks.lock().unwrap();
            Ok(clicks.iter().filter(|c| c.url_id == url_id).count() as i64)
        }

        async fn get_clicks_for_url(
            &self,
            url_id: i32,
            _start_date: Option<chrono::DateTime<chrono::Utc>>,
            _end_date: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<Vec<Click>, ClickRepositoryError> {
            let clicks = self.clicks.lock().unwrap();
            Ok(clicks.iter().filter(|c| c.url_id == url_id).cloned().collect())
        }

        async fn get_clicks_for_user(
            &self,
            _user_id: i32,
            _start_date: Option<chrono::DateTime<chrono::Utc>>,
            _end_date: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<Vec<Click>, ClickRepositoryError> {
            Ok(vec![])
        }

        async fn get_url_click_stats(&self, url_id: i32) -> Result<ClickStats, ClickRepositoryError> {
            let clicks = self.clicks.lock().unwrap();
            let url_clicks: Vec<_> = clicks.iter().filter(|c| c.url_id == url_id).collect();
            
            Ok(ClickStats {
                total_clicks: url_clicks.len() as i64,
                unique_ips: 1,
                clicks_today: url_clicks.len() as i64,
                clicks_this_week: url_clicks.len() as i64,
                clicks_this_month: url_clicks.len() as i64,
                top_countries: vec![],
                top_referers: vec![],
            })
        }

        async fn get_user_click_stats(&self, _user_id: i32) -> Result<ClickStats, ClickRepositoryError> {
            Ok(ClickStats {
                total_clicks: 0,
                unique_ips: 0,
                clicks_today: 0,
                clicks_this_week: 0,
                clicks_this_month: 0,
                top_countries: vec![],
                top_referers: vec![],
            })
        }

        async fn delete_old_clicks(&self, _older_than: chrono::DateTime<chrono::Utc>) -> Result<u64, ClickRepositoryError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_record_click() {
        let repo = MockClickRepository::new();
        let service = ClickTrackingService::new(repo);
        
        let click_info = ClickInfo {
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0...".to_string()),
            referer: Some("https://google.com".to_string()),
            country_code: Some("US".to_string()),
        };
        
        // Record click (non-blocking)
        service.record_click(42, click_info).unwrap();
        
        // Give async task time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Check click count
        let count = service.get_click_count(42).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_url_stats() {
        let repo = MockClickRepository::new();
        let service = ClickTrackingService::new(repo);
        
        let stats = service.get_url_stats(42).await.unwrap();
        assert_eq!(stats.total_clicks, 0);
    }
}
