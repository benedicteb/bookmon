use bookmon::series::{get_or_create_series, store_series};
use bookmon::storage::{Author, Book, Category, Series, Storage};
use chrono::Utc;

#[test]
fn test_series_creation() {
    let series = Series::new("Harry Potter".to_string());
    assert_eq!(series.name, "Harry Potter");
    assert!(!series.id.is_empty());
    assert!(series.created_on <= Utc::now());
}

#[test]
fn test_series_in_storage() {
    let mut storage = Storage::new();
    assert!(
        storage.series.is_empty(),
        "series should be empty initially"
    );

    let series = Series::new("Lord of the Rings".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    assert_eq!(storage.series.len(), 1);
    let stored = storage.get_series(&series_id).unwrap();
    assert_eq!(stored.name, "Lord of the Rings");
}

#[test]
fn test_book_with_series() {
    let mut storage = Storage::new();

    let author = Author::new("J.K. Rowling".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fantasy".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series = Series::new("Harry Potter".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    let mut book = Book::new(
        "Harry Potter and the Philosopher's Stone".to_string(),
        "9780747532699".to_string(),
        category_id,
        author_id,
        223,
    );
    book.series_id = Some(series_id.clone());
    book.position_in_series = Some(1);

    storage.add_book(book.clone());

    let stored_book = storage.get_book(&book.id).unwrap();
    assert_eq!(stored_book.series_id, Some(series_id));
    assert_eq!(stored_book.position_in_series, Some(1));
}

#[test]
fn test_book_without_series() {
    let book = Book::new(
        "Standalone Book".to_string(),
        "1234567890".to_string(),
        "cat-id".to_string(),
        "author-id".to_string(),
        300,
    );

    assert_eq!(book.series_id, None);
    assert_eq!(book.position_in_series, None);
}

#[test]
fn test_series_backward_compatibility() {
    // Simulate deserializing a JSON file from before the series feature existed
    let json_without_series = r#"{
        "authors": {},
        "books": {},
        "categories": {},
        "readings": {},
        "reviews": {},
        "goals": {}
    }"#;

    let storage: Storage = serde_json::from_str(json_without_series).unwrap();
    assert!(
        storage.series.is_empty(),
        "Series should default to empty HashMap when missing from JSON"
    );
}

#[test]
fn test_book_series_fields_backward_compatibility() {
    // Simulate deserializing a book from before the series feature existed
    let json_book_without_series = r#"{
        "id": "test-id",
        "title": "Old Book",
        "added_on": "2024-01-01T00:00:00Z",
        "isbn": "1234567890",
        "category_id": "cat-id",
        "author_id": "author-id",
        "total_pages": 200
    }"#;

    let book: Book = serde_json::from_str(json_book_without_series).unwrap();
    assert_eq!(book.series_id, None);
    assert_eq!(book.position_in_series, None);
}

#[test]
fn test_series_round_trip() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();

    let mut storage = Storage::new();

    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series = Series::new("Test Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    let mut book = Book::new(
        "Test Book".to_string(),
        "123".to_string(),
        category_id,
        author_id,
        200,
    );
    book.series_id = Some(series_id.clone());
    book.position_in_series = Some(3);
    let book_id = book.id.clone();
    storage.add_book(book);

    // Write to file
    bookmon::storage::write_storage(&path, &storage).unwrap();

    // Load from file
    let loaded = bookmon::storage::load_storage(&path).unwrap();

    // Verify series round-tripped
    assert_eq!(loaded.series.len(), 1);
    let loaded_series = loaded.get_series(&series_id).unwrap();
    assert_eq!(loaded_series.name, "Test Series");

    // Verify book's series fields round-tripped
    let loaded_book = loaded.get_book(&book_id).unwrap();
    assert_eq!(loaded_book.series_id, Some(series_id));
    assert_eq!(loaded_book.position_in_series, Some(3));
}

#[test]
fn test_series_serialization_sorted_json() {
    let mut storage = Storage::new();
    let series = Series::new("Test Series".to_string());
    storage.add_series(series);

    let json_string = storage.to_sorted_json_string().unwrap();
    let value: serde_json::Value = serde_json::from_str(&json_string).unwrap();

    // Verify the series field appears in the JSON
    assert!(
        value.get("series").is_some(),
        "Series should be present in serialized JSON"
    );
}

#[test]
fn test_get_books_in_series() {
    let mut storage = Storage::new();

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series = Series::new("My Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Book 1 in series (position 1)
    let mut book1 = Book::new(
        "Book One".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        100,
    );
    book1.series_id = Some(series_id.clone());
    book1.position_in_series = Some(1);
    let book1_id = book1.id.clone();
    storage.add_book(book1);

    // Book 3 in series (position 3, added before book 2 to test sorting)
    let mut book3 = Book::new(
        "Book Three".to_string(),
        "333".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    book3.series_id = Some(series_id.clone());
    book3.position_in_series = Some(3);
    let book3_id = book3.id.clone();
    storage.add_book(book3);

    // Book 2 in series (position 2)
    let mut book2 = Book::new(
        "Book Two".to_string(),
        "222".to_string(),
        category_id.clone(),
        author_id.clone(),
        200,
    );
    book2.series_id = Some(series_id.clone());
    book2.position_in_series = Some(2);
    let book2_id = book2.id.clone();
    storage.add_book(book2);

    // Book not in any series
    let standalone = Book::new(
        "Standalone".to_string(),
        "444".to_string(),
        category_id,
        author_id,
        150,
    );
    storage.add_book(standalone);

    let books_in_series = storage.get_books_in_series(&series_id);
    assert_eq!(books_in_series.len(), 3);

    // Should be sorted by position
    assert_eq!(books_in_series[0].id, book1_id);
    assert_eq!(books_in_series[1].id, book2_id);
    assert_eq!(books_in_series[2].id, book3_id);
}

#[test]
fn test_store_series() {
    let mut storage = Storage::new();
    let series = Series::new("Discworld".to_string());
    let series_id = series.id.clone();

    store_series(&mut storage, series).unwrap();

    assert_eq!(storage.series.len(), 1);
    let stored = storage.get_series(&series_id).unwrap();
    assert_eq!(stored.name, "Discworld");
}

#[test]
fn test_get_or_create_series_creates_new() {
    let mut storage = Storage::new();
    assert!(storage.series.is_empty());

    let series_id = get_or_create_series(&mut storage, "Harry Potter");

    assert_eq!(storage.series.len(), 1);
    let series = storage.get_series(&series_id).unwrap();
    assert_eq!(series.name, "Harry Potter");
}

#[test]
fn test_get_or_create_series_returns_existing() {
    let mut storage = Storage::new();
    let existing = Series::new("Harry Potter".to_string());
    let existing_id = existing.id.clone();
    storage.add_series(existing);

    let returned_id = get_or_create_series(&mut storage, "Harry Potter");

    assert_eq!(returned_id, existing_id);
    assert_eq!(storage.series.len(), 1, "should not create a duplicate");
}

#[test]
fn test_get_or_create_series_case_insensitive() {
    let mut storage = Storage::new();
    let existing = Series::new("Harry Potter".to_string());
    let existing_id = existing.id.clone();
    storage.add_series(existing);

    // Different case should still find the existing series
    let returned_id = get_or_create_series(&mut storage, "harry potter");

    assert_eq!(returned_id, existing_id);
    assert_eq!(storage.series.len(), 1, "should not create a duplicate");
}

#[test]
fn test_store_book_validates_series_id() {
    use bookmon::book::store_book;

    let mut storage = Storage::new();

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    // Book with invalid series_id should fail
    let mut book = Book::new(
        "Test Book".to_string(),
        "123".to_string(),
        category_id.clone(),
        author_id.clone(),
        200,
    );
    book.series_id = Some("nonexistent-series-id".to_string());

    let result = store_book(&mut storage, book);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Series with ID nonexistent-series-id does not exist"));

    // Book with valid series_id should succeed
    let series = Series::new("Test Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    let mut book2 = Book::new(
        "Test Book 2".to_string(),
        "456".to_string(),
        category_id,
        author_id,
        300,
    );
    book2.series_id = Some(series_id);

    assert!(store_book(&mut storage, book2).is_ok());
}

#[test]
fn test_store_book_without_series_succeeds() {
    use bookmon::book::store_book;

    let mut storage = Storage::new();

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    // Book without series_id (None) should succeed
    let book = Book::new(
        "Standalone Book".to_string(),
        "789".to_string(),
        category_id,
        author_id,
        150,
    );

    assert!(store_book(&mut storage, book).is_ok());
}
