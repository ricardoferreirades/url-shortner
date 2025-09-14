use crate::domain::entities::User;
use async_trait::async_trait;
use thiserror::Error;

/// Repository trait for User operations
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Create a new user
    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, RepositoryError>;

    /// Find a user by username
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;

    /// Find a user by email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;

    /// Find a user by ID
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, RepositoryError>;

    /// Check if username exists
    async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError>;

    /// Check if email exists
    async fn exists_by_email(&self, email: &str) -> Result<bool, RepositoryError>;
}

/// Repository errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("User not found")]
    NotFound,

    #[error("Duplicate username")]
    DuplicateUsername,

    #[error("Duplicate email")]
    DuplicateEmail,

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
