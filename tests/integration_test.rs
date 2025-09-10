use chrono::Utc;
use url_shortner::database::UrlRecord;
use url_shortner::shortener::{generate_short_code, ShortenUrlRequest, ShortenUrlResponse};

/// Integration test for the URL shortener feature
/// This test verifies the complete flow of URL shortening functionality
#[test]
fn test_url_shortener_integration() {
    // Test the complete URL shortening flow

    // 1. Test URL input validation and processing
    let test_urls = vec![
        "https://example.com",
        "https://google.com/search?q=rust+programming",
        "https://github.com/rust-lang/rust",
        "http://localhost:3000",
        "https://example.com/path/with/ünicode/🚀",
        "https://example.com/path?query=value&other=123#fragment",
    ];

    for url in test_urls {
        // 2. Test short code generation
        let short_code = generate_short_code(url);

        // Verify short code properties
        assert_eq!(
            short_code.len(),
            8,
            "Short code should be exactly 8 characters"
        );
        assert!(
            short_code.chars().all(|c| c.is_ascii_hexdigit()),
            "Short code should be hexadecimal"
        );

        // 3. Test request creation
        let request = ShortenUrlRequest {
            url: url.to_string(),
        };
        assert_eq!(request.url, url);

        // 4. Test response creation
        let short_url = format!("http://localhost:8000/{}", short_code);
        let response = ShortenUrlResponse {
            short_url: short_url.clone(),
            original_url: url.to_string(),
        };
        assert_eq!(response.short_url, short_url);
        assert_eq!(response.original_url, url);

        // 5. Test database record creation
        let now = Utc::now();
        let url_record = UrlRecord {
            id: 1,
            short_code: short_code.clone(),
            original_url: url.to_string(),
            created_at: now,
        };
        assert_eq!(url_record.short_code, short_code);
        assert_eq!(url_record.original_url, url);

        // 6. Test serialization/deserialization
        let request_json = serde_json::to_string(&request).unwrap();
        let deserialized_request: ShortenUrlRequest = serde_json::from_str(&request_json).unwrap();
        assert_eq!(deserialized_request.url, url);

        let response_json = serde_json::to_string(&response).unwrap();
        let deserialized_response: ShortenUrlResponse =
            serde_json::from_str(&response_json).unwrap();
        assert_eq!(deserialized_response.short_url, short_url);
        assert_eq!(deserialized_response.original_url, url);

        let record_json = serde_json::to_string(&url_record).unwrap();
        let deserialized_record: UrlRecord = serde_json::from_str(&record_json).unwrap();
        assert_eq!(deserialized_record.short_code, short_code);
        assert_eq!(deserialized_record.original_url, url);
    }
}

#[test]
fn test_url_shortener_deterministic_behavior() {
    // Test that the URL shortener behaves deterministically
    let test_url = "https://example.com/deterministic/test";

    // Generate the same short code multiple times
    let short_code1 = generate_short_code(test_url);
    let short_code2 = generate_short_code(test_url);
    let short_code3 = generate_short_code(test_url);

    // All should be identical
    assert_eq!(short_code1, short_code2);
    assert_eq!(short_code2, short_code3);
    assert_eq!(short_code1, short_code3);

    // Test that different URLs generate different short codes
    let different_url = "https://example.com/different/test";
    let different_short_code = generate_short_code(different_url);
    assert_ne!(short_code1, different_short_code);
}

#[test]
fn test_url_shortener_collision_resistance() {
    // Test that the URL shortener is collision-resistant
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
fn test_url_shortener_unicode_support() {
    // Test that the URL shortener handles Unicode correctly
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
fn test_url_shortener_edge_cases() {
    // Test edge cases for the URL shortener
    let long_url = "https://example.com/".repeat(1000);
    let edge_cases = vec![
        "",        // Empty URL
        &long_url, // Very long URL
        "https://example.com/path with spaces",
        "https://example.com/path+with+plus+signs",
        "https://example.com/path%20with%20encoded%20spaces",
        "javascript:alert('xss')", // Potentially malicious URL
        "ftp://example.com",       // Non-HTTP protocol
    ];

    for url in edge_cases {
        let short_code = generate_short_code(url);
        assert_eq!(short_code.len(), 8);
        assert!(short_code.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn test_url_shortener_api_contract() {
    // Test that the API contract is maintained
    let test_url = "https://example.com/api/test";

    // Test request structure
    let request = ShortenUrlRequest {
        url: test_url.to_string(),
    };
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains("url"));
    assert!(request_json.contains(test_url));

    // Test response structure
    let short_code = generate_short_code(test_url);
    let response = ShortenUrlResponse {
        short_url: format!("http://localhost:8000/{}", short_code),
        original_url: test_url.to_string(),
    };
    let response_json = serde_json::to_string(&response).unwrap();
    assert!(response_json.contains("short_url"));
    assert!(response_json.contains("original_url"));
    assert!(response_json.contains("http://localhost:8000/"));
    assert!(response_json.contains(test_url));
}

#[test]
fn test_url_shortener_performance() {
    // Test that the URL shortener performs well
    let start = std::time::Instant::now();

    // Generate 1000 short codes
    for i in 0..1000 {
        let url = format!("https://example.com/performance/test/{}", i);
        let _short_code = generate_short_code(&url);
    }

    let duration = start.elapsed();

    // Should complete in reasonable time (less than 1 second)
    assert!(duration.as_millis() < 1000, "URL shortening should be fast");
}

#[test]
fn test_url_shortener_data_integrity() {
    // Test that data integrity is maintained throughout the process
    let original_url = "https://example.com/data/integrity/test";

    // Generate short code
    let short_code = generate_short_code(original_url);

    // Create request
    let request = ShortenUrlRequest {
        url: original_url.to_string(),
    };

    // Create response
    let response = ShortenUrlResponse {
        short_url: format!("http://localhost:8000/{}", short_code),
        original_url: original_url.to_string(),
    };

    // Create database record
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 1,
        short_code: short_code.clone(),
        original_url: original_url.to_string(),
        created_at: now,
    };

    // Verify data integrity
    assert_eq!(request.url, original_url);
    assert_eq!(response.original_url, original_url);
    assert_eq!(url_record.original_url, original_url);
    assert_eq!(url_record.short_code, short_code);
    assert!(response.short_url.contains(&short_code));
}
