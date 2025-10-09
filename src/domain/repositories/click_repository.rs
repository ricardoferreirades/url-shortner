use crate::domain::entities::Click;
use async_trait::async_trait;

/// Repository trait for click/analytics data operations
#[async_trait]
#[allow(dead_code)]
pub trait ClickRepository: Send + Sync {
    /// Record a new click event
    async fn record_click(&self, click: &Click) -> Result<Click, RepositoryError>;
    
    /// Get click count for a specific URL
    async fn get_click_count(&self, url_id: i32) -> Result<i64, RepositoryError>;
    
    /// Get clicks for a specific URL within a time range
    async fn get_clicks_for_url(
        &self,
        url_id: i32,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Click>, RepositoryError>;
    
    /// Get clicks for a specific user within a time range
    async fn get_clicks_for_user(
        &self,
        user_id: i32,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Click>, RepositoryError>;
    
    /// Get click statistics for a URL
    async fn get_url_click_stats(&self, url_id: i32) -> Result<ClickStats, RepositoryError>;
    
    /// Get click statistics for a user
    async fn get_user_click_stats(&self, user_id: i32) -> Result<ClickStats, RepositoryError>;
    
    /// Delete old click records (for data retention)
    async fn delete_old_clicks(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<u64, RepositoryError>;
}

/// Click statistics data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClickStats {
    pub total_clicks: i64,
    pub unique_ips: i64,
    pub clicks_today: i64,
    pub clicks_this_week: i64,
    pub clicks_this_month: i64,
    pub top_countries: Vec<(String, i64)>,
    pub top_referers: Vec<(String, i64)>,
}

/// Repository errors
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Permission denied")]
    PermissionDenied,
}

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        RepositoryError::Database(err.to_string())
    }
}
