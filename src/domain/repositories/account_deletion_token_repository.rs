use crate::domain::entities::AccountDeletionToken;
use async_trait::async_trait;
use thiserror::Error;

/// Repository trait for Account Deletion Token operations
#[async_trait]
#[allow(dead_code)]
pub trait AccountDeletionTokenRepository: Send + Sync {
    /// Create a new account deletion token
    async fn create_token(
        &self,
        token: AccountDeletionToken,
    ) -> Result<AccountDeletionToken, Box<dyn std::error::Error + Send + Sync>>;

    /// Find an account deletion token by token string
    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<AccountDeletionToken>, Box<dyn std::error::Error + Send + Sync>>;

    /// Find active token for a user (should only be one at a time)
    async fn find_active_token_for_user(
        &self,
        user_id: i32,
    ) -> Result<Option<AccountDeletionToken>, Box<dyn std::error::Error + Send + Sync>>;

    /// Update an account deletion token
    async fn update_token(
        &self,
        token: AccountDeletionToken,
    ) -> Result<AccountDeletionToken, Box<dyn std::error::Error + Send + Sync>>;

    /// Delete expired tokens
    async fn delete_expired_tokens(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// Cancel all active tokens for a user
    async fn cancel_all_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

/// Repository errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AccountDeletionTokenError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Token not found")]
    TokenNotFound,

    #[error("Duplicate token")]
    DuplicateToken,

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

