use crate::domain::entities::User;
use crate::domain::repositories::user_repository::{RepositoryError, UserRepository};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // user id
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

/// Authentication service
#[derive(Clone)]
pub struct AuthService<R>
where
    R: UserRepository + Clone,
{
    user_repository: R,
    jwt_secret: String,
}

impl<R> AuthService<R>
where
    R: UserRepository + Clone,
{
    pub fn new(user_repository: R, jwt_secret: String) -> Self {
        Self {
            user_repository,
            jwt_secret,
        }
    }

    /// Register a new user
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, ServiceError> {
        // Validate input
        self.validate_registration_input(username, email, password)?;

        // Check if username already exists
        if self.user_repository.exists_by_username(username).await? {
            return Err(ServiceError::UsernameAlreadyExists);
        }

        // Check if email already exists
        if self.user_repository.exists_by_email(email).await? {
            return Err(ServiceError::EmailAlreadyExists);
        }

        // Hash password
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| ServiceError::PasswordHashing(e.to_string()))?;

        // Create user
        let user = self
            .user_repository
            .create_user(username, email, &password_hash)
            .await
            .map_err(ServiceError::Repository)?;

        Ok(user)
    }

    /// Login a user
    pub async fn login(&self, username: &str, password: &str) -> Result<String, ServiceError> {
        // Find user by username
        let user = self
            .user_repository
            .find_by_username(username)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or(ServiceError::InvalidCredentials)?;

        // Verify password
        let is_valid = verify(password, &user.password_hash)
            .map_err(|e| ServiceError::PasswordVerification(e.to_string()))?;

        if !is_valid {
            return Err(ServiceError::InvalidCredentials);
        }

        // Generate JWT token
        let token = self.generate_jwt_token(&user)?;

        Ok(token)
    }

    /// Verify JWT token and return user
    pub async fn verify_token(&self, token: &str) -> Result<User, ServiceError> {
        let claims = self.decode_jwt_token(token)?;

        let user = self
            .user_repository
            .find_by_id(claims.sub)
            .await
            .map_err(ServiceError::Repository)?
            .ok_or(ServiceError::UserNotFound)?;

        Ok(user)
    }

    /// Generate JWT token for user
    fn generate_jwt_token(&self, user: &User) -> Result<String, ServiceError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: user.id,
            username: user.username.clone(),
            exp: now + (24 * 60 * 60), // 24 hours
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| ServiceError::TokenGeneration(e.to_string()))?;

        Ok(token)
    }

    /// Decode JWT token
    fn decode_jwt_token(&self, token: &str) -> Result<Claims, ServiceError> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )
        .map_err(|e| ServiceError::TokenValidation(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Validate registration input
    fn validate_registration_input(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(), ServiceError> {
        if username.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "Username cannot be empty".to_string(),
            ));
        }

        if username.len() < 3 || username.len() > 50 {
            return Err(ServiceError::InvalidInput(
                "Username must be between 3 and 50 characters".to_string(),
            ));
        }

        if email.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "Email cannot be empty".to_string(),
            ));
        }

        if !email.contains('@') {
            return Err(ServiceError::InvalidInput(
                "Invalid email format".to_string(),
            ));
        }

        if password.len() < 6 {
            return Err(ServiceError::InvalidInput(
                "Password must be at least 6 characters".to_string(),
            ));
        }

        Ok(())
    }
}

/// Service errors
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Username already exists")]
    UsernameAlreadyExists,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Password hashing error: {0}")]
    PasswordHashing(String),

    #[error("Password verification error: {0}")]
    PasswordVerification(String),

    #[error("Token generation error: {0}")]
    TokenGeneration(String),

    #[error("Token validation error: {0}")]
    TokenValidation(String),
}
