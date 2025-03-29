use bookmon::storage::{Storage, Reading, ReadingEvent, Book, Category, Author};
use bookmon::reading::store_reading;
use chrono::Utc;

#[test]
fn test_store_reading_with_valid_book() {
    let mut storage = Storage::new();
    
    // Create and store a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);
    
    // Create and store a book
    let book = Book {
        id: "test-book-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: category_id,
        author_id: author_id,
    };
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);

    // Test storing the reading event
    assert!(store_reading(&mut storage, reading).is_ok());
    assert_eq!(storage.readings.len(), 1);
}

#[test]
fn test_store_reading_with_invalid_book() {
    let mut storage = Storage::new();
    
    // Create a reading event with an invalid book ID
    let reading = Reading::new("invalid-book-id".to_string(), ReadingEvent::Started);

    // Attempting to store the reading should fail
    let result = store_reading(&mut storage, reading);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Book with ID invalid-book-id does not exist"));
    assert_eq!(storage.readings.len(), 0);
}

#[test]
fn test_reading_id_matches_storage_key() {
    let mut storage = Storage::new();
    
    // Create and store a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);
    
    // Create and store a book
    let book = Book {
        id: "test-book-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: category_id,
        author_id: author_id,
    };
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create and store a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);
    let reading_id = reading.id.clone();
    storage.add_reading(reading);

    // Verify that the reading's ID matches its key in storage
    let stored_reading = storage.readings.get(&reading_id).expect("Reading should exist in storage");
    assert_eq!(stored_reading.id, reading_id, "Reading ID should match its storage key");
} 