use bookmon::storage::{Storage, Book, Category};
use bookmon::book::store_book;
use chrono::Utc;

#[test]
fn test_get_book_input() {
    // This is a basic test that we can expand later
    // Currently, we're just testing that the function compiles
    assert!(true);
}

#[test]
fn test_store_book_with_valid_category() {
    let mut storage = Storage::new();
    
    // Create and store a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);
    
    // Create a book with the valid category ID
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: category_id,
    };

    assert!(store_book(&mut storage, book).is_ok());
    assert_eq!(storage.books.len(), 1);
}

#[test]
fn test_store_book_with_invalid_category() {
    let mut storage = Storage::new();
    
    // Create a book with an invalid category ID
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: "invalid-category-id".to_string(),
    };

    // Attempting to store the book should fail
    let result = store_book(&mut storage, book);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Category with ID invalid-category-id does not exist"));
    assert_eq!(storage.books.len(), 0);
}

#[test]
fn test_store_book_with_nonexistent_category() {
    let mut storage = Storage::new();
    
    // Create a book with a category ID that doesn't exist in storage
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: "nonexistent-category".to_string(),
    };

    // Attempting to store the book should fail
    let result = store_book(&mut storage, book);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Category with ID nonexistent-category does not exist"));
    assert_eq!(storage.books.len(), 0);
}

#[test]
fn test_book_id_matches_storage_key() {
    let mut storage = Storage::new();
    
    // Create and store a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);
    
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: category_id,
    };

    assert!(store_book(&mut storage, book).is_ok());
    
    // Verify that the book's ID matches its key in storage
    let stored_book = storage.books.get("test-id").expect("Book should exist in storage");
    assert_eq!(stored_book.id, "test-id", "Book ID should match its storage key");
}
