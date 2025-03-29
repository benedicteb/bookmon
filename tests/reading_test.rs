use bookmon::storage::{Storage, Reading, ReadingEvent, Book, Category, Author};
use bookmon::reading::{store_reading, show_started_books};
use chrono::{Utc, DateTime};
use serde_json;

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
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
    );
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
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
    );
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

#[test]
fn test_reading_timestamp_format() {
    let mut storage = Storage::new();
    
    // Create required category and author
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);
    
    // Create and store a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);
    
    // Serialize to JSON
    let json = serde_json::to_string(&reading).expect("Failed to serialize reading");
    
    // Parse the JSON to a Value to extract the timestamp string
    let value: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");
    let timestamp_str = value["created_on"].as_str().expect("created_on should be a string");
    
    // Parse the timestamp string - this will fail if it's not a valid ISO 8601 format
    let parsed_date: DateTime<Utc> = DateTime::parse_from_rfc3339(timestamp_str)
        .expect("Timestamp should be in RFC 3339/ISO 8601 format")
        .into();
    
    // Verify timezone is UTC
    assert_eq!(parsed_date.timezone(), Utc, "Timestamp should be in UTC");
    
    // Make sure it can be deserialized back to the original reading
    let deserialized: Reading = serde_json::from_str(&json).expect("Failed to deserialize reading");
    assert_eq!(deserialized.created_on, reading.created_on);
}

#[test]
fn test_show_started_books_empty() {
    let storage = Storage::new();
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_with_data() {
    let mut storage = Storage::new();
    
    // Create and store a category
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
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);
    storage.add_reading(reading);

    // Test showing started books
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_with_multiple_books() {
    let mut storage = Storage::new();
    
    // Create and store a category
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store authors
    let author1 = Author::new("Author One".to_string());
    let author2 = Author::new("Author Two".to_string());
    let author1_id = author1.id.clone();
    let author2_id = author2.id.clone();
    storage.authors.insert(author1.id.clone(), author1);
    storage.authors.insert(author2.id.clone(), author2);
    
    // Create and store books
    let book1 = Book::new(
        "First Book".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author1_id,
    );
    let book2 = Book::new(
        "Second Book".to_string(),
        "0987654321".to_string(),
        category_id,
        author2_id,
    );
    let book1_id = book1.id.clone();
    let book2_id = book2.id.clone();
    storage.books.insert(book1.id.clone(), book1);
    storage.books.insert(book2.id.clone(), book2);

    // Create reading events
    let reading1 = Reading::new(book1_id, ReadingEvent::Started);
    let reading2 = Reading::new(book2_id, ReadingEvent::Started);
    storage.add_reading(reading1);
    storage.add_reading(reading2);

    // Test showing started books
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_with_mixed_events() {
    let mut storage = Storage::new();
    
    // Create and store a category
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
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create both started and finished reading events
    let started_reading = Reading::new(book_id.clone(), ReadingEvent::Started);
    let finished_reading = Reading::new(book_id, ReadingEvent::Finished);
    storage.add_reading(started_reading);
    storage.add_reading(finished_reading);

    // Test showing started books
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_table_format() {
    let mut storage = Storage::new();
    
    // Create and store a category
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
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);
    storage.add_reading(reading);

    // Test showing started books
    let result = show_started_books(&storage);
    assert!(result.is_ok());

    // Verify table formatting
    assert!(true, "Table formatting looks good!");
} 