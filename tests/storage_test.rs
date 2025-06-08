use bookmon::storage::{
    sort_json_value, Author, Book, Category, Reading, ReadingEvent, ReadingMetadata, Storage,
};
use chrono::{Duration, TimeZone, Utc};
use serde_json::value::Value;
use uuid::Uuid;

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

    let category = Category::new(
        "Test Category".to_string(),
        Some("Test Description".to_string()),
    );
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
    assert_eq!(
        first_json, second_json,
        "Multiple serializations should produce identical output"
    );

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
    let loaded_book = loaded_storage
        .books
        .get("1234567890")
        .expect("Book should exist");
    assert_eq!(loaded_book.id, "test-id");
    assert_eq!(loaded_book.category_id, category_id);
    assert_eq!(loaded_book.author_id, author_id);
    assert_eq!(loaded_book.total_pages, 300);

    // Verify category and author were properly loaded with created_on
    let loaded_category = loaded_storage
        .categories
        .get(&category_id)
        .expect("Category should exist");
    assert!(loaded_category.created_on <= Utc::now());

    let loaded_author = loaded_storage
        .authors
        .get(&author_id)
        .expect("Author should exist");
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
        assert_eq!(
            key, &author.id,
            "Author HashMap key does not match author id"
        );
        assert!(
            author.created_on <= Utc::now(),
            "Author created_on should be set"
        );
    }

    for (key, reading) in &storage.readings {
        assert_eq!(
            key, &reading.id,
            "Reading HashMap key does not match reading id"
        );
    }

    for (key, category) in &storage.categories {
        assert_eq!(
            key, &category.id,
            "Category HashMap key does not match category id"
        );
        assert!(
            category.created_on <= Utc::now(),
            "Category created_on should be set"
        );
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

    let reading = Reading::new("book1".to_string(), ReadingEvent::Started);

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
        assert!(
            author.created_on <= Utc::now(),
            "Author created_on should be set"
        );
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
    assert!(started_readings
        .iter()
        .all(|r| matches!(r.event, ReadingEvent::Started)));

    // Test getting finished readings
    let finished_readings = storage.get_readings_by_event(ReadingEvent::Finished);
    assert_eq!(finished_readings.len(), 1, "Should have 1 finished reading");
    assert!(finished_readings
        .iter()
        .all(|r| matches!(r.event, ReadingEvent::Finished)));
}

#[test]
fn test_get_readings_by_event_empty() {
    let storage = Storage::new();

    // Test getting readings when storage is empty
    let started_readings = storage.get_readings_by_event(ReadingEvent::Started);
    assert!(
        started_readings.is_empty(),
        "Should have no started readings"
    );

    let finished_readings = storage.get_readings_by_event(ReadingEvent::Finished);
    assert!(
        finished_readings.is_empty(),
        "Should have no finished readings"
    );
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
    assert!(finished_books
        .iter()
        .any(|b| b.title == "Started Then Finished Book"));

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
    assert!(
        !finished_books
            .iter()
            .any(|b| b.title == "Finished Then Started Book"),
        "Book that was finished then started should not be included"
    );
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
    assert!(
        storage.is_book_started(&book_id),
        "Book should be considered started even with Update as most recent event"
    );
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

    assert!(
        check_keys_sorted(&value),
        "JSON keys are not properly sorted"
    );
}

#[test]
fn test_sort_books() {
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

    // Create and store books with different statuses
    let book1 = Book::new(
        "First Book".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author1_id.clone(),
        300,
    );
    let book2 = Book::new(
        "Second Book".to_string(),
        "0987654321".to_string(),
        category_id.clone(),
        author2_id.clone(),
        300,
    );
    let book3 = Book::new(
        "Third Book".to_string(),
        "1111111111".to_string(),
        category_id.clone(),
        author1_id.clone(),
        300,
    );
    let book4 = Book::new(
        "Fourth Book".to_string(),
        "2222222222".to_string(),
        category_id.clone(),
        author2_id.clone(),
        300,
    );

    // Add books to storage
    storage.add_book(book1.clone());
    storage.add_book(book2.clone());
    storage.add_book(book3.clone());
    storage.add_book(book4.clone());

    // Add reading events to set different statuses
    // Book1: Currently reading
    storage.add_reading(Reading::new(book1.id.clone(), ReadingEvent::Started));

    // Book2: Finished
    storage.add_reading(Reading::new(book2.id.clone(), ReadingEvent::Started));
    storage.add_reading(Reading::new(book2.id.clone(), ReadingEvent::Finished));

    // Book3: Not started (no reading events)

    // Book4: Currently reading
    storage.add_reading(Reading::new(book4.id.clone(), ReadingEvent::Started));

    // Sort books
    let sorted_books = storage.sort_books();

    // Verify sorting order
    assert_eq!(sorted_books.len(), 4);

    // First two should be currently reading (sorted by author then title)
    assert!(storage.is_book_started(&sorted_books[0].id));
    assert!(storage.is_book_started(&sorted_books[1].id));
    assert!(!storage.is_book_finished(&sorted_books[0].id));
    assert!(!storage.is_book_finished(&sorted_books[1].id));

    // Third should be not started
    assert!(!storage.is_book_started(&sorted_books[2].id));
    assert!(!storage.is_book_finished(&sorted_books[2].id));

    // Fourth should be finished
    assert!(storage.is_book_finished(&sorted_books[3].id));

    // Verify author and title sorting within each status group
    // Currently reading books should be sorted by author then title
    let author1_name = storage.authors.get(&author1_id).unwrap().name.clone();
    let author2_name = storage.authors.get(&author2_id).unwrap().name.clone();

    if author1_name < author2_name {
        assert_eq!(sorted_books[0].title, "First Book"); // Author1's book
        assert_eq!(sorted_books[1].title, "Fourth Book"); // Author2's book
    } else {
        assert_eq!(sorted_books[0].title, "Fourth Book"); // Author2's book
        assert_eq!(sorted_books[1].title, "First Book"); // Author1's book
    }

    // Not started book should be sorted by author then title
    assert_eq!(sorted_books[2].title, "Third Book");
}

#[test]
fn test_reading_event_bought() {
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

    // Create a Bought reading event
    let bought_reading = Reading::new(book_id.clone(), ReadingEvent::Bought);
    storage.add_reading(bought_reading);

    // Test getting bought readings
    let bought_readings = storage.get_readings_by_event(ReadingEvent::Bought);
    assert_eq!(bought_readings.len(), 1, "Should have 1 bought reading");
    assert!(bought_readings
        .iter()
        .all(|r| matches!(r.event, ReadingEvent::Bought)));
}

#[test]
fn test_reading_event_want_to_read() {
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

    // Create a WantToRead reading event
    let want_to_read_reading = Reading::new(book_id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(want_to_read_reading);

    // Test getting want to read readings
    let want_to_read_readings = storage.get_readings_by_event(ReadingEvent::WantToRead);
    assert_eq!(
        want_to_read_readings.len(),
        1,
        "Should have 1 want to read reading"
    );
    assert!(want_to_read_readings
        .iter()
        .all(|r| matches!(r.event, ReadingEvent::WantToRead)));
}

#[test]
fn test_get_bought_books() {
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

    let book1 = Book::new(
        "Test Book 1".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book2 = Book::new(
        "Test Book 2".to_string(),
        "0987654321".to_string(),
        category_id,
        author_id,
        400,
    );

    // Store books
    storage.add_book(book1.clone());
    storage.add_book(book2.clone());

    // Add Bought event for book1
    let reading1 = Reading::new(book1.id.clone(), ReadingEvent::Bought);
    storage.add_reading(reading1);

    // Add Started event for book2 (should not be in bought list)
    let reading2 = Reading::new(book2.id.clone(), ReadingEvent::Started);
    storage.add_reading(reading2);

    // Get bought books
    let bought_books = storage.get_bought_books();

    // Verify results
    assert_eq!(bought_books.len(), 1);
    assert_eq!(bought_books[0].title, "Test Book 1");
}

#[test]
fn test_get_want_to_read_books() {
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

    let book1 = Book::new(
        "Test Book 1".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book2 = Book::new(
        "Test Book 2".to_string(),
        "0987654321".to_string(),
        category_id,
        author_id,
        400,
    );

    // Store books
    storage.add_book(book1.clone());
    storage.add_book(book2.clone());

    // Add WantToRead event for book1
    let reading1 = Reading::new(book1.id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(reading1);

    // Add Started event for book2 (should not be in want to read list)
    let reading2 = Reading::new(book2.id.clone(), ReadingEvent::Started);
    storage.add_reading(reading2);

    // Get want to read books
    let want_to_read_books = storage.get_want_to_read_books();

    // Verify results
    assert_eq!(want_to_read_books.len(), 1);
    assert_eq!(want_to_read_books[0].title, "Test Book 1");
}

#[test]
fn test_most_recent_event_takes_precedence() {
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

    // Store book
    storage.add_book(book.clone());

    // Add Bought event first
    let reading1 = Reading::new(book.id.clone(), ReadingEvent::Bought);
    storage.add_reading(reading1);

    // Add WantToRead event later (should take precedence)
    let reading2 = Reading::new(book.id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(reading2);

    // Get bought and want to read books
    let bought_books = storage.get_bought_books();
    let want_to_read_books = storage.get_want_to_read_books();

    // Verify results
    assert_eq!(bought_books.len(), 0); // Should not be in bought list
    assert_eq!(want_to_read_books.len(), 1); // Should be in want to read list
    assert_eq!(want_to_read_books[0].title, "Test Book");
}

#[test]
fn test_bought_event_precedence() {
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

    // Store book
    storage.add_book(book.clone());

    // Add Bought event first
    let reading1 = Reading::new(book.id.clone(), ReadingEvent::Bought);
    storage.add_reading(reading1);

    // Add Started event later (should take precedence)
    let reading2 = Reading::new(book.id.clone(), ReadingEvent::Started);
    storage.add_reading(reading2);

    // Get bought books
    let bought_books = storage.get_bought_books();

    // Verify results
    assert_eq!(bought_books.len(), 0); // Should not be in bought list since Started is more recent
}

#[test]
fn test_want_to_read_event_precedence() {
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

    // Store book
    storage.add_book(book.clone());

    // Add WantToRead event first
    let reading1 = Reading::new(book.id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(reading1);

    // Add Started event later (should take precedence)
    let reading2 = Reading::new(book.id.clone(), ReadingEvent::Started);
    storage.add_reading(reading2);

    // Get want to read books
    let want_to_read_books = storage.get_want_to_read_books();

    // Verify results
    assert_eq!(want_to_read_books.len(), 0); // Should not be in want to read list since Started is more recent
}

#[test]
fn test_get_books_by_most_recent_event() {
    let mut storage = Storage::new();

    // Create test data
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new(
        "Test Category".to_string(),
        Some("Test Description".to_string()),
    );
    let category_id = category.id.clone();
    storage.add_category(category);

    // Create three books
    let book1 = Book::new(
        "Book 1".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author_id.clone(),
        100,
    );
    let book1_id = book1.id.clone();
    storage.add_book(book1);

    let book2 = Book::new(
        "Book 2".to_string(),
        "2345678901".to_string(),
        category_id.clone(),
        author_id.clone(),
        200,
    );
    let book2_id = book2.id.clone();
    storage.add_book(book2);

    let book3 = Book::new(
        "Book 3".to_string(),
        "3456789012".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let book3_id = book3.id.clone();
    storage.add_book(book3);

    // Add readings for each book with different events
    // Book 1: Started -> Finished
    storage.add_reading(Reading::new(book1_id.clone(), ReadingEvent::Started));
    storage.add_reading(Reading::new(book1_id.clone(), ReadingEvent::Finished));

    // Book 2: Started -> Update -> Bought
    storage.add_reading(Reading::new(book2_id.clone(), ReadingEvent::Started));
    storage.add_reading(Reading::new(book2_id.clone(), ReadingEvent::Update));
    storage.add_reading(Reading::new(book2_id.clone(), ReadingEvent::Bought));

    // Book 3: WantToRead -> Started -> Update
    storage.add_reading(Reading::new(book3_id.clone(), ReadingEvent::WantToRead));
    storage.add_reading(Reading::new(book3_id.clone(), ReadingEvent::Started));
    storage.add_reading(Reading::new(book3_id.clone(), ReadingEvent::Update));

    // Test getting books with Finished as most recent event
    let finished_books = storage.get_books_by_most_recent_event(ReadingEvent::Finished);
    assert_eq!(
        finished_books.len(),
        1,
        "Should have 1 book with Finished as most recent event"
    );
    assert_eq!(
        finished_books[0].id, book1_id,
        "Book 1 should have Finished as most recent event"
    );

    // Test getting books with Bought as most recent event
    let bought_books = storage.get_books_by_most_recent_event(ReadingEvent::Bought);
    assert_eq!(
        bought_books.len(),
        1,
        "Should have 1 book with Bought as most recent event"
    );
    assert_eq!(
        bought_books[0].id, book2_id,
        "Book 2 should have Bought as most recent event"
    );

    // Test getting books with Update as most recent event
    let update_books = storage.get_books_by_most_recent_event(ReadingEvent::Update);
    assert_eq!(
        update_books.len(),
        1,
        "Should have 1 book with Update as most recent event"
    );
    assert_eq!(
        update_books[0].id, book3_id,
        "Book 3 should have Update as most recent event"
    );

    // Test getting books with Started as most recent event (should be none)
    let started_books = storage.get_books_by_most_recent_event(ReadingEvent::Started);
    assert_eq!(
        started_books.len(),
        0,
        "Should have 0 books with Started as most recent event"
    );

    // Test getting books with WantToRead as most recent event (should be none)
    let want_to_read_books = storage.get_books_by_most_recent_event(ReadingEvent::WantToRead);
    assert_eq!(
        want_to_read_books.len(),
        0,
        "Should have 0 books with WantToRead as most recent event"
    );
}

#[test]
fn test_unmarked_as_want_to_read() {
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

    // Store book
    storage.add_book(book.clone());

    // Add WantToRead event first
    let reading1 = Reading::new(book.id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(reading1);

    // Add UnmarkedAsWantToRead event later
    let reading2 = Reading::new(book.id.clone(), ReadingEvent::UnmarkedAsWantToRead);
    storage.add_reading(reading2);

    // Get want to read books
    let want_to_read_books = storage.get_want_to_read_books();

    // Verify results
    assert_eq!(want_to_read_books.len(), 0); // Should not be in want to read list since UnmarkedAsWantToRead is more recent
}

#[test]
fn test_remarked_as_want_to_read() {
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

    // Store book
    storage.add_book(book.clone());

    // Add WantToRead event first
    let reading1 = Reading::new(book.id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(reading1);

    // Add UnmarkedAsWantToRead event later
    let reading2 = Reading::new(book.id.clone(), ReadingEvent::UnmarkedAsWantToRead);
    storage.add_reading(reading2);

    // Add another WantToRead event after the UnmarkedAsWantToRead event
    let reading3 = Reading::new(book.id.clone(), ReadingEvent::WantToRead);
    storage.add_reading(reading3);

    // Get want to read books
    let want_to_read_books = storage.get_want_to_read_books();

    // Verify results
    assert_eq!(want_to_read_books.len(), 1); // Should be in want to read list since the most recent event is WantToRead
    assert_eq!(want_to_read_books[0].title, "Test Book");
}

#[test]
fn test_get_read_books_by_time_period() {
    let mut storage = Storage::new();

    // Create test data
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Test Category".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    // Create three books
    let book1 = Book::new(
        "Book 1".to_string(),
        "123".to_string(),
        category_id.clone(),
        author_id.clone(),
        100,
    );
    let book2 = Book::new(
        "Book 2".to_string(),
        "456".to_string(),
        category_id.clone(),
        author_id.clone(),
        200,
    );
    let book3 = Book::new(
        "Book 3".to_string(),
        "789".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );

    let book1_id = book1.id.clone();
    let book2_id = book2.id.clone();
    let book3_id = book3.id.clone();

    storage.add_book(book1);
    storage.add_book(book2);
    storage.add_book(book3);

    // Create readings at different times
    let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    // Book 1: Finished before the period
    let reading1 = Reading::new(book1_id.clone(), ReadingEvent::Finished);
    storage.add_reading(reading1);

    // Book 2: Finished during the period
    let reading2 = Reading {
        id: Uuid::new_v4().to_string(),
        created_on: base_time + Duration::days(5),
        book_id: book2_id.clone(),
        event: ReadingEvent::Finished,
        metadata: ReadingMetadata { current_page: None },
    };
    storage.add_reading(reading2);

    // Book 3: Finished after the period
    let reading3 = Reading {
        id: Uuid::new_v4().to_string(),
        created_on: base_time + Duration::days(15),
        book_id: book3_id.clone(),
        event: ReadingEvent::Finished,
        metadata: ReadingMetadata { current_page: None },
    };
    storage.add_reading(reading3);

    // Test period: 2024-01-03 to 2024-01-10
    let from = base_time + Duration::days(3);
    let to = base_time + Duration::days(10);

    let result = storage.get_read_books_by_time_period(from, to);

    // Should only find Book 2
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, book2_id);
}

#[test]
fn test_get_read_books_by_time_period_empty() {
    let storage = Storage::new();
    let from = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let to = Utc.with_ymd_and_hms(2024, 1, 31, 0, 0, 0).unwrap();

    let result = storage.get_read_books_by_time_period(from, to);
    assert!(result.is_empty());
}
