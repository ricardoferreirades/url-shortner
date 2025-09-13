use serde::{Deserialize, Serialize};
use std::fmt;

/// Domain entity representing a short code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ShortCode {
    value: String,
}

impl ShortCode {
    /// Create a new short code with validation
    pub fn new(value: String) -> Result<Self, ShortCodeError> {
        if value.is_empty() {
            return Err(ShortCodeError::Empty);
        }
        
        if value.len() > 50 {
            return Err(ShortCodeError::TooLong);
        }
        
        // Check for invalid characters
        if !value.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ShortCodeError::InvalidCharacters);
        }
        
        Ok(ShortCode { value })
    }

    /// Create a short code from a string without validation (for internal use)
    pub fn from_string_unchecked(value: String) -> Self {
        ShortCode { value }
    }

    /// Get the string value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if this is a custom short code
    pub fn is_custom(&self) -> bool {
        // Custom codes are typically longer and more readable
        self.value.len() > 6 || !self.value.chars().all(|c| c.is_alphanumeric())
    }

    /// Check if this is a generated short code
    pub fn is_generated(&self) -> bool {
        !self.is_custom()
    }
}

impl fmt::Display for ShortCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<ShortCode> for String {
    fn from(short_code: ShortCode) -> Self {
        short_code.value
    }
}

/// Errors that can occur when creating a short code
#[derive(Debug, Clone, PartialEq)]
pub enum ShortCodeError {
    Empty,
    TooLong,
    InvalidCharacters,
}

impl fmt::Display for ShortCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortCodeError::Empty => write!(f, "Short code cannot be empty"),
            ShortCodeError::TooLong => write!(f, "Short code is too long (max 50 characters)"),
            ShortCodeError::InvalidCharacters => write!(f, "Short code contains invalid characters (only alphanumeric, -, _ allowed)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_code_creation_valid() {
        let short_code = ShortCode::new("abc123".to_string()).unwrap();
        assert_eq!(short_code.value(), "abc123");
        assert!(short_code.is_generated());
    }

    #[test]
    fn test_short_code_creation_with_special_chars() {
        let short_code = ShortCode::new("my-url".to_string()).unwrap();
        assert_eq!(short_code.value(), "my-url");
        assert!(short_code.is_custom());
    }

    #[test]
    fn test_short_code_creation_empty() {
        let result = ShortCode::new("".to_string());
        assert_eq!(result, Err(ShortCodeError::Empty));
    }

    #[test]
    fn test_short_code_creation_too_long() {
        let long_code = "a".repeat(51);
        let result = ShortCode::new(long_code);
        assert_eq!(result, Err(ShortCodeError::TooLong));
    }

    #[test]
    fn test_short_code_creation_invalid_characters() {
        let result = ShortCode::new("abc@123".to_string());
        assert_eq!(result, Err(ShortCodeError::InvalidCharacters));
    }

    #[test]
    fn test_short_code_display() {
        let short_code = ShortCode::new("abc123".to_string()).unwrap();
        assert_eq!(format!("{}", short_code), "abc123");
    }

    #[test]
    fn test_short_code_from_string() {
        let short_code = ShortCode::new("abc123".to_string()).unwrap();
        let string: String = short_code.into();
        assert_eq!(string, "abc123");
    }
}
