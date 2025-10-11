use crate::domain::entities::AccountDeletionToken;
use crate::domain::repositories::account_deletion_token_repository::AccountDeletionTokenRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

/// PostgreSQL implementation of the AccountDeletionTokenRepository trait
#[allow(dead_code)]
#[derive(Clone)]
pub struct PostgresAccountDeletionTokenRepository {
    pool: PgPool,
}

#[allow(dead_code)]
impl PostgresAccountDeletionTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert a database row to an AccountDeletionToken entity
    fn row_to_token(&self, row: &sqlx::postgres::PgRow) -> AccountDeletionToken {
        AccountDeletionToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token: row.get("token"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            confirmed_at: row.get("confirmed_at"),
            is_confirmed: row.get("is_confirmed"),
            is_cancelled: row.get("is_cancelled"),
        }
    }
}

#[async_trait]
impl AccountDeletionTokenRepository for PostgresAccountDeletionTokenRepository {
    async fn create_token(
        &self,
        token: AccountDeletionToken,
    ) -> Result<AccountDeletionToken, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "INSERT INTO account_deletion_tokens (user_id, token, created_at, expires_at, is_confirmed, is_cancelled) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id, user_id, token, created_at, expires_at, confirmed_at, is_confirmed, is_cancelled"
        )
        .bind(token.user_id)
        .bind(&token.token)
        .bind(token.created_at)
        .bind(token.expires_at)
        .bind(token.is_confirmed)
        .bind(token.is_cancelled)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.row_to_token(&row))
    }

    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<AccountDeletionToken>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT id, user_id, token, created_at, expires_at, confirmed_at, is_confirmed, is_cancelled 
             FROM account_deletion_tokens 
             WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_token(&row))),
            None => Ok(None),
        }
    }

    async fn find_active_token_for_user(
        &self,
        user_id: i32,
    ) -> Result<Option<AccountDeletionToken>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT id, user_id, token, created_at, expires_at, confirmed_at, is_confirmed, is_cancelled 
             FROM account_deletion_tokens 
             WHERE user_id = $1 AND is_confirmed = false AND is_cancelled = false AND expires_at > NOW() 
             ORDER BY created_at DESC 
             LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_token(&row))),
            None => Ok(None),
        }
    }

    async fn update_token(
        &self,
        token: AccountDeletionToken,
    ) -> Result<AccountDeletionToken, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "UPDATE account_deletion_tokens 
             SET is_confirmed = $1, is_cancelled = $2, confirmed_at = $3 
             WHERE id = $4 
             RETURNING id, user_id, token, created_at, expires_at, confirmed_at, is_confirmed, is_cancelled"
        )
        .bind(token.is_confirmed)
        .bind(token.is_cancelled)
        .bind(token.confirmed_at)
        .bind(token.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.row_to_token(&row))
    }

    async fn delete_expired_tokens(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            "DELETE FROM account_deletion_tokens WHERE expires_at < NOW()"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    async fn cancel_all_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            "UPDATE account_deletion_tokens 
             SET is_cancelled = true 
             WHERE user_id = $1 AND is_confirmed = false AND is_cancelled = false"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }
}

