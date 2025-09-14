use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Domain entity representing a User
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Create a new User entity
    pub fn new(
        id: i32,
        username: String,
        email: String,
        password_hash: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            created_at,
        }
    }

    /// Create a new User with current timestamp
    pub fn new_with_timestamp(
        id: i32,
        username: String,
        email: String,
        password_hash: String,
    ) -> Self {
        Self::new(id, username, email, password_hash, Utc::now())
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User(id={}, username={}, email={}, created_at={})",
            self.id, self.username, self.email, self.created_at
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new_with_timestamp(
            1,
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );
        
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
    }
}
