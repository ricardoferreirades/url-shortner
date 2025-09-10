use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRecord {
    pub id: i32,
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Database { pool })
    }

    pub async fn create_url(
        &self,
        short_code: &str,
        original_url: &str,
    ) -> Result<UrlRecord, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO urls (short_code, original_url) VALUES ($1, $2) RETURNING id, short_code, original_url, created_at"
        )
        .bind(short_code)
        .bind(original_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(UrlRecord {
            id: row.get("id"),
            short_code: row.get("short_code"),
            original_url: row.get("original_url"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn get_url_by_short_code(
        &self,
        short_code: &str,
    ) -> Result<Option<UrlRecord>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, short_code, original_url, created_at FROM urls WHERE short_code = $1",
        )
        .bind(short_code)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(UrlRecord {
                id: row.get("id"),
                short_code: row.get("short_code"),
                original_url: row.get("original_url"),
                created_at: row.get("created_at"),
            })),
            None => Ok(None),
        }
    }
}
