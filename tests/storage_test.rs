use bookmon::storage::{self, Storage, Book};
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
    let book = Book {
        id: "test-id".to_string(),
        added_on: Utc::now(),
        isbn: "1234567890".to_string(),
        category: "Fiction".to_string(),
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
    assert_eq!(loaded_book.category, "Fiction");
} 