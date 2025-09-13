use std::env;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost:5432/url_shortener".to_string()),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            acquire_timeout: env::var("DATABASE_ACQUIRE_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            idle_timeout: env::var("DATABASE_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .unwrap_or(600),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(DatabaseConfig {
            url: env::var("DATABASE_URL")?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
            acquire_timeout: env::var("DATABASE_ACQUIRE_TIMEOUT")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
            idle_timeout: env::var("DATABASE_IDLE_TIMEOUT")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
        })
    }
}
