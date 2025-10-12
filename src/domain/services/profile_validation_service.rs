use crate::domain::entities::ProfilePrivacy;
use regex::Regex;
use thiserror::Error;
use url::Url;

/// Profile validation service for sanitizing and validating user profile data
pub struct ProfileValidationService {
    name_regex: Regex,
    bio_max_length: usize,
    website_max_length: usize,
    location_max_length: usize,
}

/// Profile validation errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ProfileValidationError {
    #[error("Invalid first name: {0}")]
    InvalidFirstName(String),

    #[error("Invalid last name: {0}")]
    InvalidLastName(String),

    #[error("Invalid bio: {0}")]
    InvalidBio(String),

    #[error("Invalid website URL: {0}")]
    InvalidWebsite(String),

    #[error("Invalid location: {0}")]
    InvalidLocation(String),

    #[error("Invalid avatar URL: {0}")]
    InvalidAvatarUrl(String),

    #[error("Profile data too long: {0}")]
    DataTooLong(String),

    #[error("Invalid characters in field: {0}")]
    InvalidCharacters(String),
}

/// Validated and sanitized profile data
#[derive(Debug, Clone)]
pub struct ValidatedProfileData {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub privacy: ProfilePrivacy,
}

#[allow(dead_code)]
impl ProfileValidationService {
    /// Create a new profile validation service
    pub fn new() -> Self {
        Self {
            name_regex: Regex::new(r"^[a-zA-Z\s\-'\.]+$").unwrap(),
            bio_max_length: 500,
            website_max_length: 2000,
            location_max_length: 100,
        }
    }

    /// Validate and sanitize profile data
    pub fn validate_profile_data(
        &self,
        first_name: Option<String>,
        last_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        website: Option<String>,
        location: Option<String>,
        privacy: ProfilePrivacy,
    ) -> Result<ValidatedProfileData, ProfileValidationError> {
        let validated_first_name = if let Some(name) = first_name {
            Some(self.validate_and_sanitize_name(name, "first name")?)
        } else {
            None
        };

        let validated_last_name = if let Some(name) = last_name {
            Some(self.validate_and_sanitize_name(name, "last name")?)
        } else {
            None
        };

        let validated_bio = if let Some(bio) = bio {
            Some(self.validate_and_sanitize_bio(bio)?)
        } else {
            None
        };

        let validated_avatar_url = if let Some(url) = avatar_url {
            Some(self.validate_and_sanitize_url(url, "avatar")?)
        } else {
            None
        };

        let validated_website = if let Some(website) = website {
            Some(self.validate_and_sanitize_website(website)?)
        } else {
            None
        };

        let validated_location = if let Some(location) = location {
            Some(self.validate_and_sanitize_location(location)?)
        } else {
            None
        };

        Ok(ValidatedProfileData {
            first_name: validated_first_name,
            last_name: validated_last_name,
            bio: validated_bio,
            avatar_url: validated_avatar_url,
            website: validated_website,
            location: validated_location,
            privacy,
        })
    }

    /// Validate and sanitize name fields
    fn validate_and_sanitize_name(
        &self,
        name: String,
        field_name: &str,
    ) -> Result<String, ProfileValidationError> {
        let trimmed = name.trim().to_string();

        if trimmed.is_empty() {
            return Err(ProfileValidationError::InvalidCharacters(format!(
                "{} cannot be empty",
                field_name
            )));
        }

        if trimmed.len() > 50 {
            return Err(ProfileValidationError::DataTooLong(format!(
                "{} cannot exceed 50 characters",
                field_name
            )));
        }

        if !self.name_regex.is_match(&trimmed) {
            return Err(ProfileValidationError::InvalidCharacters(
                format!("{} contains invalid characters. Only letters, spaces, hyphens, apostrophes, and periods are allowed", field_name)
            ));
        }

        // Sanitize: remove extra spaces and normalize
        let sanitized = trimmed
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(sanitized)
    }

    /// Validate and sanitize bio
    fn validate_and_sanitize_bio(&self, bio: String) -> Result<String, ProfileValidationError> {
        let trimmed = bio.trim().to_string();

        if trimmed.len() > self.bio_max_length {
            return Err(ProfileValidationError::DataTooLong(format!(
                "Bio cannot exceed {} characters",
                self.bio_max_length
            )));
        }

        // Basic HTML sanitization - remove potentially dangerous tags
        let sanitized = self.sanitize_html(&trimmed);

        Ok(sanitized)
    }

    /// Validate and sanitize avatar URL
    fn validate_and_sanitize_url(
        &self,
        url: String,
        field_name: &str,
    ) -> Result<String, ProfileValidationError> {
        let trimmed = url.trim().to_string();

        if trimmed.is_empty() {
            return Ok(trimmed);
        }

        if trimmed.len() > self.website_max_length {
            return Err(ProfileValidationError::DataTooLong(format!(
                "{} URL cannot exceed {} characters",
                field_name, self.website_max_length
            )));
        }

        // Validate URL format
        if let Err(_) = Url::parse(&trimmed) {
            return Err(ProfileValidationError::InvalidAvatarUrl(format!(
                "Invalid {} URL format",
                field_name
            )));
        }

        // Check for allowed protocols
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return Err(ProfileValidationError::InvalidAvatarUrl(format!(
                "{} URL must use http:// or https:// protocol",
                field_name
            )));
        }

        Ok(trimmed)
    }

    /// Validate and sanitize website URL
    fn validate_and_sanitize_website(
        &self,
        website: String,
    ) -> Result<String, ProfileValidationError> {
        let trimmed = website.trim().to_string();

        if trimmed.is_empty() {
            return Ok(trimmed);
        }

        if trimmed.len() > self.website_max_length {
            return Err(ProfileValidationError::DataTooLong(format!(
                "Website URL cannot exceed {} characters",
                self.website_max_length
            )));
        }

        // Add https:// if no protocol is specified
        let url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed
        } else {
            format!("https://{}", trimmed)
        };

        // Validate URL format
        if let Err(_) = Url::parse(&url) {
            return Err(ProfileValidationError::InvalidWebsite(
                "Invalid website URL format".to_string(),
            ));
        }

        Ok(url)
    }

    /// Validate and sanitize location
    fn validate_and_sanitize_location(
        &self,
        location: String,
    ) -> Result<String, ProfileValidationError> {
        let trimmed = location.trim().to_string();

        if trimmed.len() > self.location_max_length {
            return Err(ProfileValidationError::DataTooLong(format!(
                "Location cannot exceed {} characters",
                self.location_max_length
            )));
        }

        // Basic sanitization - remove potentially dangerous characters
        let sanitized = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,-()[]{}".contains(*c))
            .collect::<String>();

        if sanitized.is_empty() && !trimmed.is_empty() {
            return Err(ProfileValidationError::InvalidCharacters(
                "Location contains only invalid characters".to_string(),
            ));
        }

        Ok(sanitized)
    }

    /// Basic HTML sanitization
    fn sanitize_html(&self, input: &str) -> String {
        // Remove potentially dangerous HTML tags and attributes
        let dangerous_tags = [
            "script", "object", "embed", "iframe", "form", "input", "button",
        ];
        let mut result = input.to_string();

        for tag in &dangerous_tags {
            let pattern = format!(r"<{}[^>]*>.*?</{}>", tag, tag);
            if let Ok(regex) = Regex::new(&pattern) {
                result = regex.replace_all(&result, "").to_string();
            }
        }

        // Remove any remaining HTML tags
        let html_tag_regex = Regex::new(r"<[^>]*>").unwrap();
        result = html_tag_regex.replace_all(&result, "").to_string();

        result
    }

    /// Validate profile privacy setting
    pub fn validate_privacy(
        &self,
        privacy: ProfilePrivacy,
    ) -> Result<ProfilePrivacy, ProfileValidationError> {
        // Privacy enum is already validated by the type system
        Ok(privacy)
    }

    /// Check if profile data is complete enough for public display
    pub fn is_profile_complete(&self, data: &ValidatedProfileData) -> bool {
        data.first_name.is_some() && data.last_name.is_some()
    }

    /// Get profile completeness score (0-100)
    pub fn get_completeness_score(&self, data: &ValidatedProfileData) -> u8 {
        let mut score = 0;
        let total_fields = 6;

        if data.first_name.is_some() {
            score += 1;
        }
        if data.last_name.is_some() {
            score += 1;
        }
        if data.bio.is_some() {
            score += 1;
        }
        if data.avatar_url.is_some() {
            score += 1;
        }
        if data.website.is_some() {
            score += 1;
        }
        if data.location.is_some() {
            score += 1;
        }

        (score * 100 / total_fields) as u8
    }
}

impl Default for ProfileValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        let service = ProfileValidationService::new();

        // Valid names
        assert!(service
            .validate_and_sanitize_name("John".to_string(), "first name")
            .is_ok());
        assert!(service
            .validate_and_sanitize_name("O'Connor".to_string(), "last name")
            .is_ok());
        assert!(service
            .validate_and_sanitize_name("Mary-Jane".to_string(), "first name")
            .is_ok());
        assert!(service
            .validate_and_sanitize_name("Dr. Smith".to_string(), "last name")
            .is_ok());

        // Invalid names
        assert!(service
            .validate_and_sanitize_name("John123".to_string(), "first name")
            .is_err());
        assert!(service
            .validate_and_sanitize_name("John@Doe".to_string(), "first name")
            .is_err());
        assert!(service
            .validate_and_sanitize_name("".to_string(), "first name")
            .is_err());
        assert!(service
            .validate_and_sanitize_name("   ".to_string(), "first name")
            .is_err());
    }

    #[test]
    fn test_validate_bio() {
        let service = ProfileValidationService::new();

        // Valid bio
        let valid_bio = "Software developer with 5 years of experience".to_string();
        assert!(service.validate_and_sanitize_bio(valid_bio).is_ok());

        // Bio too long
        let long_bio = "a".repeat(600);
        assert!(service.validate_and_sanitize_bio(long_bio).is_err());

        // Bio with HTML (should be sanitized)
        let html_bio = "Hello <script>alert('xss')</script> world".to_string();
        let result = service.validate_and_sanitize_bio(html_bio).unwrap();
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_validate_website() {
        let service = ProfileValidationService::new();

        // Valid websites
        assert!(service
            .validate_and_sanitize_website("https://example.com".to_string())
            .is_ok());
        assert!(service
            .validate_and_sanitize_website("example.com".to_string())
            .is_ok());
        assert!(service
            .validate_and_sanitize_website("".to_string())
            .is_ok());

        // Invalid websites
        assert!(service
            .validate_and_sanitize_website("not-a-url".to_string())
            .is_err());
        assert!(service
            .validate_and_sanitize_website("ftp://example.com".to_string())
            .is_err());
    }

    #[test]
    fn test_validate_location() {
        let service = ProfileValidationService::new();

        // Valid locations
        assert!(service
            .validate_and_sanitize_location("New York, NY".to_string())
            .is_ok());
        assert!(service
            .validate_and_sanitize_location("San Francisco, CA, USA".to_string())
            .is_ok());
        assert!(service
            .validate_and_sanitize_location("".to_string())
            .is_ok());

        // Invalid locations
        let long_location = "a".repeat(150);
        assert!(service
            .validate_and_sanitize_location(long_location)
            .is_err());
    }

    #[test]
    fn test_completeness_score() {
        let service = ProfileValidationService::new();

        let complete_data = ValidatedProfileData {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            bio: Some("Software developer".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            website: Some("https://johndoe.com".to_string()),
            location: Some("New York, NY".to_string()),
            privacy: ProfilePrivacy::Public,
        };

        assert_eq!(service.get_completeness_score(&complete_data), 100);

        let partial_data = ValidatedProfileData {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            bio: None,
            avatar_url: None,
            website: None,
            location: None,
            privacy: ProfilePrivacy::Public,
        };

        assert_eq!(service.get_completeness_score(&partial_data), 33);
    }
}
