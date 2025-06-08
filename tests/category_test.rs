use bookmon::category::store_category;
use bookmon::storage::{Category, Storage};
use chrono::{DateTime, Utc};
use serde_json;

#[test]
fn test_category_creation() {
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );

    assert_eq!(category.name, "Fiction");
    assert_eq!(
        category.description,
        Some("Fictional books and novels".to_string())
    );
    assert!(!category.id.is_empty());
    assert!(category.created_on <= Utc::now());
}

#[test]
fn test_category_storage() {
    let mut storage = Storage::new();
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );

    // Test storing a category
    assert!(store_category(&mut storage, category.clone()).is_ok());
    assert_eq!(storage.categories.len(), 1);

    // Test retrieving the category
    let stored_category = storage
        .get_category(&category.id)
        .expect("Category should exist");
    assert_eq!(stored_category.name, category.name);
    assert_eq!(stored_category.description, category.description);
    assert_eq!(stored_category.id, category.id);
    assert!(stored_category.created_on <= Utc::now());
}

#[test]
fn test_category_without_description() {
    let category = Category::new("Non-Fiction".to_string(), None);

    assert_eq!(category.name, "Non-Fiction");
    assert!(category.description.is_none());
    assert!(!category.id.is_empty());
    assert!(category.created_on <= Utc::now());
}

#[test]
fn test_multiple_categories() {
    let mut storage = Storage::new();

    let categories = vec![
        Category::new("Fiction".to_string(), Some("Fictional books".to_string())),
        Category::new(
            "Non-Fiction".to_string(),
            Some("Real-world books".to_string()),
        ),
        Category::new(
            "Science".to_string(),
            Some("Scientific literature".to_string()),
        ),
    ];

    // Store multiple categories
    for category in categories.clone() {
        assert!(store_category(&mut storage, category).is_ok());
    }

    // Verify all categories were stored
    assert_eq!(storage.categories.len(), 3);

    // Verify each category can be retrieved
    for category in categories {
        let stored = storage
            .get_category(&category.id)
            .expect("Category should exist");
        assert_eq!(stored.name, category.name);
        assert_eq!(stored.description, category.description);
        assert_eq!(stored.id, category.id);
        assert!(stored.created_on <= Utc::now());
    }
}

#[test]
fn test_category_timestamp_format() {
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&category).expect("Failed to serialize category");

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

    // Make sure it can be deserialized back to the original category
    let deserialized: Category =
        serde_json::from_str(&json).expect("Failed to deserialize category");
    assert_eq!(deserialized.created_on, category.created_on);
}
