use url_shortner::shortener::{generate_short_code, ShortenUrlRequest, ShortenUrlResponse};

#[test]
fn test_generate_short_code_basic() {
    let url = "https://example.com/very/long/url";
    let short_code = generate_short_code(url);

    // Should be exactly 8 characters
    assert_eq!(short_code.len(), 8);

    // Should be hexadecimal
    assert!(short_code.chars().all(|c| c.is_ascii_hexdigit()));

    // Same URL should always generate same short code
    let short_code2 = generate_short_code(url);
    assert_eq!(short_code, short_code2);

    // Different URLs should generate different short codes
    let different_url = "https://different.com/url";
    let different_short_code = generate_short_code(different_url);
    assert_ne!(short_code, different_short_code);
}

#[test]
fn test_generate_short_code_with_different_urls() {
    let urls = vec![
        "https://example.com",
        "https://google.com",
        "https://github.com/rust-lang/rust",
        "http://localhost:3000",
        "ftp://files.example.com",
    ];

    let mut short_codes = Vec::new();

    for url in urls {
        let short_code = generate_short_code(url);
        assert_eq!(short_code.len(), 8);
        assert!(short_code.chars().all(|c| c.is_ascii_hexdigit()));
        short_codes.push(short_code);
    }

    // All short codes should be unique
    for i in 0..short_codes.len() {
        for j in i + 1..short_codes.len() {
            assert_ne!(short_codes[i], short_codes[j]);
        }
    }
}

#[test]
fn test_generate_short_code_with_special_characters() {
    let urls_with_special_chars = vec![
        "https://example.com/path?query=value&other=123",
        "https://example.com/path#fragment",
        "https://example.com/path with spaces",
        "https://example.com/path/with/ünicode",
        "https://example.com/path/with/emoji/🚀",
    ];

    for url in urls_with_special_chars {
        let short_code = generate_short_code(url);
        assert_eq!(short_code.len(), 8);
        assert!(short_code.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn test_shorten_url_request_creation() {
    let request = ShortenUrlRequest {
        url: "https://example.com".to_string(),
    };

    assert_eq!(request.url, "https://example.com");
}

#[test]
fn test_shorten_url_request_serialization() {
    let request = ShortenUrlRequest {
        url: "https://example.com".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("https://example.com"));
    assert!(json.contains("url"));

    // Test deserialization
    let deserialized: ShortenUrlRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.url, "https://example.com");
}

#[test]
fn test_shorten_url_response_creation() {
    let response = ShortenUrlResponse {
        short_url: "http://localhost:8000/abc12345".to_string(),
        original_url: "https://example.com".to_string(),
    };

    assert_eq!(response.short_url, "http://localhost:8000/abc12345");
    assert_eq!(response.original_url, "https://example.com");
}

#[test]
fn test_shorten_url_response_serialization() {
    let response = ShortenUrlResponse {
        short_url: "http://localhost:8000/abc12345".to_string(),
        original_url: "https://example.com".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("http://localhost:8000/abc12345"));
    assert!(json.contains("https://example.com"));
    assert!(json.contains("short_url"));
    assert!(json.contains("original_url"));

    // Test deserialization
    let deserialized: ShortenUrlResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.short_url, "http://localhost:8000/abc12345");
    assert_eq!(deserialized.original_url, "https://example.com");
}

#[test]
fn test_generate_short_code_properties() {
    // Test that short codes are always 8 characters
    for i in 0..100 {
        let url = format!("https://example.com/url{}", i);
        let short_code = generate_short_code(&url);
        assert_eq!(
            short_code.len(),
            8,
            "Short code should always be 8 characters"
        );
    }

    // Test that short codes are deterministic
    let url = "https://example.com/deterministic";
    let code1 = generate_short_code(url);
    let code2 = generate_short_code(url);
    assert_eq!(
        code1, code2,
        "Same URL should always generate same short code"
    );
}

#[test]
fn test_short_code_collision_resistance() {
    // Test that short codes are reasonably collision-resistant
    let base_url = "https://example.com";
    let mut short_codes = std::collections::HashSet::new();

    // Generate 1000 short codes with slightly different URLs
    for i in 0..1000 {
        let url = format!("{}/path/{}", base_url, i);
        let short_code = generate_short_code(&url);
        short_codes.insert(short_code);
    }

    // All short codes should be unique
    assert_eq!(short_codes.len(), 1000, "All short codes should be unique");
}

#[test]
fn test_unicode_handling() {
    // Test that Unicode characters are handled correctly
    let unicode_urls = vec![
        "https://example.com/ünicode",
        "https://example.com/🚀/rocket",
        "https://example.com/中文",
        "https://example.com/العربية",
        "https://example.com/русский",
    ];

    for url in unicode_urls {
        let short_code = generate_short_code(url);
        assert_eq!(short_code.len(), 8);
        assert!(short_code.chars().all(|c| c.is_ascii_hexdigit()));

        // Test that the same Unicode URL generates the same short code
        let short_code2 = generate_short_code(url);
        assert_eq!(short_code, short_code2);
    }
}

#[test]
fn test_empty_and_very_long_urls() {
    // Test edge cases with empty and very long URLs
    let empty_url = "";
    let very_long_url = "https://example.com/".repeat(1000);

    // Empty URL should still generate a short code
    let empty_short_code = generate_short_code(empty_url);
    assert_eq!(empty_short_code.len(), 8);
    assert!(empty_short_code.chars().all(|c| c.is_ascii_hexdigit()));

    // Very long URL should generate a short code
    let long_short_code = generate_short_code(&very_long_url);
    assert_eq!(long_short_code.len(), 8);
    assert!(long_short_code.chars().all(|c| c.is_ascii_hexdigit()));

    // Both should be different
    assert_ne!(empty_short_code, long_short_code);
}
