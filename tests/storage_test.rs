use bookmon::storage::{
    Storage, Book, Author, Reading, Category, ReadingEvent,
    ReadingMetadata, sort_json_value
};
use chrono::Utc;
use serde_json::value::Value;

#[test]
fn test_storage_initialization() {
    // Create a new empty storage
    let storage = Storage::new();
    
    // Verify the storage is empty
    assert!(storage.books.is_empty(), "books should be empty");
    assert!(storage.readings.is_empty(), "readings should be empty");
    assert!(storage.authors.is_empty(), "authors should be empty");
    assert!(storage.categories.is_empty(), "categories should be empty");
}

#[test]
fn test_storage_serialization() {
    // Create a storage with test data
    let mut storage = Storage::new();
    
    // Add some test data in a specific order
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);
    
    let category = Category::new("Test Category".to_string(), Some("Test Description".to_string()));
    let category_id = category.id.clone();
    storage.add_category(category);
    
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author_id.clone(),
        100,
    );
    let book_id = book.id.clone();
    storage.add_book(book);
    
    let reading = Reading::new(book_id.clone(), ReadingEvent::Started);
    storage.add_reading(reading);

    // Serialize to JSON
    let json_value = serde_json::to_value(&storage).unwrap();
    let sorted_value = sort_json_value(json_value);
    let first_json = serde_json::to_string_pretty(&sorted_value).unwrap();
    
    // Serialize again to verify deterministic output
    let json_value = serde_json::to_value(&storage).unwrap();
    let sorted_value = sort_json_value(json_value);
    let second_json = serde_json::to_string_pretty(&sorted_value).unwrap();
    
    // Verify that both serializations produce identical output
    assert_eq!(first_json, second_json, "Multiple serializations should produce identical output");

    // Verify that the JSON structure is valid and keys are sorted
    let parsed_value: Value = serde_json::from_str(&first_json).unwrap();
    if let Value::Object(map) = parsed_value {
        let keys: Vec<_> = map.keys().collect();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys, "Top-level keys should be sorted");
    } else {
        panic!("Expected JSON object at root level");
    }

    // Test deserialization
    let deserialized: Storage = serde_json::from_str(&first_json).unwrap();
    assert_eq!(deserialized.books.len(), storage.books.len());
    assert_eq!(deserialized.authors.len(), storage.authors.len());
    assert_eq!(deserialized.categories.len(), storage.categories.len());
    assert_eq!(deserialized.readings.len(), storage.readings.len());
}

#[test]
fn test_storage_load() {
    // Create a new empty storage
    let storage = Storage::new();
    
    // Verify the storage is empty
    assert!(storage.books.is_empty(), "books should be empty");
    assert!(storage.readings.is_empty(), "readings should be empty");
    assert!(storage.authors.is_empty(), "authors should be empty");
    assert!(storage.categories.is_empty(), "categories should be empty");
}

#[test]
fn test_storage_save_and_load() {
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

    // Serialize to JSON
    let json_value = serde_json::to_value(&storage).unwrap();
    let sorted_value = sort_json_value(json_value);
    let json_string = serde_json::to_string_pretty(&sorted_value).unwrap();

    // Deserialize back to a new storage instance
    let loaded_storage: Storage = serde_json::from_str(&json_string).unwrap();

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
        metadata: ReadingMetadata::default(),
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

#[test]
fn test_reading_event_update() {
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

    // Create a reading event with Update type and metadata
    let reading = Reading::with_metadata(book_id.clone(), ReadingEvent::Update, 50);
    storage.add_reading(reading);

    // Verify the reading event was stored correctly
    let readings = storage.get_readings_by_event(ReadingEvent::Update);
    assert_eq!(readings.len(), 1);
    assert_eq!(readings[0].metadata.current_page, Some(50));
}

#[test]
fn test_reading_event_metadata_serialization() {
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

    // Create a reading event with Update type and metadata
    let reading = Reading::with_metadata(book_id.clone(), ReadingEvent::Update, 50);
    
    // Serialize to JSON
    let json = serde_json::to_string(&reading).expect("Failed to serialize reading");
    
    // Deserialize back
    let deserialized: Reading = serde_json::from_str(&json).expect("Failed to deserialize reading");
    
    // Verify the metadata was preserved
    assert_eq!(deserialized.metadata.current_page, Some(50));
}

#[test]
fn test_reading_event_default_metadata() {
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

    // Create a reading event without metadata
    let reading = Reading::new(book_id.clone(), ReadingEvent::Started);
    storage.add_reading(reading);

    // Verify the reading event was stored with default metadata
    let readings = storage.get_readings_by_event(ReadingEvent::Started);
    assert_eq!(readings.len(), 1);
    assert_eq!(readings[0].metadata.current_page, None);
}

#[test]
fn test_is_book_started_with_update_event() {
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

    // Create a reading event sequence: Started -> Update
    let started_reading = Reading::new(book_id.clone(), ReadingEvent::Started);
    let update_reading = Reading::with_metadata(book_id.clone(), ReadingEvent::Update, 50);

    storage.add_reading(started_reading);
    storage.add_reading(update_reading);

    // The book should be considered started even though the most recent event is Update
    assert!(storage.is_book_started(&book_id), "Book should be considered started even with Update as most recent event");
}

#[test]
fn test_sort_json_value() {
    // Test with a nested structure
    let input = serde_json::json!({
        "c": {
            "b": 2,
            "a": 1
        },
        "a": {
            "z": 3,
            "y": 2
        },
        "b": [{
            "b": 2,
            "a": 1
        }]
    });

    let sorted = sort_json_value(input);
    
    // Verify the structure is maintained but keys are sorted
    if let Value::Object(map) = sorted {
        let keys: Vec<_> = map.keys().collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
        
        // Check nested object
        if let Some(Value::Object(nested)) = map.get("c") {
            let nested_keys: Vec<_> = nested.keys().collect();
            assert_eq!(nested_keys, vec!["a", "b"]);
        }
        
        // Check array contents
        if let Some(Value::Array(arr)) = map.get("b") {
            if let Some(Value::Object(arr_obj)) = arr.first() {
                let arr_keys: Vec<_> = arr_obj.keys().collect();
                assert_eq!(arr_keys, vec!["a", "b"]);
            }
        }
    }
}

#[test]
fn test_json_sorting() {
    let mut storage = Storage::new();
    
    // Add some test data
    let author = Author::new("Test Author".to_string());
    let category = Category::new("Test Category".to_string(), None);
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category.id.clone(),
        author.id.clone(),
        100,
    );
    
    storage.add_author(author);
    storage.add_category(category);
    storage.add_book(book);
    
    // Convert to JSON string
    let json_string = storage.to_sorted_json_string().unwrap();
    
    // Parse back to Value to verify sorting
    let value: Value = serde_json::from_str(&json_string).unwrap();
    
    // Helper function to check if keys are sorted
    fn check_keys_sorted(value: &Value) -> bool {
        match value {
            Value::Object(map) => {
                let keys: Vec<_> = map.keys().collect();
                let sorted_keys: Vec<_> = {
                    let mut sorted = keys.clone();
                    sorted.sort();
                    sorted
                };
                keys == sorted_keys && map.values().all(check_keys_sorted)
            }
            Value::Array(arr) => arr.iter().all(check_keys_sorted),
            _ => true,
        }
    }
    
    assert!(check_keys_sorted(&value), "JSON keys are not properly sorted");
} 