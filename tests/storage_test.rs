use bookmon::storage::{self, Storage};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

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