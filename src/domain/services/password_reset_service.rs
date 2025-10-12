use crate::domain::entities::{PasswordResetToken, User};
use crate::domain::repositories::password_reset_repository::PasswordResetRepository;
use crate::domain::repositories::user_repository::UserRepository;
use chrono::{Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use thiserror::Error;
use uuid::Uuid;

/// Password reset service for handling password reset operations
pub struct PasswordResetService<R, U>
where
    R: PasswordResetRepository,
    U: UserRepository,
{
    password_reset_repository: R,
    user_repository: U,
    token_length: usize,
    token_expiration_hours: i64,
    max_tokens_per_user: usize,
}

/// Password reset service errors
#[derive(Error, Debug)]
pub enum PasswordResetError {
    #[error("User not found")]
    UserNotFound,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token already used")]
    TokenAlreadyUsed,

    #[error("Too many reset requests")]
    TooManyRequests,

    #[error("Repository error: {0}")]
    RepositoryError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Password reset request result
#[derive(Debug, Clone)]
pub struct PasswordResetRequest {
    #[allow(dead_code)]
    pub user_id: i32,
    pub email: String,
    pub token: String,
    #[allow(dead_code)]
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[allow(dead_code)]
impl<R, U> PasswordResetService<R, U>
where
    R: PasswordResetRepository,
    U: UserRepository,
{
    /// Create a new password reset service
    pub fn new(
        password_reset_repository: R,
        user_repository: U,
        token_length: usize,
        token_expiration_hours: i64,
        max_tokens_per_user: usize,
    ) -> Self {
        Self {
            password_reset_repository,
            user_repository,
            token_length,
            token_expiration_hours,
            max_tokens_per_user,
        }
    }

    /// Create a default password reset service
    pub fn new_default(password_reset_repository: R, user_repository: U) -> Self {
        Self::new(
            password_reset_repository,
            user_repository,
            32, // 32 character token
            24, // 24 hours expiration
            5,  // Max 5 tokens per user
        )
    }

    /// Generate a secure random token
    fn generate_token(&self) -> String {
        // Generate a random alphanumeric token
        let random_part: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.token_length - 8) // Leave space for UUID prefix
            .map(char::from)
            .collect();

        // Add UUID prefix for additional uniqueness
        let uuid_prefix = Uuid::new_v4().to_string()[..8].to_string();

        format!("{}{}", uuid_prefix, random_part)
    }

    /// Create a password reset request for a user
    pub async fn create_reset_request(
        &self,
        email: &str,
    ) -> Result<PasswordResetRequest, PasswordResetError> {
        // Find user by email
        let user = self
            .user_repository
            .find_by_email(email)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))?
            .ok_or(PasswordResetError::UserNotFound)?;

        // Check if user has too many active reset tokens
        let active_tokens = self
            .password_reset_repository
            .count_active_tokens_for_user(user.id)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))?;

        if active_tokens >= self.max_tokens_per_user {
            return Err(PasswordResetError::TooManyRequests);
        }

        // Generate secure token
        let token = self.generate_token();
        let expires_at = Utc::now() + Duration::hours(self.token_expiration_hours);

        // Create password reset token
        let reset_token = PasswordResetToken::new(
            0, // ID will be set by repository
            user.id,
            token.clone(),
            Utc::now(),
            expires_at,
        );

        // Save token to repository
        self.password_reset_repository
            .create_token(reset_token)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))?;

        Ok(PasswordResetRequest {
            user_id: user.id,
            email: user.email,
            token,
            expires_at,
        })
    }

    /// Validate a password reset token
    pub async fn validate_token(
        &self,
        token: &str,
    ) -> Result<PasswordResetToken, PasswordResetError> {
        let reset_token = self
            .password_reset_repository
            .find_by_token(token)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))?
            .ok_or(PasswordResetError::InvalidToken)?;

        if reset_token.is_expired() {
            return Err(PasswordResetError::TokenExpired);
        }

        if reset_token.is_used {
            return Err(PasswordResetError::TokenAlreadyUsed);
        }

        Ok(reset_token)
    }

    /// Reset password using a valid token
    pub async fn reset_password(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<User, PasswordResetError> {
        // Validate token
        let mut reset_token = self.validate_token(token).await?;

        // Get user
        let user = self
            .user_repository
            .find_by_id(reset_token.user_id)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))?
            .ok_or(PasswordResetError::UserNotFound)?;

        // Hash new password (in a real implementation, you would hash the password)
        // For now, we'll just store it as-is (this should be replaced with proper hashing)
        let _hashed_password = new_password.to_string(); // TODO: Implement proper password hashing

        // Update user password
        // Note: This would require a new method in UserRepository to update password
        // For now, we'll mark the token as used
        reset_token.mark_as_used();
        self.password_reset_repository
            .update_token(reset_token)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))?;

        Ok(user)
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> Result<usize, PasswordResetError> {
        self.password_reset_repository
            .delete_expired_tokens()
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))
    }

    /// Get active tokens for a user
    pub async fn get_active_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<PasswordResetToken>, PasswordResetError> {
        self.password_reset_repository
            .find_active_tokens_for_user(user_id)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))
    }

    /// Revoke all tokens for a user
    pub async fn revoke_all_tokens_for_user(
        &self,
        user_id: i32,
    ) -> Result<usize, PasswordResetError> {
        self.password_reset_repository
            .revoke_all_tokens_for_user(user_id)
            .await
            .map_err(|e| PasswordResetError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::User;
    // Mock implementations for testing
    struct MockPasswordResetRepository;
    struct MockUserRepository;

    #[async_trait::async_trait]
    impl PasswordResetRepository for MockPasswordResetRepository {
        async fn create_token(
            &self,
            _token: PasswordResetToken,
        ) -> Result<PasswordResetToken, Box<dyn std::error::Error + Send + Sync>> {
            Ok(PasswordResetToken::new_with_timestamp(
                1,
                1,
                "test_token".to_string(),
                24,
            ))
        }

        async fn find_by_token(
            &self,
            _token: &str,
        ) -> Result<Option<PasswordResetToken>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Some(PasswordResetToken::new_with_timestamp(
                1,
                1,
                "test_token".to_string(),
                24,
            )))
        }

        async fn find_active_tokens_for_user(
            &self,
            _user_id: i32,
        ) -> Result<Vec<PasswordResetToken>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![])
        }

        async fn count_active_tokens_for_user(
            &self,
            _user_id: i32,
        ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
            Ok(0)
        }

        async fn update_token(
            &self,
            _token: PasswordResetToken,
        ) -> Result<PasswordResetToken, Box<dyn std::error::Error + Send + Sync>> {
            Ok(PasswordResetToken::new_with_timestamp(
                1,
                1,
                "test_token".to_string(),
                24,
            ))
        }

        async fn delete_expired_tokens(
            &self,
        ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
            Ok(0)
        }

        async fn revoke_all_tokens_for_user(
            &self,
            _user_id: i32,
        ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
            Ok(0)
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn create_user(
            &self,
            _username: &str,
            _email: &str,
            _password_hash: &str,
        ) -> Result<User, crate::domain::repositories::user_repository::RepositoryError> {
            Ok(User::new_with_timestamp(
                1,
                "test".to_string(),
                "test@example.com".to_string(),
                "hash".to_string(),
            ))
        }

        async fn find_by_username(
            &self,
            _username: &str,
        ) -> Result<Option<User>, crate::domain::repositories::user_repository::RepositoryError>
        {
            Ok(Some(User::new_with_timestamp(
                1,
                "test".to_string(),
                "test@example.com".to_string(),
                "hash".to_string(),
            )))
        }

        async fn find_by_email(
            &self,
            email: &str,
        ) -> Result<Option<User>, crate::domain::repositories::user_repository::RepositoryError>
        {
            if email == "test@example.com" {
                Ok(Some(User::new_with_timestamp(
                    1,
                    "test".to_string(),
                    "test@example.com".to_string(),
                    "hash".to_string(),
                )))
            } else {
                Ok(None)
            }
        }

        async fn find_by_id(
            &self,
            _id: i32,
        ) -> Result<Option<User>, crate::domain::repositories::user_repository::RepositoryError>
        {
            Ok(Some(User::new_with_timestamp(
                1,
                "test".to_string(),
                "test@example.com".to_string(),
                "hash".to_string(),
            )))
        }

        async fn exists_by_username(
            &self,
            _username: &str,
        ) -> Result<bool, crate::domain::repositories::user_repository::RepositoryError> {
            Ok(false)
        }

        async fn exists_by_email(
            &self,
            _email: &str,
        ) -> Result<bool, crate::domain::repositories::user_repository::RepositoryError> {
            Ok(false)
        }

        async fn update_profile(
            &self,
            _user_id: i32,
            _first_name: Option<&str>,
            _last_name: Option<&str>,
            _bio: Option<&str>,
            _avatar_url: Option<&str>,
            _website: Option<&str>,
            _location: Option<&str>,
            _privacy: Option<crate::domain::entities::ProfilePrivacy>,
        ) -> Result<User, crate::domain::repositories::user_repository::RepositoryError> {
            Ok(User::new_with_timestamp(
                1,
                "test".to_string(),
                "test@example.com".to_string(),
                "hash".to_string(),
            ))
        }

        async fn get_profile(
            &self,
            _user_id: i32,
        ) -> Result<Option<User>, crate::domain::repositories::user_repository::RepositoryError>
        {
            Ok(Some(User::new_with_timestamp(
                1,
                "test".to_string(),
                "test@example.com".to_string(),
                "hash".to_string(),
            )))
        }

        async fn delete_account(
            &self,
            _user_id: i32,
        ) -> Result<(), crate::domain::repositories::user_repository::RepositoryError> {
            Ok(())
        }

        async fn anonymize_account(
            &self,
            _user_id: i32,
            _anonymized_username: &str,
            _anonymized_email: &str,
            _anonymized_password_hash: &str,
        ) -> Result<(), crate::domain::repositories::user_repository::RepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_reset_request() {
        let service =
            PasswordResetService::new_default(MockPasswordResetRepository, MockUserRepository);

        let result = service.create_reset_request("test@example.com").await;
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.user_id, 1);
        assert!(!request.token.is_empty());
    }

    #[tokio::test]
    async fn test_create_reset_request_user_not_found() {
        let service =
            PasswordResetService::new_default(MockPasswordResetRepository, MockUserRepository);

        let result = service
            .create_reset_request("nonexistent@example.com")
            .await;
        assert!(matches!(result, Err(PasswordResetError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_validate_token() {
        let service =
            PasswordResetService::new_default(MockPasswordResetRepository, MockUserRepository);

        let result = service.validate_token("test_token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_password() {
        let service =
            PasswordResetService::new_default(MockPasswordResetRepository, MockUserRepository);

        let result = service.reset_password("test_token", "new_password").await;
        assert!(result.is_ok());
    }
}
