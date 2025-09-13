use std::env;

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            window_size: env::var("RATE_LIMIT_WINDOW_SIZE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        }
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(RateLimitConfig {
            requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
            burst_size: env::var("RATE_LIMIT_BURST_SIZE")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
            window_size: env::var("RATE_LIMIT_WINDOW_SIZE")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
        })
    }
}
