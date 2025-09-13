use crate::domain::entities::{ShortCode, Url};
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
}

#[async_trait]
impl UrlRepository for PostgresUrlRepository {
    async fn create_url(
        &self,
        short_code: &ShortCode,
        original_url: &str,
        user_id: Option<i32>,
    ) -> Result<Url, RepositoryError> {
        let row = sqlx::query(
            "INSERT INTO urls (short_code, original_url, user_id) VALUES ($1, $2, $3) RETURNING id, short_code, original_url, created_at, user_id"
        )
        .bind(short_code.value())
        .bind(original_url)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Url {
            id: row.get("id"),
            short_code: row.get("short_code"),
            original_url: row.get("original_url"),
            created_at: row.get("created_at"),
            user_id: row.get("user_id"),
        })
    }

    async fn find_by_short_code(&self, short_code: &ShortCode) -> Result<Option<Url>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, short_code, original_url, created_at, user_id FROM urls WHERE short_code = $1"
        )
        .bind(short_code.value())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Url {
                id: row.get("id"),
                short_code: row.get("short_code"),
                original_url: row.get("original_url"),
                created_at: row.get("created_at"),
                user_id: row.get("user_id"),
            })),
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Url>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT id, short_code, original_url, created_at, user_id FROM urls WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let urls = rows
            .into_iter()
            .map(|row| Url {
                id: row.get("id"),
                short_code: row.get("short_code"),
                original_url: row.get("original_url"),
                created_at: row.get("created_at"),
                user_id: row.get("user_id"),
            })
            .collect();

        Ok(urls)
    }

    async fn exists_by_short_code(&self, short_code: &ShortCode) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM urls WHERE short_code = $1"
        )
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
            sqlx::query("DELETE FROM urls WHERE id = $1 AND user_id IS NULL")
                .bind(id)
        };

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn update_url(&self, url: &Url) -> Result<Url, RepositoryError> {
        let row = sqlx::query(
            "UPDATE urls SET short_code = $1, original_url = $2 WHERE id = $3 RETURNING id, short_code, original_url, created_at, user_id"
        )
        .bind(&url.short_code)
        .bind(&url.original_url)
        .bind(url.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Url {
            id: row.get("id"),
            short_code: row.get("short_code"),
            original_url: row.get("original_url"),
            created_at: row.get("created_at"),
            user_id: row.get("user_id"),
        })
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
}
