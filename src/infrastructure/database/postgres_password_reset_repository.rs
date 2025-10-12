use crate::domain::entities::PasswordResetToken;
use crate::domain::repositories::password_reset_repository::PasswordResetRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

/// PostgreSQL implementation of the PasswordResetRepository trait
#[derive(Clone)]
pub struct PostgresPasswordResetRepository {
    pool: PgPool,
}

impl PostgresPasswordResetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert a database row to a PasswordResetToken entity
    fn row_to_token(&self, row: &sqlx::postgres::PgRow) -> PasswordResetToken {
        PasswordResetToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token: row.get("token"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            used_at: row.get("used_at"),
            is_used: row.get("is_used"),
        }
    }
}

#[async_trait]
impl PasswordResetRepository for PostgresPasswordResetRepository {
    async fn create_token(
        &self,
        token: PasswordResetToken,
    ) -> Result<PasswordResetToken, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "INSERT INTO password_reset_tokens (user_id, token, created_at, expires_at, is_used) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id, user_id, token, created_at, expires_at, used_at, is_used",
        )
        .bind(token.user_id)
        .bind(&token.token)
        .bind(token.created_at)
        .bind(token.expires_at)
        .bind(token.is_used)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.row_to_token(&row))
    }

    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<PasswordResetToken>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT id, user_id, token, created_at, expires_at, used_at, is_used 
             FROM password_reset_tokens 
             WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_token(&row))),
            None => Ok(None),
        }
    }

    async fn find_active_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<PasswordResetToken>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT id, user_id, token, created_at, expires_at, used_at, is_used 
             FROM password_reset_tokens 
             WHERE user_id = $1 AND is_used = false AND expires_at > NOW() 
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let tokens = rows.iter().map(|row| self.row_to_token(row)).collect();

        Ok(tokens)
    }

    async fn count_active_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM password_reset_tokens 
             WHERE user_id = $1 AND is_used = false AND expires_at > NOW()",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count as usize)
    }

    async fn update_token(
        &self,
        token: PasswordResetToken,
    ) -> Result<PasswordResetToken, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "UPDATE password_reset_tokens 
             SET is_used = $1, used_at = $2 
             WHERE id = $3 
             RETURNING id, user_id, token, created_at, expires_at, used_at, is_used",
        )
        .bind(token.is_used)
        .bind(token.used_at)
        .bind(token.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.row_to_token(&row))
    }

    async fn delete_expired_tokens(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            "DELETE FROM password_reset_tokens 
             WHERE expires_at < NOW()",
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    async fn revoke_all_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            "UPDATE password_reset_tokens 
             SET is_used = true, used_at = NOW() 
             WHERE user_id = $1 AND is_used = false",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }
}
