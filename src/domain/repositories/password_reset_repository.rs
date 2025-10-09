use crate::domain::entities::PasswordResetToken;
use async_trait::async_trait;
use thiserror::Error;

/// Repository trait for Password Reset Token operations
#[async_trait]
#[allow(dead_code)]
pub trait PasswordResetRepository: Send + Sync {
    /// Create a new password reset token
    async fn create_token(
        &self,
        token: PasswordResetToken,
    ) -> Result<PasswordResetToken, Box<dyn std::error::Error + Send + Sync>>;

    /// Find a password reset token by token string
    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<PasswordResetToken>, Box<dyn std::error::Error + Send + Sync>>;

    /// Find all active tokens for a user
    async fn find_active_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<PasswordResetToken>, Box<dyn std::error::Error + Send + Sync>>;

    /// Count active tokens for a user
    async fn count_active_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// Update a password reset token
    async fn update_token(
        &self,
        token: PasswordResetToken,
    ) -> Result<PasswordResetToken, Box<dyn std::error::Error + Send + Sync>>;

    /// Delete expired tokens
    async fn delete_expired_tokens(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// Revoke all tokens for a user
    async fn revoke_all_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

/// Repository errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum PasswordResetRepositoryError {
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
