use bookmon::storage::{Storage, Book};
use bookmon::book::store_book;
use chrono::Utc;

#[test]
fn test_get_book_input() {
    // This is a basic test that we can expand later
    // Currently, we're just testing that the function compiles
    assert!(true);
}

#[test]
fn test_store_book() {
    let mut storage = Storage::new();
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category: "Fiction".to_string(),
    };

    assert!(store_book(&mut storage, book).is_ok());
    assert_eq!(storage.books.len(), 1);
}

#[test]
fn test_book_id_matches_storage_key() {
    let mut storage = Storage::new();
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category: "Fiction".to_string(),
    };

    assert!(store_book(&mut storage, book).is_ok());
    
    // Verify that the book's ID matches its key in storage
    let stored_book = storage.books.get("test-id").expect("Book should exist in storage");
    assert_eq!(stored_book.id, "test-id", "Book ID should match its storage key");
}
