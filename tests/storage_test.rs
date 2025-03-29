use bookmon::storage::{self, Storage, Book, Author, Reading, Category};
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
    
    let book = Book {
        id: "test-id".to_string(),
        title: "Test Book".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category_id: category_id.clone(),
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
    };

    let author = Author {
        id: "author1".to_string(),
        name: "Test Author".to_string(),
    };

    let reading = Reading {
        id: "reading1".to_string(),
        created_on: Utc::now(),
        book_id: "book1".to_string(),
        event: "started".to_string(),
    };

    // Add items to storage
    storage.add_book(book.clone());
    storage.add_author(author.clone());
    storage.add_reading(reading.clone());

    // Verify that each item's id matches its HashMap key
    for (key, book) in &storage.books {
        assert_eq!(key, &book.id, "Book HashMap key does not match book id");
    }

    for (key, author) in &storage.authors {
        assert_eq!(key, &author.id, "Author HashMap key does not match author id");
    }

    for (key, reading) in &storage.readings {
        assert_eq!(key, &reading.id, "Reading HashMap key does not match reading id");
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
    );

    let author = Author::new("Test Author".to_string());

    let reading = Reading::new(
        "book1".to_string(),
        "started".to_string(),
    );

    // Add items to storage
    storage.add_book(book);
    storage.add_author(author);
    storage.add_reading(reading);

    // Verify that each item has a valid UUID
    for (key, _book) in &storage.books {
        assert!(!key.is_empty(), "Book ID should not be empty");
        assert!(key.len() > 0, "Book ID should have length");
    }

    for (key, _author) in &storage.authors {
        assert!(!key.is_empty(), "Author ID should not be empty");
        assert!(key.len() > 0, "Author ID should have length");
    }

    for (key, _reading) in &storage.readings {
        assert!(!key.is_empty(), "Reading ID should not be empty");
        assert!(key.len() > 0, "Reading ID should have length");
    }
} 