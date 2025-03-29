use bookmon::storage::{Storage, Category};
use bookmon::category::store_category;
use chrono::Utc;

#[test]
fn test_category_creation() {
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );

    assert_eq!(category.name, "Fiction");
    assert_eq!(category.description, Some("Fictional books and novels".to_string()));
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
    let stored_category = storage.get_category(&category.id).expect("Category should exist");
    assert_eq!(stored_category.name, category.name);
    assert_eq!(stored_category.description, category.description);
    assert_eq!(stored_category.id, category.id);
    assert!(stored_category.created_on <= Utc::now());
}

#[test]
fn test_category_without_description() {
    let category = Category::new(
        "Non-Fiction".to_string(),
        None,
    );

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
        Category::new("Non-Fiction".to_string(), Some("Real-world books".to_string())),
        Category::new("Science".to_string(), Some("Scientific literature".to_string())),
    ];

    // Store multiple categories
    for category in categories.clone() {
        assert!(store_category(&mut storage, category).is_ok());
    }

    // Verify all categories were stored
    assert_eq!(storage.categories.len(), 3);

    // Verify each category can be retrieved
    for category in categories {
        let stored = storage.get_category(&category.id).expect("Category should exist");
        assert_eq!(stored.name, category.name);
        assert_eq!(stored.description, category.description);
        assert_eq!(stored.id, category.id);
        assert!(stored.created_on <= Utc::now());
    }
} 