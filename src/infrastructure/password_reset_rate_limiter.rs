use chrono::{DateTime, Duration, Utc};
use governor::{
    clock::{Clock, DefaultClock},
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Password reset rate limiting configuration
#[derive(Debug, Clone)]
pub struct PasswordResetRateLimitConfig {
    /// Maximum requests per IP per hour
    pub requests_per_hour_per_ip: u32,
    /// Maximum requests per email per hour
    pub requests_per_hour_per_email: u32,
    /// Cooldown period between requests (in minutes)
    pub cooldown_minutes: i64,
    /// Maximum active tokens per user
    #[allow(dead_code)]
    pub max_active_tokens_per_user: usize,
}

impl Default for PasswordResetRateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_hour_per_ip: 5,    // 5 requests per hour per IP
            requests_per_hour_per_email: 3, // 3 requests per hour per email
            cooldown_minutes: 5,            // 5 minutes between requests
            max_active_tokens_per_user: 5,  // Max 5 active tokens per user
        }
    }
}

/// Password reset rate limiter
pub struct PasswordResetRateLimiter {
    config: PasswordResetRateLimitConfig,
    ip_limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
    email_limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
    last_request_times: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
}

/// Rate limiting errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum PasswordResetRateLimitError {
    #[error("Too many requests from this IP address. Please try again in {0} seconds")]
    IpRateLimitExceeded(u64),

    #[error("Too many requests for this email address. Please try again in {0} seconds")]
    EmailRateLimitExceeded(u64),

    #[error("Please wait {0} minutes before requesting another password reset")]
    CooldownPeriodActive(i64),

    #[error(
        "Too many active reset tokens. Please use an existing token or wait for them to expire"
    )]
    TooManyActiveTokens,

    #[error("Internal rate limiting error: {0}")]
    Internal(String),
}

#[allow(dead_code)]
impl PasswordResetRateLimiter {
    /// Create a new password reset rate limiter
    pub fn new(config: PasswordResetRateLimitConfig) -> Self {
        let ip_quota = Quota::per_hour(NonZeroU32::new(config.requests_per_hour_per_ip).unwrap());
        let email_quota =
            Quota::per_hour(NonZeroU32::new(config.requests_per_hour_per_email).unwrap());

        let ip_limiter = Arc::new(RateLimiter::new(
            ip_quota,
            DefaultKeyedStateStore::new(),
            &DefaultClock::default(),
        ));

        let email_limiter = Arc::new(RateLimiter::new(
            email_quota,
            DefaultKeyedStateStore::new(),
            &DefaultClock::default(),
        ));

        Self {
            config,
            ip_limiter,
            email_limiter,
            last_request_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a default password reset rate limiter
    pub fn new_default() -> Self {
        Self::new(PasswordResetRateLimitConfig::default())
    }

    /// Check IP rate limit
    pub fn check_ip_limit(&self, ip: &str) -> Result<(), PasswordResetRateLimitError> {
        match self.ip_limiter.check_key(&ip.to_string()) {
            Ok(_) => Ok(()),
            Err(negative) => {
                let retry_after = negative
                    .wait_time_from(DefaultClock::default().now())
                    .as_secs();
                Err(PasswordResetRateLimitError::IpRateLimitExceeded(
                    retry_after,
                ))
            }
        }
    }

    /// Check email rate limit
    pub fn check_email_limit(&self, email: &str) -> Result<(), PasswordResetRateLimitError> {
        match self.email_limiter.check_key(&email.to_string()) {
            Ok(_) => Ok(()),
            Err(negative) => {
                let retry_after = negative
                    .wait_time_from(DefaultClock::default().now())
                    .as_secs();
                Err(PasswordResetRateLimitError::EmailRateLimitExceeded(
                    retry_after,
                ))
            }
        }
    }

    /// Check cooldown period
    pub async fn check_cooldown(&self, email: &str) -> Result<(), PasswordResetRateLimitError> {
        let mut last_times = self.last_request_times.lock().await;

        if let Some(last_time) = last_times.get(email) {
            let now = Utc::now();
            let elapsed = now - *last_time;
            let cooldown = Duration::minutes(self.config.cooldown_minutes);

            if elapsed < cooldown {
                let remaining_minutes = (cooldown - elapsed).num_minutes();
                return Err(PasswordResetRateLimitError::CooldownPeriodActive(
                    remaining_minutes,
                ));
            }
        }

        // Update last request time
        last_times.insert(email.to_string(), Utc::now());

        Ok(())
    }

    /// Check all rate limits
    pub async fn check_all_limits(
        &self,
        ip: &str,
        email: &str,
    ) -> Result<(), PasswordResetRateLimitError> {
        // Check IP rate limit
        self.check_ip_limit(ip)?;

        // Check email rate limit
        self.check_email_limit(email)?;

        // Check cooldown period
        self.check_cooldown(email).await?;

        Ok(())
    }

    /// Clean up old entries from last request times
    pub async fn cleanup_old_entries(&self) {
        let mut last_times = self.last_request_times.lock().await;
        let cutoff_time = Utc::now() - Duration::hours(24);

        last_times.retain(|_, time| *time > cutoff_time);
    }

    /// Get rate limit info for debugging
    pub async fn get_rate_limit_info(&self, email: &str) -> RateLimitInfo {
        let last_times = self.last_request_times.lock().await;

        let last_request = last_times.get(email).cloned();
        let cooldown_remaining = if let Some(last_time) = last_request {
            let now = Utc::now();
            let elapsed = now - last_time;
            let cooldown = Duration::minutes(self.config.cooldown_minutes);

            if elapsed < cooldown {
                Some((cooldown - elapsed).num_minutes())
            } else {
                None
            }
        } else {
            None
        };

        RateLimitInfo {
            requests_per_hour_per_ip: self.config.requests_per_hour_per_ip,
            requests_per_hour_per_email: self.config.requests_per_hour_per_email,
            cooldown_minutes: self.config.cooldown_minutes,
            cooldown_remaining_minutes: cooldown_remaining,
            last_request,
        }
    }
}

/// Rate limit information for debugging
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub requests_per_hour_per_ip: u32,
    pub requests_per_hour_per_email: u32,
    pub cooldown_minutes: i64,
    pub cooldown_remaining_minutes: Option<i64>,
    pub last_request: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let config = PasswordResetRateLimitConfig::default();
        assert_eq!(config.requests_per_hour_per_ip, 5);
        assert_eq!(config.requests_per_hour_per_email, 3);
        assert_eq!(config.cooldown_minutes, 5);
        assert_eq!(config.max_active_tokens_per_user, 5);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = PasswordResetRateLimiter::new_default();
        assert!(limiter.check_ip_limit("192.168.1.1").is_ok());
        assert!(limiter.check_email_limit("test@example.com").is_ok());
    }

    #[tokio::test]
    async fn test_cooldown_check() {
        let limiter = PasswordResetRateLimiter::new_default();

        // First request should succeed
        assert!(limiter.check_cooldown("test@example.com").await.is_ok());

        // Second immediate request should fail
        let result = limiter.check_cooldown("test@example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_old_entries() {
        let limiter = PasswordResetRateLimiter::new_default();

        // Add an entry
        let _ = limiter.check_cooldown("test@example.com").await;

        // Cleanup should work
        limiter.cleanup_old_entries().await;

        // Entry should still exist (not old enough)
        let last_times = limiter.last_request_times.lock().await;
        assert!(last_times.contains_key("test@example.com"));
    }

    #[tokio::test]
    async fn test_get_rate_limit_info() {
        let limiter = PasswordResetRateLimiter::new_default();
        let _ = limiter.check_cooldown("test@example.com").await;

        let info = limiter.get_rate_limit_info("test@example.com").await;
        assert_eq!(info.requests_per_hour_per_ip, 5);
        assert_eq!(info.requests_per_hour_per_email, 3);
        assert!(info.last_request.is_some());
    }
}
