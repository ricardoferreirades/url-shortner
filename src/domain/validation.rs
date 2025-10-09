use thiserror::Error;
use url::Url;
use validator::{Validate, ValidationError};

/// Custom error types for validation failures
#[derive(Error, Debug)]
pub enum ValidationErrorType {
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error("URL scheme not allowed: {0}. Only http and https are allowed")]
    InvalidScheme(String),

    #[error("URL too long: {0} characters. Maximum allowed: {1}")]
    UrlTooLong(usize, usize),

    #[error("URL too short: {0} characters. Minimum required: {1}")]
    UrlTooShort(usize, usize),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("Malicious URL pattern detected: {0}")]
    MaliciousPattern(String),

    #[error("Empty URL provided")]
    EmptyUrl,

    #[error("Validation failed: {0}")]
    General(String),
}

/// Configuration for URL validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_url_length: usize,
    pub min_url_length: usize,
    pub allowed_schemes: Vec<String>,
    pub blocked_patterns: Vec<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_url_length: 2048,
            min_url_length: 10,
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            blocked_patterns: vec![
                "javascript:".to_string(),
                "data:".to_string(),
                "file:".to_string(),
                "ftp:".to_string(),
                "mailto:".to_string(),
                "tel:".to_string(),
                "sms:".to_string(),
            ],
        }
    }
}

/// Validates and sanitizes a URL
pub fn validate_url(url: &str, config: &ValidationConfig) -> Result<String, ValidationErrorType> {
    // Check if URL is empty
    if url.trim().is_empty() {
        return Err(ValidationErrorType::EmptyUrl);
    }

    let trimmed_url = url.trim();

    // Check URL length
    if trimmed_url.len() > config.max_url_length {
        return Err(ValidationErrorType::UrlTooLong(
            trimmed_url.len(),
            config.max_url_length,
        ));
    }

    if trimmed_url.len() < config.min_url_length {
        return Err(ValidationErrorType::UrlTooShort(
            trimmed_url.len(),
            config.min_url_length,
        ));
    }

    // Check for malicious patterns
    let lower_url = trimmed_url.to_lowercase();
    for pattern in &config.blocked_patterns {
        if lower_url.starts_with(pattern) {
            return Err(ValidationErrorType::MaliciousPattern(pattern.clone()));
        }
    }

    // Parse URL
    let parsed_url = match Url::parse(trimmed_url) {
        Ok(url) => url,
        Err(e) => return Err(ValidationErrorType::InvalidUrl(e.to_string())),
    };

    // Check scheme
    let scheme = parsed_url.scheme().to_lowercase();
    if !config.allowed_schemes.contains(&scheme) {
        return Err(ValidationErrorType::InvalidScheme(scheme));
    }

    // Check if URL has a host
    if parsed_url.host().is_none() {
        return Err(ValidationErrorType::InvalidUrl(
            "URL must have a host".to_string(),
        ));
    }

    // Additional domain validation
    if let Some(host) = parsed_url.host() {
        let host_str = host.to_string();

        // Check for localhost and private IPs (optional security measure)
        if host_str == "localhost"
            || host_str.starts_with("127.")
            || host_str.starts_with("192.168.")
        {
            // Allow localhost for development, but log it
            tracing::warn!("Local URL detected: {}", host_str);
        }

        // Check for suspicious domains (basic check)
        if host_str.contains("..") || host_str.starts_with('.') || host_str.ends_with('.') {
            return Err(ValidationErrorType::InvalidDomain(host_str));
        }
    }

    // Return the normalized URL
    Ok(parsed_url.to_string())
}

/// Validates a short code
#[allow(dead_code)]
pub fn validate_short_code(short_code: &str) -> Result<String, ValidationErrorType> {
    if short_code.trim().is_empty() {
        return Err(ValidationErrorType::EmptyUrl);
    }

    let trimmed = short_code.trim();

    // Check length (should be reasonable for short codes)
    if trimmed.len() < 3 || trimmed.len() > 50 {
        return Err(ValidationErrorType::General(format!(
            "Short code must be between 3 and 50 characters, got {}",
            trimmed.len()
        )));
    }

    // Check for valid characters (alphanumeric and some safe characters)
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ValidationErrorType::General(
            "Short code can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

/// Enhanced request validation with validator crate
#[derive(Debug, Validate, serde::Deserialize)]
pub struct ShortenUrlRequest {
    #[validate(length(
        min = 10,
        max = 2048,
        message = "URL must be between 10 and 2048 characters"
    ))]
    #[validate(custom(function = "validate_url_format"))]
    pub url: String,
}

/// Custom validator for URL format using our validation logic
fn validate_url_format(url: &str) -> Result<(), ValidationError> {
    let config = ValidationConfig::default();
    match validate_url(url, &config) {
        Ok(_) => Ok(()),
        Err(e) => {
            let mut error = ValidationError::new("invalid_url");
            error.message = Some(std::borrow::Cow::from(e.to_string()));
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_success() {
        let config = ValidationConfig::default();

        assert!(validate_url("https://example.com", &config).is_ok());
        assert!(validate_url("http://google.com/path", &config).is_ok());
        assert!(validate_url("https://subdomain.example.com/path?query=value", &config).is_ok());
    }

    #[test]
    fn test_validate_url_empty() {
        let config = ValidationConfig::default();

        assert!(matches!(
            validate_url("", &config),
            Err(ValidationErrorType::EmptyUrl)
        ));
        assert!(matches!(
            validate_url("   ", &config),
            Err(ValidationErrorType::EmptyUrl)
        ));
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        let config = ValidationConfig::default();

        assert!(matches!(
            validate_url("javascript:alert('xss')", &config),
            Err(ValidationErrorType::MaliciousPattern(_))
        ));
        assert!(matches!(
            validate_url("data:text/html,<script>alert('xss')</script>", &config),
            Err(ValidationErrorType::MaliciousPattern(_))
        ));
        assert!(matches!(
            validate_url("ftp://example.com", &config),
            Err(ValidationErrorType::MaliciousPattern(_))
        ));
        assert!(matches!(
            validate_url("ssh://example.com", &config),
            Err(ValidationErrorType::InvalidScheme(_))
        ));
    }

    #[test]
    fn test_validate_url_too_long() {
        let mut config = ValidationConfig::default();
        config.max_url_length = 50;

        let long_url = "https://example.com/".to_string() + &"a".repeat(100);
        assert!(matches!(
            validate_url(&long_url, &config),
            Err(ValidationErrorType::UrlTooLong(_, _))
        ));
    }

    #[test]
    fn test_validate_url_too_short() {
        let mut config = ValidationConfig::default();
        config.min_url_length = 20;

        assert!(matches!(
            validate_url("https://a.co", &config),
            Err(ValidationErrorType::UrlTooShort(_, _))
        ));
    }

    #[test]
    fn test_validate_short_code() {
        assert!(validate_short_code("abc123").is_ok());
        assert!(validate_short_code("my-short_code").is_ok());
        assert!(validate_short_code("").is_err());
        assert!(validate_short_code("ab").is_err()); // too short
        assert!(validate_short_code(&"a".repeat(100)).is_err()); // too long
        assert!(validate_short_code("invalid@code").is_err()); // invalid characters
    }
}
