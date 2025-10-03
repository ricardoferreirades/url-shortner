use crate::domain::entities::{User, ProfilePrivacy};
use crate::domain::repositories::user_repository::{RepositoryError, UserRepository};
use async_trait::async_trait;
use sqlx::{PgPool, Row};

/// PostgreSQL implementation of the UserRepository trait
#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert a database row to a User entity
    fn row_to_user(&self, row: &sqlx::postgres::PgRow) -> User {
        let privacy_str: String = row.get("privacy");
        let privacy = match privacy_str.as_str() {
            "public" => ProfilePrivacy::Public,
            "private" => ProfilePrivacy::Private,
            "friends_only" => ProfilePrivacy::FriendsOnly,
            _ => ProfilePrivacy::Public, // Default fallback
        };

        User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            bio: row.get("bio"),
            avatar_url: row.get("avatar_url"),
            website: row.get("website"),
            location: row.get("location"),
            privacy,
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, RepositoryError> {
        let row = sqlx::query(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) 
             RETURNING id, username, email, password_hash, created_at, first_name, last_name, 
             bio, avatar_url, website, location, privacy, updated_at"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.row_to_user(&row))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, first_name, last_name, 
             bio, avatar_url, website, location, privacy, updated_at FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_user(&row))),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, first_name, last_name, 
             bio, avatar_url, website, location, privacy, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_user(&row))),
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, first_name, last_name, 
             bio, avatar_url, website, location, privacy, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_user(&row))),
            None => Ok(None),
        }
    }

    async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    async fn update_profile(
        &self,
        user_id: i32,
        first_name: Option<&str>,
        last_name: Option<&str>,
        bio: Option<&str>,
        avatar_url: Option<&str>,
        website: Option<&str>,
        location: Option<&str>,
        privacy: Option<ProfilePrivacy>,
    ) -> Result<User, RepositoryError> {
        // Build dynamic update query
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if first_name.is_some() {
            query_parts.push(format!("first_name = ${}", param_count));
            param_count += 1;
        }
        if last_name.is_some() {
            query_parts.push(format!("last_name = ${}", param_count));
            param_count += 1;
        }
        if bio.is_some() {
            query_parts.push(format!("bio = ${}", param_count));
            param_count += 1;
        }
        if avatar_url.is_some() {
            query_parts.push(format!("avatar_url = ${}", param_count));
            param_count += 1;
        }
        if website.is_some() {
            query_parts.push(format!("website = ${}", param_count));
            param_count += 1;
        }
        if location.is_some() {
            query_parts.push(format!("location = ${}", param_count));
            param_count += 1;
        }
        if privacy.is_some() {
            query_parts.push(format!("privacy = ${}", param_count));
            param_count += 1;
        }

        if query_parts.is_empty() {
            return Err(RepositoryError::InvalidData("No fields to update".to_string()));
        }

        query_parts.push("updated_at = CURRENT_TIMESTAMP".to_string());

        let query = format!(
            "UPDATE users SET {} WHERE id = ${} 
             RETURNING id, username, email, password_hash, created_at, first_name, last_name, 
             bio, avatar_url, website, location, privacy, updated_at",
            query_parts.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query(&query);

        if let Some(name) = first_name {
            query_builder = query_builder.bind(name);
        }
        if let Some(name) = last_name {
            query_builder = query_builder.bind(name);
        }
        if let Some(bio) = bio {
            query_builder = query_builder.bind(bio);
        }
        if let Some(url) = avatar_url {
            query_builder = query_builder.bind(url);
        }
        if let Some(site) = website {
            query_builder = query_builder.bind(site);
        }
        if let Some(loc) = location {
            query_builder = query_builder.bind(loc);
        }
        if let Some(privacy) = privacy {
            let privacy_str = match privacy {
                ProfilePrivacy::Public => "public",
                ProfilePrivacy::Private => "private",
                ProfilePrivacy::FriendsOnly => "friends_only",
            };
            query_builder = query_builder.bind(privacy_str);
        }

        query_builder = query_builder.bind(user_id);

        let row = query_builder
            .fetch_one(&self.pool)
            .await?;

        Ok(self.row_to_user(&row))
    }

    async fn get_profile(&self, user_id: i32) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, first_name, last_name, 
             bio, avatar_url, website, location, privacy, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_user(&row))),
            None => Ok(None),
        }
    }
}
