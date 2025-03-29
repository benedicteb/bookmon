use bookmon::storage;
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
    let data: serde_json::Value = serde_json::from_str(&contents).expect("Failed to parse JSON");
    
    assert!(data["books"].is_object(), "books should be an object");
    assert!(data["readings"].is_object(), "readings should be an object");
    assert!(data["books"].as_object().unwrap().is_empty(), "books should be empty");
    assert!(data["readings"].as_object().unwrap().is_empty(), "readings should be empty");
}

#[test]
fn test_storage_load() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("storage.json").to_str().unwrap().to_string();

    // Initialize storage
    storage::initialize_storage_file(&storage_path).expect("Failed to initialize storage");
    
    // Test loading
    let data = storage::load_storage(&storage_path).expect("Failed to load storage");
    
    assert!(data["books"].is_object(), "books should be an object");
    assert!(data["readings"].is_object(), "readings should be an object");
    assert!(data["books"].as_object().unwrap().is_empty(), "books should be empty");
    assert!(data["readings"].as_object().unwrap().is_empty(), "readings should be empty");
} 