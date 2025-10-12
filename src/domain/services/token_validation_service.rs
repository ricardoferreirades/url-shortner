use crate::domain::entities::PasswordResetToken;
use chrono::Duration;
use thiserror::Error;

/// Token validation service for comprehensive token validation
pub struct TokenValidationService {
    min_token_length: usize,
    max_token_age_hours: i64,
    #[allow(dead_code)]
    require_https: bool,
}

/// Token validation errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum TokenValidationError {
    #[error("Token is too short: {0} characters (minimum: {1})")]
    TokenTooShort(usize, usize),

    #[error("Token is expired")]
    TokenExpired,

    #[error("Token has already been used")]
    TokenAlreadyUsed,

    #[error("Token format is invalid")]
    InvalidFormat,

    #[error("Token is too old: created {0} hours ago (maximum: {1})")]
    TokenTooOld(i64, i64),

    #[error("Security validation failed: {0}")]
    SecurityValidationFailed(String),
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenValidationResult {
    pub is_valid: bool,
    #[allow(dead_code)]
    pub is_expired: bool,
    #[allow(dead_code)]
    pub is_used: bool,
    pub time_until_expiration: Option<Duration>,
    #[allow(dead_code)]
    pub time_since_creation: Duration,
    #[allow(dead_code)]
    pub validation_errors: Vec<String>,
}

#[allow(dead_code)]
impl TokenValidationService {
    /// Create a new token validation service
    pub fn new(min_token_length: usize, max_token_age_hours: i64, require_https: bool) -> Self {
        Self {
            min_token_length,
            max_token_age_hours,
            require_https,
        }
    }

    /// Create a default token validation service
    pub fn new_default() -> Self {
        Self::new(
            16,   // Minimum 16 characters
            48,   // Maximum 48 hours (2 days)
            true, // Require HTTPS for security
        )
    }

    /// Validate token string format
    pub fn validate_token_format(&self, token: &str) -> Result<(), TokenValidationError> {
        // Check token length
        if token.len() < self.min_token_length {
            return Err(TokenValidationError::TokenTooShort(
                token.len(),
                self.min_token_length,
            ));
        }

        // Check token contains only valid characters (alphanumeric and hyphens)
        if !token.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(TokenValidationError::InvalidFormat);
        }

        Ok(())
    }

    /// Validate token entity
    pub fn validate_token_entity(
        &self,
        token: &PasswordResetToken,
    ) -> Result<(), TokenValidationError> {
        // Check if token is expired
        if token.is_expired() {
            return Err(TokenValidationError::TokenExpired);
        }

        // Check if token has been used
        if token.is_used {
            return Err(TokenValidationError::TokenAlreadyUsed);
        }

        // Check token age
        let age_hours = token.time_since_creation().num_hours();
        if age_hours > self.max_token_age_hours {
            return Err(TokenValidationError::TokenTooOld(
                age_hours,
                self.max_token_age_hours,
            ));
        }

        Ok(())
    }

    /// Comprehensive token validation
    pub fn validate(
        &self,
        token: &str,
        token_entity: &PasswordResetToken,
    ) -> Result<TokenValidationResult, TokenValidationError> {
        let mut validation_errors = Vec::new();

        // Validate token format
        if let Err(e) = self.validate_token_format(token) {
            validation_errors.push(e.to_string());
        }

        // Validate token entity
        if let Err(e) = self.validate_token_entity(token_entity) {
            validation_errors.push(e.to_string());
        }

        let is_valid = validation_errors.is_empty();

        Ok(TokenValidationResult {
            is_valid,
            is_expired: token_entity.is_expired(),
            is_used: token_entity.is_used,
            time_until_expiration: token_entity.time_until_expiration(),
            time_since_creation: token_entity.time_since_creation(),
            validation_errors,
        })
    }

    /// Validate reset link URL (for security)
    pub fn validate_reset_link(&self, link: &str) -> Result<(), TokenValidationError> {
        if self.require_https && !link.starts_with("https://") {
            return Err(TokenValidationError::SecurityValidationFailed(
                "Reset link must use HTTPS".to_string(),
            ));
        }

        // Check for common XSS patterns
        if link.contains("<script") || link.contains("javascript:") {
            return Err(TokenValidationError::SecurityValidationFailed(
                "Reset link contains potentially dangerous content".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if token will expire soon (within specified hours)
    pub fn will_expire_soon(&self, token: &PasswordResetToken, hours: i64) -> bool {
        if let Some(time_until) = token.time_until_expiration() {
            time_until.num_hours() <= hours
        } else {
            true // Already expired
        }
    }

    /// Get token strength score (0-100)
    pub fn get_token_strength_score(&self, token: &str) -> u8 {
        let mut score = 0;

        // Length score (max 40 points)
        let length_score = std::cmp::min((token.len() * 40) / 64, 40);
        score += length_score;

        // Character variety score (max 30 points)
        let has_uppercase = token.chars().any(|c| c.is_uppercase());
        let has_lowercase = token.chars().any(|c| c.is_lowercase());
        let has_digit = token.chars().any(|c| c.is_numeric());
        let has_special = token.chars().any(|c| !c.is_alphanumeric());

        if has_uppercase {
            score += 7;
        }
        if has_lowercase {
            score += 7;
        }
        if has_digit {
            score += 8;
        }
        if has_special {
            score += 8;
        }

        // Randomness score (max 30 points)
        // Simple check: no repeated characters in sequence
        let has_no_repeats = !token
            .chars()
            .collect::<Vec<_>>()
            .windows(3)
            .any(|w| w[0] == w[1] && w[1] == w[2]);

        if has_no_repeats {
            score += 30;
        }

        std::cmp::min(score, 100) as u8
    }
}

impl Default for TokenValidationService {
    fn default() -> Self {
        Self::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::PasswordResetToken;

    #[test]
    fn test_validate_token_format() {
        let service = TokenValidationService::new_default();

        // Valid token
        assert!(service
            .validate_token_format("abc123-def456-ghi789-jkl012")
            .is_ok());

        // Too short
        assert!(service.validate_token_format("short").is_err());

        // Invalid characters
        assert!(service
            .validate_token_format("token@with#special!chars")
            .is_err());
    }

    #[test]
    fn test_validate_token_entity() {
        let service = TokenValidationService::new_default();

        // Valid token
        let valid_token =
            PasswordResetToken::new_with_timestamp(1, 1, "valid_token".to_string(), 24);
        assert!(service.validate_token_entity(&valid_token).is_ok());

        // Expired token
        let expired_token =
            PasswordResetToken::new_with_timestamp(1, 1, "expired_token".to_string(), -1);
        assert!(service.validate_token_entity(&expired_token).is_err());

        // Used token
        let mut used_token =
            PasswordResetToken::new_with_timestamp(1, 1, "used_token".to_string(), 24);
        used_token.mark_as_used();
        assert!(service.validate_token_entity(&used_token).is_err());
    }

    #[test]
    fn test_validate_reset_link() {
        let service = TokenValidationService::new_default();

        // Valid HTTPS link
        assert!(service
            .validate_reset_link("https://example.com/reset?token=abc123")
            .is_ok());

        // Invalid HTTP link (requires HTTPS)
        assert!(service
            .validate_reset_link("http://example.com/reset?token=abc123")
            .is_err());

        // XSS attempt
        assert!(service
            .validate_reset_link("https://example.com/<script>alert(1)</script>")
            .is_err());
        assert!(service.validate_reset_link("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_will_expire_soon() {
        let service = TokenValidationService::new_default();

        // Token expires in 1 hour
        let token = PasswordResetToken::new_with_timestamp(1, 1, "token".to_string(), 1);
        assert!(service.will_expire_soon(&token, 2)); // Will expire within 2 hours
        assert!(service.will_expire_soon(&token, 1)); // Will expire within 1 hour (edge case due to num_hours truncation)

        // Token expires in 24 hours (won't expire soon with small threshold)
        let token = PasswordResetToken::new_with_timestamp(1, 1, "token".to_string(), 24);
        assert!(!service.will_expire_soon(&token, 2)); // Won't expire within 2 hours
        assert!(!service.will_expire_soon(&token, 12)); // Won't expire within 12 hours
    }

    #[test]
    fn test_token_strength_score() {
        let service = TokenValidationService::new_default();

        // Strong token (mixed case, digits, special chars, good length)
        let strong_token = "AbC123-DeF456-GhI789-JkL012-MnO345";
        let score = service.get_token_strength_score(strong_token);
        assert!(score > 80);

        // Weak token (short, only lowercase)
        let weak_token = "short";
        let score = service.get_token_strength_score(weak_token);
        assert!(score < 50);
    }

    #[test]
    fn test_comprehensive_validation() {
        let service = TokenValidationService::new_default();

        let token_str = "abc123-def456-ghi789-jkl012";
        let token_entity = PasswordResetToken::new_with_timestamp(1, 1, token_str.to_string(), 24);

        let result = service.validate(token_str, &token_entity).unwrap();
        assert!(result.is_valid);
        assert!(!result.is_expired);
        assert!(!result.is_used);
        assert!(result.validation_errors.is_empty());
    }
}
