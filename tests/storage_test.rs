use bookmon::storage::{self, Storage, Book, Author, Reading, Category, ReadingEvent};
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use chrono::Utc;

#[test]
fn test_storage_initialization() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("storage.json").to_str().unwrap().to_string();

    // Test initialization
    storage::initialize_storage_file(&storage_path).expect("Failed to initialize storage");
    
    // Verify file exists
    assert!(Path::new(&storage_path).exists(), "Storage file should be created");
    
    // Read and verify contents
    let contents = fs::read_to_string(&storage_path).expect("Failed to read storage file");
    let storage: Storage = serde_json::from_str(&contents).expect("Failed to parse JSON");
    
    assert!(storage.books.is_empty(), "books should be empty");
    assert!(storage.readings.is_empty(), "readings should be empty");
    assert!(storage.authors.is_empty(), "authors should be empty");
    assert!(storage.categories.is_empty(), "categories should be empty");
}

#[test]
fn test_storage_load() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("storage.json").to_str().unwrap().to_string();

    // Initialize storage
    storage::initialize_storage_file(&storage_path).expect("Failed to initialize storage");
    
    // Test loading
    let storage = storage::load_storage(&storage_path).expect("Failed to load storage");
    
    assert!(storage.books.is_empty(), "books should be empty");
    assert!(storage.readings.is_empty(), "readings should be empty");
    assert!(storage.authors.is_empty(), "authors should be empty");
    assert!(storage.categories.is_empty(), "categories should be empty");
}

#[test]
fn test_storage_save_and_load() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("storage.json").to_str().unwrap().to_string();

    // Create a storage with a book
    let mut storage = Storage::new();
    
    // Create a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);
    
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: category_id.clone(),
        author_id: author_id.clone(),
        total_pages: 300,
    };
    storage.books.insert(book.isbn.clone(), book);

    // Save the storage
    storage::save_storage(&storage_path, &storage).expect("Failed to save storage");

    // Load the storage
    let loaded_storage = storage::load_storage(&storage_path).expect("Failed to load storage");

    // Verify the loaded storage matches the original
    assert_eq!(loaded_storage.books.len(), 1, "Should have one book");
    let loaded_book = loaded_storage.books.get("1234567890").expect("Book should exist");
    assert_eq!(loaded_book.id, "test-id");
    assert_eq!(loaded_book.category_id, category_id);
    assert_eq!(loaded_book.author_id, author_id);
    assert_eq!(loaded_book.total_pages, 300);

    // Verify category and author were properly loaded with created_on
    let loaded_category = loaded_storage.categories.get(&category_id).expect("Category should exist");
    assert!(loaded_category.created_on <= Utc::now());

    let loaded_author = loaded_storage.authors.get(&author_id).expect("Author should exist");
    assert!(loaded_author.created_on <= Utc::now());
}

#[test]
fn test_id_matches_hashmap_keys() {
    let mut storage = Storage::new();

    // Create test data
    // Create a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);
    
    let book = Book {
        id: "book1".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: category_id,
        author_id: "author1".to_string(),
        total_pages: 300,
    };

    let author = Author::new("Test Author".to_string());

    let reading = Reading {
        id: "reading1".to_string(),
        created_on: Utc::now(),
        book_id: "book1".to_string(),
        event: ReadingEvent::Started,
    };

    // Add items to storage
    storage.add_book(book.clone());
    storage.add_author(author.clone());
    storage.add_reading(reading.clone());

    // Verify that each item's id matches its HashMap key
    for (key, book) in &storage.books {
        assert_eq!(key, &book.id, "Book HashMap key does not match book id");
        assert_eq!(book.total_pages, 300, "Book total_pages should be 300");
    }

    for (key, author) in &storage.authors {
        assert_eq!(key, &author.id, "Author HashMap key does not match author id");
        assert!(author.created_on <= Utc::now(), "Author created_on should be set");
    }

    for (key, reading) in &storage.readings {
        assert_eq!(key, &reading.id, "Reading HashMap key does not match reading id");
    }

    for (key, category) in &storage.categories {
        assert_eq!(key, &category.id, "Category HashMap key does not match category id");
        assert!(category.created_on <= Utc::now(), "Category created_on should be set");
    }
}

#[test]
fn test_automatic_uuid_generation() {
    let mut storage = Storage::new();

    // Create items using the new constructors
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        "Fiction".to_string(),
        "Test Author".to_string(),
        300,
    );

    let author = Author::new("Test Author".to_string());

    let reading = Reading::new(
        "book1".to_string(),
        ReadingEvent::Started,
    );

    // Add items to storage
    storage.add_book(book);
    storage.add_author(author);
    storage.add_reading(reading);

    // Verify that each item has a valid UUID
    for (key, book) in &storage.books {
        assert!(!key.is_empty(), "Book ID should not be empty");
        assert!(key.len() > 0, "Book ID should have length");
        assert_eq!(book.total_pages, 300, "Book total_pages should be 300");
    }

    for (key, author) in &storage.authors {
        assert!(!key.is_empty(), "Author ID should not be empty");
        assert!(key.len() > 0, "Author ID should have length");
        assert!(author.created_on <= Utc::now(), "Author created_on should be set");
    }

    for (key, _reading) in &storage.readings {
        assert!(!key.is_empty(), "Reading ID should not be empty");
        assert!(key.len() > 0, "Reading ID should have length");
    }
}

#[test]
fn test_get_readings_by_event() {
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
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create multiple reading events with different types
    let started_reading1 = Reading::new(book_id.clone(), ReadingEvent::Started);
    let started_reading2 = Reading::new(book_id.clone(), ReadingEvent::Started);
    let finished_reading = Reading::new(book_id, ReadingEvent::Finished);

    storage.add_reading(started_reading1);
    storage.add_reading(started_reading2);
    storage.add_reading(finished_reading);

    // Test getting started readings
    let started_readings = storage.get_readings_by_event(ReadingEvent::Started);
    assert_eq!(started_readings.len(), 2, "Should have 2 started readings");
    assert!(started_readings.iter().all(|r| matches!(r.event, ReadingEvent::Started)));

    // Test getting finished readings
    let finished_readings = storage.get_readings_by_event(ReadingEvent::Finished);
    assert_eq!(finished_readings.len(), 1, "Should have 1 finished reading");
    assert!(finished_readings.iter().all(|r| matches!(r.event, ReadingEvent::Finished)));
}

#[test]
fn test_get_readings_by_event_empty() {
    let storage = Storage::new();

    // Test getting readings when storage is empty
    let started_readings = storage.get_readings_by_event(ReadingEvent::Started);
    assert!(started_readings.is_empty(), "Should have no started readings");

    let finished_readings = storage.get_readings_by_event(ReadingEvent::Finished);
    assert!(finished_readings.is_empty(), "Should have no finished readings");
}

#[test]
fn test_get_unstarted_books() {
    let mut storage = Storage::new();

    // Create test data
    let author = Author::new("Test Author".to_string());
    let category = Category::new("Test Category".to_string(), None);
    let book1 = Book::new(
        "Book 1".to_string(),
        "ISBN1".to_string(),
        category.id.clone(),
        author.id.clone(),
        300,
    );
    let book2 = Book::new(
        "Book 2".to_string(),
        "ISBN2".to_string(),
        category.id.clone(),
        author.id.clone(),
        300,
    );

    // Add test data to storage
    storage.add_author(author);
    storage.add_category(category);
    storage.add_book(book1.clone());
    storage.add_book(book2.clone());

    // Initially both books should be unstarted
    let unstarted = storage.get_unstarted_books();
    assert_eq!(unstarted.len(), 2);
    assert!(unstarted.iter().any(|b| b.id == book1.id));
    assert!(unstarted.iter().any(|b| b.id == book2.id));

    // Add a started reading for book1
    let reading = Reading::new(book1.id.clone(), ReadingEvent::Started);
    storage.add_reading(reading);

    // Now only book2 should be unstarted
    let unstarted = storage.get_unstarted_books();
    assert_eq!(unstarted.len(), 1);
    assert_eq!(unstarted[0].id, book2.id);

    // Add a finished reading for book2
    let reading = Reading::new(book2.id.clone(), ReadingEvent::Finished);
    storage.add_reading(reading);

    // Now no books should be unstarted
    let unstarted = storage.get_unstarted_books();
    assert!(unstarted.is_empty());
}

#[test]
fn test_get_started_books() {
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

    // Create multiple books
    let book1 = Book::new(
        "Started Book".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book1_id = book1.id.clone();
    storage.books.insert(book1.id.clone(), book1);

    let book2 = Book::new(
        "Finished Book".to_string(),
        "0987654321".to_string(),
        category_id,
        author_id,
        300,
    );
    let book2_id = book2.id.clone();
    storage.books.insert(book2.id.clone(), book2);

    // Create reading events
    let started_reading = Reading::new(book1_id.clone(), ReadingEvent::Started);
    let finished_reading = Reading::new(book2_id.clone(), ReadingEvent::Finished);

    storage.add_reading(started_reading);
    storage.add_reading(finished_reading);

    // Test getting started books
    let started_books = storage.get_started_books();
    assert_eq!(started_books.len(), 1, "Should have 1 started book");
    assert_eq!(started_books[0].title, "Started Book");
}

#[test]
fn test_get_finished_books() {
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

    // Create multiple books
    let book1 = Book::new(
        "Started Book".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book1_id = book1.id.clone();
    storage.books.insert(book1.id.clone(), book1);

    let book2 = Book::new(
        "Finished Book".to_string(),
        "0987654321".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book2_id = book2.id.clone();
    storage.books.insert(book2.id.clone(), book2);

    // Create reading events with different timestamps
    let started_reading = Reading::new(book1_id.clone(), ReadingEvent::Started);
    let finished_reading = Reading::new(book2_id.clone(), ReadingEvent::Finished);

    storage.add_reading(started_reading);
    storage.add_reading(finished_reading);

    // Test getting finished books
    let finished_books = storage.get_finished_books();
    assert_eq!(finished_books.len(), 1, "Should have 1 finished book");
    assert_eq!(finished_books[0].title, "Finished Book");

    // Test a book that was started then finished
    let book3 = Book::new(
        "Started Then Finished Book".to_string(),
        "1111111111".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book3_id = book3.id.clone();
    storage.books.insert(book3.id.clone(), book3);

    // Add started then finished reading events
    let started_reading2 = Reading::new(book3_id.clone(), ReadingEvent::Started);
    let finished_reading2 = Reading::new(book3_id, ReadingEvent::Finished);

    storage.add_reading(started_reading2);
    storage.add_reading(finished_reading2);

    // Test getting finished books again
    let finished_books = storage.get_finished_books();
    assert_eq!(finished_books.len(), 2, "Should have 2 finished books");
    assert!(finished_books.iter().any(|b| b.title == "Finished Book"));
    assert!(finished_books.iter().any(|b| b.title == "Started Then Finished Book"));

    // Test a book that was finished then started (should not be returned)
    let book4 = Book::new(
        "Finished Then Started Book".to_string(),
        "2222222222".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book4_id = book4.id.clone();
    storage.books.insert(book4.id.clone(), book4);

    // Add finished then started reading events with controlled timestamps
    let mut finished_reading3 = Reading::new(book4_id.clone(), ReadingEvent::Finished);
    finished_reading3.created_on = Utc::now() - chrono::Duration::hours(2); // 2 hours ago

    let mut started_reading3 = Reading::new(book4_id, ReadingEvent::Started);
    started_reading3.created_on = Utc::now() - chrono::Duration::hours(1); // 1 hour ago (more recent)

    storage.add_reading(finished_reading3);
    storage.add_reading(started_reading3);

    // Test getting finished books again - should still only have 2 books
    let finished_books = storage.get_finished_books();
    assert_eq!(finished_books.len(), 2, "Should still have 2 finished books (not including the book that was finished then started)");
    assert!(!finished_books.iter().any(|b| b.title == "Finished Then Started Book"), "Book that was finished then started should not be included");
} 