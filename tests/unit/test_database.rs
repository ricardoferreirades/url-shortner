use chrono::Utc;
use url_shortner::database::UrlRecord;

#[test]
fn test_url_record_creation() {
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    assert_eq!(url_record.id, 1);
    assert_eq!(url_record.short_code, "abc12345");
    assert_eq!(url_record.original_url, "https://example.com");
    assert_eq!(url_record.created_at, now);
}

#[test]
fn test_url_record_serialization() {
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    // Test serialization
    let json = serde_json::to_string(&url_record).unwrap();
    assert!(json.contains("abc12345"));
    assert!(json.contains("https://example.com"));
    assert!(json.contains("id"));
    assert!(json.contains("short_code"));
    assert!(json.contains("original_url"));
    assert!(json.contains("created_at"));

    // Test deserialization
    let deserialized: UrlRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, 1);
    assert_eq!(deserialized.short_code, "abc12345");
    assert_eq!(deserialized.original_url, "https://example.com");
    assert_eq!(deserialized.created_at, now);
}

#[test]
fn test_url_record_clone() {
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    let cloned = url_record.clone();
    assert_eq!(cloned.id, url_record.id);
    assert_eq!(cloned.short_code, url_record.short_code);
    assert_eq!(cloned.original_url, url_record.original_url);
    assert_eq!(cloned.created_at, url_record.created_at);
}

#[test]
fn test_url_record_debug() {
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    let debug_string = format!("{:?}", url_record);
    assert!(debug_string.contains("abc12345"));
    assert!(debug_string.contains("https://example.com"));
}

#[test]
fn test_url_record_with_special_characters() {
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com/path?query=value&other=123#fragment".to_string(),
        created_at: now,
    };

    assert_eq!(
        url_record.original_url,
        "https://example.com/path?query=value&other=123#fragment"
    );
}

#[test]
fn test_url_record_with_unicode() {
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com/path/with/ünicode/🚀".to_string(),
        created_at: now,
    };

    assert_eq!(
        url_record.original_url,
        "https://example.com/path/with/ünicode/🚀"
    );
}

#[test]
fn test_url_record_field_access() {
    let now = Utc::now();
    let url_record = UrlRecord {
        id: 42,
        short_code: "xyz98765".to_string(),
        original_url: "https://test.example.com/path".to_string(),
        created_at: now,
    };

    // Test individual field access
    assert_eq!(url_record.id, 42);
    assert_eq!(url_record.short_code, "xyz98765");
    assert_eq!(url_record.original_url, "https://test.example.com/path");
    assert_eq!(url_record.created_at, now);
}

#[test]
fn test_url_record_equality() {
    let now = Utc::now();
    let url_record1 = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    let url_record2 = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    // Test that identical records are equal
    assert_eq!(url_record1.id, url_record2.id);
    assert_eq!(url_record1.short_code, url_record2.short_code);
    assert_eq!(url_record1.original_url, url_record2.original_url);
    assert_eq!(url_record1.created_at, url_record2.created_at);
}

#[test]
fn test_url_record_different_ids() {
    let now = Utc::now();
    let url_record1 = UrlRecord {
        id: 1,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    let url_record2 = UrlRecord {
        id: 2,
        short_code: "abc12345".to_string(),
        original_url: "https://example.com".to_string(),
        created_at: now,
    };

    // Test that different IDs make records different
    assert_ne!(url_record1.id, url_record2.id);
    assert_eq!(url_record1.short_code, url_record2.short_code);
    assert_eq!(url_record1.original_url, url_record2.original_url);
}

#[test]
fn test_url_record_serialization_roundtrip() {
    let now = Utc::now();
    let original = UrlRecord {
        id: 123,
        short_code: "test1234".to_string(),
        original_url: "https://test.example.com/very/long/path?query=value#fragment".to_string(),
        created_at: now,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();

    // Deserialize back
    let deserialized: UrlRecord = serde_json::from_str(&json).unwrap();

    // Should be identical
    assert_eq!(original.id, deserialized.id);
    assert_eq!(original.short_code, deserialized.short_code);
    assert_eq!(original.original_url, deserialized.original_url);
    assert_eq!(original.created_at, deserialized.created_at);
}
