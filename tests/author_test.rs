use bookmon::storage::{Storage, Author};
use bookmon::author::{store_author, get_author_by_id};

#[test]
fn test_store_and_retrieve_author() {
    let mut storage = Storage::new();
    let author = Author {
        id: "test-id".to_string(),
        name: "Test Author".to_string(),
    };

    assert!(store_author(&mut storage, author.clone()).is_ok());
    
    let retrieved = get_author_by_id(&storage, "test-id").unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Author");
}

#[test]
fn test_get_nonexistent_author() {
    let mut storage = Storage::new();
    let author = Author {
        id: "test-id".to_string(),
        name: "Test Author".to_string(),
    };

    assert!(store_author(&mut storage, author).is_ok());
    
    let retrieved = get_author_by_id(&storage, "nonexistent-id").unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_author_creation() {
    let author = Author::new("Test Author".to_string());
    assert_eq!(author.name, "Test Author");
    assert!(!author.id.is_empty());
} 