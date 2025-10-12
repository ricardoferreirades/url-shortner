use crate::domain::entities::{ShortCode, Url, UrlStatus};
use crate::domain::repositories::{RepositoryError, UrlRepository, UrlStats};
use async_trait::async_trait;
use sqlx::{PgPool, Row};

/// PostgreSQL implementation of the UrlRepository trait
#[derive(Clone)]
pub struct PostgresUrlRepository {
    pool: PgPool,
}

impl PostgresUrlRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Helper function to convert string status to UrlStatus
    fn status_from_string(status: String) -> UrlStatus {
        match status.as_str() {
            "active" => UrlStatus::Active,
            "inactive" => UrlStatus::Inactive,
            _ => UrlStatus::Active, // Default fallback
        }
    }

    /// Helper function to create Url from database row
    fn url_from_row(row: &sqlx::postgres::PgRow) -> Url {
        Url {
            id: row.get("id"),
            short_code: row.get("short_code"),
            original_url: row.get("original_url"),
            created_at: row.get("created_at"),
            expiration_date: row.get("expiration_date"),
            user_id: row.get("user_id"),
            status: Self::status_from_string(row.get("status")),
        }
    }
}

#[async_trait]
impl UrlRepository for PostgresUrlRepository {
    async fn create_url(
        &self,
        short_code: &ShortCode,
        original_url: &str,
        expiration_date: Option<chrono::DateTime<chrono::Utc>>,
        user_id: Option<i32>,
        status: UrlStatus,
    ) -> Result<Url, RepositoryError> {
        let row = sqlx::query(
            "INSERT INTO urls (short_code, original_url, expiration_date, user_id, status) VALUES ($1, $2, $3, $4, $5) RETURNING id, short_code, original_url, created_at, expiration_date, user_id, status"
        )
        .bind(short_code.value())
        .bind(original_url)
        .bind(expiration_date)
        .bind(user_id)
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::url_from_row(&row))
    }

    async fn find_by_short_code(
        &self,
        short_code: &ShortCode,
    ) -> Result<Option<Url>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, short_code, original_url, created_at, expiration_date, user_id, status FROM urls WHERE short_code = $1"
        )
        .bind(short_code.value())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Self::url_from_row(&row))),
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Url>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT id, short_code, original_url, created_at, expiration_date, user_id, status FROM urls WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let urls = rows
            .into_iter()
            .map(|row| Self::url_from_row(&row))
            .collect();

        Ok(urls)
    }

    async fn exists_by_short_code(&self, short_code: &ShortCode) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM urls WHERE short_code = $1")
            .bind(short_code.value())
            .fetch_one(&self.pool)
            .await?;

        Ok(count > 0)
    }

    async fn delete_by_id(&self, id: i32, user_id: Option<i32>) -> Result<bool, RepositoryError> {
        let query = if let Some(uid) = user_id {
            sqlx::query("DELETE FROM urls WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(uid)
        } else {
            sqlx::query("DELETE FROM urls WHERE id = $1 AND user_id IS NULL").bind(id)
        };

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn update_url(&self, url: &Url) -> Result<Url, RepositoryError> {
        let row = sqlx::query(
            "UPDATE urls SET short_code = $1, original_url = $2, expiration_date = $3, status = $4 WHERE id = $5 RETURNING id, short_code, original_url, created_at, expiration_date, user_id, status"
        )
        .bind(&url.short_code)
        .bind(&url.original_url)
        .bind(&url.expiration_date)
        .bind(url.status.to_string())
        .bind(url.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::url_from_row(&row))
    }

    async fn get_stats(&self, user_id: Option<i32>) -> Result<UrlStats, RepositoryError> {
        let (total_urls, unique_short_codes) = if let Some(uid) = user_id {
            let row = sqlx::query(
                "SELECT COUNT(*) as total_urls, COUNT(DISTINCT short_code) as unique_short_codes FROM urls WHERE user_id = $1"
            )
            .bind(uid)
            .fetch_one(&self.pool)
            .await?;

            (row.get("total_urls"), row.get("unique_short_codes"))
        } else {
            let row = sqlx::query(
                "SELECT COUNT(*) as total_urls, COUNT(DISTINCT short_code) as unique_short_codes FROM urls"
            )
            .fetch_one(&self.pool)
            .await?;

            (row.get("total_urls"), row.get("unique_short_codes"))
        };

        // TODO: Add click tracking when analytics are implemented
        let total_clicks = 0;

        Ok(UrlStats {
            total_urls,
            total_clicks,
            unique_short_codes,
        })
    }

    async fn find_urls_expiring_soon(
        &self,
        duration: chrono::Duration,
    ) -> Result<Vec<Url>, RepositoryError> {
        let now = chrono::Utc::now();
        let warning_time = now + duration;

        let rows = sqlx::query(
            "SELECT id, short_code, original_url, created_at, expiration_date, user_id, status 
             FROM urls 
             WHERE expiration_date IS NOT NULL 
             AND expiration_date > $1 
             AND expiration_date <= $2 
             ORDER BY expiration_date ASC",
        )
        .bind(now)
        .bind(warning_time)
        .fetch_all(&self.pool)
        .await?;

        let urls = rows
            .into_iter()
            .map(|row| Self::url_from_row(&row))
            .collect();

        Ok(urls)
    }

    async fn find_expired_urls(&self) -> Result<Vec<Url>, RepositoryError> {
        let now = chrono::Utc::now();

        let rows = sqlx::query(
            "SELECT id, short_code, original_url, created_at, expiration_date, user_id, status 
             FROM urls 
             WHERE expiration_date IS NOT NULL 
             AND expiration_date <= $1 
             ORDER BY expiration_date ASC",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let urls = rows
            .into_iter()
            .map(|row| Self::url_from_row(&row))
            .collect();

        Ok(urls)
    }

    async fn delete_expired_urls(&self) -> Result<u64, RepositoryError> {
        let now = chrono::Utc::now();

        let result = sqlx::query(
            "DELETE FROM urls WHERE expiration_date IS NOT NULL AND expiration_date <= $1",
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn soft_delete_by_id(
        &self,
        id: i32,
        user_id: Option<i32>,
    ) -> Result<bool, RepositoryError> {
        let query = if let Some(uid) = user_id {
            sqlx::query("UPDATE urls SET status = 'inactive' WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(uid)
        } else {
            sqlx::query("UPDATE urls SET status = 'inactive' WHERE id = $1 AND user_id IS NULL")
                .bind(id)
        };

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn reactivate_by_id(
        &self,
        id: i32,
        user_id: Option<i32>,
    ) -> Result<bool, RepositoryError> {
        let query = if let Some(uid) = user_id {
            sqlx::query("UPDATE urls SET status = 'active' WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(uid)
        } else {
            sqlx::query("UPDATE urls SET status = 'active' WHERE id = $1 AND user_id IS NULL")
                .bind(id)
        };

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn find_by_status(
        &self,
        status: UrlStatus,
        user_id: Option<i32>,
    ) -> Result<Vec<Url>, RepositoryError> {
        let rows = if let Some(uid) = user_id {
            sqlx::query(
                "SELECT id, short_code, original_url, created_at, expiration_date, user_id, status 
                 FROM urls WHERE status = $1 AND user_id = $2 ORDER BY created_at DESC",
            )
            .bind(status.to_string())
            .bind(uid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT id, short_code, original_url, created_at, expiration_date, user_id, status 
                 FROM urls WHERE status = $1 ORDER BY created_at DESC",
            )
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await?
        };

        let urls = rows
            .into_iter()
            .map(|row| Self::url_from_row(&row))
            .collect();

        Ok(urls)
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
        use crate::domain::repositories::url_repository::{BatchItemResult, BatchOperationResult};

        let mut results = Vec::new();
        for &url_id in url_ids {
            let result = if let Some(uid) = user_id {
                sqlx::query("DELETE FROM urls WHERE id = $1 AND user_id = $2")
                    .bind(url_id)
                    .bind(uid)
                    .execute(&self.pool)
                    .await
            } else {
                sqlx::query("DELETE FROM urls WHERE id = $1")
                    .bind(url_id)
                    .execute(&self.pool)
                    .await
            };

            match result {
                Ok(res) if res.rows_affected() > 0 => results.push(BatchItemResult {
                    url_id,
                    success: true,
                    error: None,
                }),
                Ok(_) => results.push(BatchItemResult {
                    url_id,
                    success: false,
                    error: Some("URL not found or unauthorized".to_string()),
                }),
                Err(e) => results.push(BatchItemResult {
                    url_id,
                    success: false,
                    error: Some(e.to_string()),
                }),
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
        use crate::domain::repositories::url_repository::{BatchItemResult, BatchOperationResult};

        let mut results = Vec::new();
        for &url_id in url_ids {
            let result = if let Some(uid) = user_id {
                sqlx::query("UPDATE urls SET status = $1 WHERE id = $2 AND user_id = $3")
                    .bind(status.to_string())
                    .bind(url_id)
                    .bind(uid)
                    .execute(&self.pool)
                    .await
            } else {
                sqlx::query("UPDATE urls SET status = $1 WHERE id = $2")
                    .bind(status.to_string())
                    .bind(url_id)
                    .execute(&self.pool)
                    .await
            };

            match result {
                Ok(res) if res.rows_affected() > 0 => results.push(BatchItemResult {
                    url_id,
                    success: true,
                    error: None,
                }),
                Ok(_) => results.push(BatchItemResult {
                    url_id,
                    success: false,
                    error: Some("URL not found or unauthorized".to_string()),
                }),
                Err(e) => results.push(BatchItemResult {
                    url_id,
                    success: false,
                    error: Some(e.to_string()),
                }),
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
        use crate::domain::repositories::url_repository::{BatchItemResult, BatchOperationResult};

        let mut results = Vec::new();
        for &url_id in url_ids {
            let result = if let Some(uid) = user_id {
                sqlx::query("UPDATE urls SET expiration_date = $1 WHERE id = $2 AND user_id = $3")
                    .bind(expiration_date)
                    .bind(url_id)
                    .bind(uid)
                    .execute(&self.pool)
                    .await
            } else {
                sqlx::query("UPDATE urls SET expiration_date = $1 WHERE id = $2")
                    .bind(expiration_date)
                    .bind(url_id)
                    .execute(&self.pool)
                    .await
            };

            match result {
                Ok(res) if res.rows_affected() > 0 => results.push(BatchItemResult {
                    url_id,
                    success: true,
                    error: None,
                }),
                Ok(_) => results.push(BatchItemResult {
                    url_id,
                    success: false,
                    error: Some("URL not found or unauthorized".to_string()),
                }),
                Err(e) => results.push(BatchItemResult {
                    url_id,
                    success: false,
                    error: Some(e.to_string()),
                }),
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
