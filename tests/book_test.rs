use bookmon::book::store_book;
use bookmon::storage::{Author, Book, Category, Reading, ReadingEvent, Storage};
use chrono::{DateTime, Utc};
use serde_json;

#[test]
fn test_get_book_input() {
    // This is a basic test that we can expand later
    // Currently, we're just testing that the function compiles
    assert!(true);
}

#[test]
fn test_store_book_with_valid_category_and_author() {
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

    // Create a book with the valid category and author IDs
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );

    assert!(store_book(&mut storage, book).is_ok());
    assert_eq!(storage.books.len(), 1);
}

#[test]
fn test_store_book_with_invalid_category() {
    let mut storage = Storage::new();

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    // Create a book with an invalid category ID
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        "invalid-category-id".to_string(),
        author_id,
        300,
    );

    // Attempting to store the book should fail
    let result = store_book(&mut storage, book);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Category with ID invalid-category-id does not exist"));
    assert_eq!(storage.books.len(), 0);
}

#[test]
fn test_store_book_with_invalid_author() {
    let mut storage = Storage::new();

    // Create and store a category
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create a book with an invalid author ID
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        "invalid-author-id".to_string(),
        300,
    );

    // Attempting to store the book should fail
    let result = store_book(&mut storage, book);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Author with ID invalid-author-id does not exist"));
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

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );

    assert!(store_book(&mut storage, book.clone()).is_ok());

    // Verify that the book's ID matches its key in storage
    let stored_book = storage
        .books
        .get(&book.id)
        .expect("Book should exist in storage");
    assert_eq!(
        stored_book.id, book.id,
        "Book ID should match its storage key"
    );
}

#[test]
fn test_book_timestamp_format() {
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

    // Create a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );

    // Serialize to JSON
    let json = serde_json::to_string(&book).expect("Failed to serialize book");

    // Parse the JSON to a Value to extract the timestamp string
    let value: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");
    let timestamp_str = value["added_on"]
        .as_str()
        .expect("added_on should be a string");

    // Parse the timestamp string - this will fail if it's not a valid ISO 8601 format
    let parsed_date: DateTime<Utc> = DateTime::parse_from_rfc3339(timestamp_str)
        .expect("Timestamp should be in RFC 3339/ISO 8601 format")
        .into();

    // Verify timezone is UTC
    assert_eq!(parsed_date.timezone(), Utc, "Timestamp should be in UTC");

    // Make sure it can be deserialized back to the original book
    let deserialized: Book = serde_json::from_str(&json).expect("Failed to deserialize book");
    assert_eq!(deserialized.added_on, book.added_on);
}

#[test]
fn test_book_with_bought_status() {
    let mut storage = Storage::new();

    // Create test data
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );

    // Store book and add Bought event
    storage.add_book(book.clone());
    let reading = Reading::new(book.id.clone(), ReadingEvent::Bought);
    storage.add_reading(reading);

    // Verify the book has a Bought event
    let bought_readings = storage.get_readings_by_event(ReadingEvent::Bought);
    assert_eq!(bought_readings.len(), 1, "Should have 1 bought reading");
    assert_eq!(bought_readings[0].book_id, book.id);
}

#[test]
fn test_book_with_want_to_read_status() {
    let mut storage = Storage::new();

    // Create test data
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );

    // Store book and add WantToRead event
    storage.add_book(book.clone());
    let reading = Reading::new(book.id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(reading);

    // Verify the book has a WantToRead event
    let want_to_read_readings = storage.get_readings_by_event(ReadingEvent::WantToRead);
    assert_eq!(
        want_to_read_readings.len(),
        1,
        "Should have 1 want to read reading"
    );
    assert_eq!(want_to_read_readings[0].book_id, book.id);
}

#[test]
fn test_book_with_no_status() {
    let mut storage = Storage::new();

    // Create test data
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );

    // Store book without any event
    storage.add_book(book.clone());

    // Verify the book has no events
    let bought_readings = storage.get_readings_by_event(ReadingEvent::Bought);
    let want_to_read_readings = storage.get_readings_by_event(ReadingEvent::WantToRead);

    assert_eq!(bought_readings.len(), 0, "Should have no bought readings");
    assert_eq!(
        want_to_read_readings.len(),
        0,
        "Should have no want to read readings"
    );
}
