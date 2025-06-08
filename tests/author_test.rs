use bookmon::author::{get_author_by_id, store_author};
use bookmon::storage::{Author, Storage};
use chrono::{DateTime, Utc};
use serde_json;

#[test]
fn test_store_and_retrieve_author() {
    let mut storage = Storage::new();
    let author = Author::new("Test Author".to_string());

    assert!(store_author(&mut storage, author.clone()).is_ok());

    let retrieved = get_author_by_id(&storage, &author.id).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.name, "Test Author");
    assert!(retrieved.created_on <= Utc::now());
}

#[test]
fn test_get_nonexistent_author() {
    let mut storage = Storage::new();
    let author = Author::new("Test Author".to_string());

    assert!(store_author(&mut storage, author).is_ok());

    let retrieved = get_author_by_id(&storage, "nonexistent-id").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_author_creation() {
    let author = Author::new("Test Author".to_string());
    assert_eq!(author.name, "Test Author");
    assert!(!author.id.is_empty());
    assert!(author.created_on <= Utc::now());
}

#[test]
fn test_author_timestamp_format() {
    let author = Author::new("Test Author".to_string());

    // Serialize to JSON
    let json = serde_json::to_string(&author).expect("Failed to serialize author");

    // Parse the JSON to a Value to extract the timestamp string
    let value: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");
    let timestamp_str = value["created_on"]
        .as_str()
        .expect("created_on should be a string");

    // Parse the timestamp string - this will fail if it's not a valid ISO 8601 format
    let parsed_date: DateTime<Utc> = DateTime::parse_from_rfc3339(timestamp_str)
        .expect("Timestamp should be in RFC 3339/ISO 8601 format")
        .into();

    // Verify timezone is UTC
    assert_eq!(parsed_date.timezone(), Utc, "Timestamp should be in UTC");

    // Make sure it can be deserialized back to the original author
    let deserialized: Author = serde_json::from_str(&json).expect("Failed to deserialize author");
    assert_eq!(deserialized.created_on, author.created_on);
}
