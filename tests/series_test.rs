use bookmon::series::{
    delete_series, format_series_display, format_series_label, get_or_create_series,
    is_position_occupied, parse_position_input, rename_series, store_series,
};
use bookmon::storage::{Author, Book, Category, Reading, ReadingEvent, Series, Storage};
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
    book.position_in_series = Some("1".to_string());

    storage.add_book(book.clone());

    let stored_book = storage.get_book(&book.id).unwrap();
    assert_eq!(stored_book.series_id, Some(series_id));
    assert_eq!(stored_book.position_in_series, Some("1".to_string()));
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
fn test_book_series_fields_backward_compatibility_with_integer_position() {
    // Simulate deserializing a book saved with the old i32 position format
    let json_book_with_int_position = r#"{
        "id": "test-id",
        "title": "Old Book",
        "added_on": "2024-01-01T00:00:00Z",
        "isbn": "1234567890",
        "category_id": "cat-id",
        "author_id": "author-id",
        "total_pages": 200,
        "series_id": "series-1",
        "position_in_series": 3
    }"#;

    let book: Book = serde_json::from_str(json_book_with_int_position).unwrap();
    assert_eq!(book.series_id, Some("series-1".to_string()));
    assert_eq!(book.position_in_series, Some("3".to_string()));
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
    book.position_in_series = Some("3".to_string());
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
    assert_eq!(loaded_book.position_in_series, Some("3".to_string()));
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
    book1.position_in_series = Some("1".to_string());
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
    book3.position_in_series = Some("3".to_string());
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
    book2.position_in_series = Some("2".to_string());
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
fn test_get_books_in_series_with_fractional_positions() {
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

    // Book at position 1
    let mut book1 = Book::new(
        "Book One".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        100,
    );
    book1.series_id = Some(series_id.clone());
    book1.position_in_series = Some("1".to_string());
    let book1_id = book1.id.clone();
    storage.add_book(book1);

    // Book at position 2.5 (novella between main entries)
    let mut book_half = Book::new(
        "Book Two-and-a-Half".to_string(),
        "150".to_string(),
        category_id.clone(),
        author_id.clone(),
        80,
    );
    book_half.series_id = Some(series_id.clone());
    book_half.position_in_series = Some("2.5".to_string());
    let book_half_id = book_half.id.clone();
    storage.add_book(book_half);

    // Book at position 2
    let mut book2 = Book::new(
        "Book Two".to_string(),
        "222".to_string(),
        category_id.clone(),
        author_id.clone(),
        200,
    );
    book2.series_id = Some(series_id.clone());
    book2.position_in_series = Some("2".to_string());
    let book2_id = book2.id.clone();
    storage.add_book(book2);

    // Book at position 0 (prequel)
    let mut book0 = Book::new(
        "Prequel".to_string(),
        "000".to_string(),
        category_id,
        author_id,
        50,
    );
    book0.series_id = Some(series_id.clone());
    book0.position_in_series = Some("0".to_string());
    let book0_id = book0.id.clone();
    storage.add_book(book0);

    let books = storage.get_books_in_series(&series_id);
    assert_eq!(books.len(), 4);
    assert_eq!(books[0].id, book0_id); // 0
    assert_eq!(books[1].id, book1_id); // 1
    assert_eq!(books[2].id, book2_id); // 2
    assert_eq!(books[3].id, book_half_id); // 2.5
}

// --- Enriched series display tests ---

#[test]
fn test_format_series_display_with_reading_status() {
    let mut storage = Storage::new();

    let author = Author::new("J.K. Rowling".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fantasy".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let mut series = Series::new("Harry Potter".to_string());
    series.total_books = Some(7);
    let series_id = series.id.clone();
    storage.add_series(series);

    // Book 1: finished
    let mut book1 = Book::new(
        "Philosopher's Stone".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    book1.series_id = Some(series_id.clone());
    book1.position_in_series = Some("1".to_string());
    let book1_id = book1.id.clone();
    storage.add_book(book1);
    storage.add_reading(Reading::new(book1_id.clone(), ReadingEvent::Finished));

    // Book 2: currently reading
    let mut book2 = Book::new(
        "Chamber of Secrets".to_string(),
        "222".to_string(),
        category_id.clone(),
        author_id.clone(),
        350,
    );
    book2.series_id = Some(series_id.clone());
    book2.position_in_series = Some("2".to_string());
    let book2_id = book2.id.clone();
    storage.add_book(book2);
    storage.add_reading(Reading::new(book2_id.clone(), ReadingEvent::Started));

    // Book 3: not started
    let mut book3 = Book::new(
        "Prisoner of Azkaban".to_string(),
        "333".to_string(),
        category_id,
        author_id,
        400,
    );
    book3.series_id = Some(series_id.clone());
    book3.position_in_series = Some("3".to_string());
    storage.add_book(book3);

    let output = format_series_display(&storage, &series_id);

    // Should contain series name with progress
    assert!(
        output.contains("Harry Potter"),
        "should contain series name"
    );
    assert!(output.contains("1/7"), "should show 1 of 7 read");

    // Should contain status indicators
    assert!(
        output.contains("\u{2713}"),
        "should contain checkmark for finished book"
    );
    assert!(
        output.contains("\u{25b8}"),
        "should contain triangle for currently reading"
    );

    // Should contain book titles
    assert!(output.contains("Philosopher's Stone"));
    assert!(output.contains("Chamber of Secrets"));
    assert!(output.contains("Prisoner of Azkaban"));
}

#[test]
fn test_format_series_display_without_total_books() {
    let mut storage = Storage::new();

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series = Series::new("Discworld".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // One finished book
    let mut book1 = Book::new(
        "The Colour of Magic".to_string(),
        "111".to_string(),
        category_id,
        author_id,
        300,
    );
    book1.series_id = Some(series_id.clone());
    book1.position_in_series = Some("1".to_string());
    let book1_id = book1.id.clone();
    storage.add_book(book1);
    storage.add_reading(Reading::new(book1_id, ReadingEvent::Finished));

    let output = format_series_display(&storage, &series_id);

    // Without total_books, should show just count of read
    assert!(output.contains("Discworld"), "should contain series name");
    assert!(output.contains("1 read"), "should show books read count");
    assert!(
        !output.contains("/"),
        "should not show X/Y format without total"
    );
}

#[test]
fn test_format_series_display_empty_series() {
    let mut storage = Storage::new();
    let series = Series::new("Empty Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    let output = format_series_display(&storage, &series_id);
    assert!(output.contains("Empty Series"));
    assert!(output.contains("(no books)"));
}

// --- Duplicate position detection tests ---

#[test]
fn test_is_position_occupied_returns_book_title_when_occupied() {
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

    let mut book = Book::new(
        "First Book".to_string(),
        "111".to_string(),
        category_id,
        author_id,
        100,
    );
    book.series_id = Some(series_id.clone());
    book.position_in_series = Some("1".to_string());
    storage.add_book(book);

    // Position 1 is occupied
    assert_eq!(
        is_position_occupied(&storage, &series_id, "1"),
        Some("First Book".to_string())
    );

    // Position 2 is not occupied
    assert_eq!(is_position_occupied(&storage, &series_id, "2"), None);
}

#[test]
fn test_is_position_occupied_returns_none_for_empty_series() {
    let mut storage = Storage::new();
    let series = Series::new("Empty".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    assert_eq!(is_position_occupied(&storage, &series_id, "1"), None);
}

// --- Series status and total_books tests ---

#[test]
fn test_series_with_status_and_total_books() {
    let mut series = Series::new("Harry Potter".to_string());
    series.status = Some(bookmon::storage::SeriesStatus::Completed);
    series.total_books = Some(7);

    assert_eq!(
        series.status,
        Some(bookmon::storage::SeriesStatus::Completed)
    );
    assert_eq!(series.total_books, Some(7));
}

#[test]
fn test_series_default_status_and_total_books() {
    let series = Series::new("New Series".to_string());
    assert_eq!(series.status, None);
    assert_eq!(series.total_books, None);
}

#[test]
fn test_series_status_backward_compatibility() {
    // Old JSON without status/total_books fields should load fine
    let json = r#"{
        "id": "test-id",
        "name": "Old Series",
        "created_on": "2024-01-01T00:00:00Z"
    }"#;

    let series: Series = serde_json::from_str(json).unwrap();
    assert_eq!(series.name, "Old Series");
    assert_eq!(series.status, None);
    assert_eq!(series.total_books, None);
}

#[test]
fn test_series_status_round_trip() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();

    let mut storage = Storage::new();
    let mut series = Series::new("Stormlight".to_string());
    series.status = Some(bookmon::storage::SeriesStatus::Ongoing);
    series.total_books = Some(10);
    let series_id = series.id.clone();
    storage.add_series(series);

    bookmon::storage::write_storage(&path, &storage).unwrap();
    let loaded = bookmon::storage::load_storage(&path).unwrap();

    let loaded_series = loaded.get_series(&series_id).unwrap();
    assert_eq!(
        loaded_series.status,
        Some(bookmon::storage::SeriesStatus::Ongoing)
    );
    assert_eq!(loaded_series.total_books, Some(10));
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

#[test]
fn test_format_series_label_with_position() {
    let series = Series::new("Harry Potter".to_string());
    assert_eq!(format_series_label(&series, Some("3")), "Harry Potter #3");
}

#[test]
fn test_format_series_label_without_position() {
    let series = Series::new("Harry Potter".to_string());
    assert_eq!(format_series_label(&series, None), "Harry Potter");
}

#[test]
fn test_format_series_label_with_fractional_position() {
    let series = Series::new("Kingkiller Chronicle".to_string());
    assert_eq!(
        format_series_label(&series, Some("2.5")),
        "Kingkiller Chronicle #2.5"
    );
}

#[test]
fn test_format_series_label_with_zero_position() {
    let series = Series::new("Magicians Guild".to_string());
    assert_eq!(
        format_series_label(&series, Some("0")),
        "Magicians Guild #0"
    );
}

#[test]
fn test_series_label_for_book_in_storage() {
    let mut storage = Storage::new();

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series = Series::new("Lord of the Rings".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    let mut book = Book::new(
        "The Fellowship of the Ring".to_string(),
        "123".to_string(),
        category_id,
        author_id,
        423,
    );
    book.series_id = Some(series_id.clone());
    book.position_in_series = Some("1".to_string());
    storage.add_book(book.clone());

    // Get the label for this book using its series data
    let series_ref = storage.get_series(&series_id).unwrap();
    let label = format_series_label(series_ref, book.position_in_series.as_deref());
    assert_eq!(label, "Lord of the Rings #1");
}

#[test]
fn test_series_name_for_book() {
    let mut storage = Storage::new();

    let series = Series::new("My Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    let mut book = Book::new(
        "Test".to_string(),
        "123".to_string(),
        "cat".to_string(),
        "auth".to_string(),
        100,
    );
    book.series_id = Some(series_id);

    assert_eq!(storage.series_name_for_book(&book), "My Series");
}

#[test]
fn test_series_name_for_book_without_series() {
    let storage = Storage::new();

    let book = Book::new(
        "Standalone".to_string(),
        "123".to_string(),
        "cat".to_string(),
        "auth".to_string(),
        100,
    );

    assert_eq!(storage.series_name_for_book(&book), "");
}

#[test]
fn test_parse_position_input_valid_integers() {
    assert_eq!(parse_position_input("1"), Some("1".to_string()));
    assert_eq!(parse_position_input("5"), Some("5".to_string()));
    assert_eq!(parse_position_input("100"), Some("100".to_string()));
}

#[test]
fn test_parse_position_input_with_whitespace() {
    assert_eq!(parse_position_input("  3  "), Some("3".to_string()));
}

#[test]
fn test_parse_position_input_empty() {
    assert_eq!(parse_position_input(""), None);
    assert_eq!(parse_position_input("   "), None);
}

#[test]
fn test_parse_position_input_zero_accepted() {
    assert_eq!(parse_position_input("0"), Some("0".to_string()));
}

#[test]
fn test_parse_position_input_negative_rejected() {
    assert_eq!(parse_position_input("-1"), None);
    assert_eq!(parse_position_input("-99"), None);
}

#[test]
fn test_parse_position_input_fractional() {
    assert_eq!(parse_position_input("2.5"), Some("2.5".to_string()));
    assert_eq!(parse_position_input("0.5"), Some("0.5".to_string()));
}

#[test]
fn test_parse_position_input_non_numeric_rejected() {
    assert_eq!(parse_position_input("abc"), None);
    assert_eq!(parse_position_input("one"), None);
}

// --- Delete series tests ---

#[test]
fn test_delete_series_removes_series_and_unlinks_books() {
    let mut storage = Storage::new();
    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);
    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series = Series::new("Harry Potter".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Add two books in the series
    let mut book1 = Book::new(
        "Philosopher's Stone".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    book1.series_id = Some(series_id.clone());
    book1.position_in_series = Some("1".to_string());
    let book1_id = book1.id.clone();
    storage.add_book(book1);

    let mut book2 = Book::new(
        "Chamber of Secrets".to_string(),
        "222".to_string(),
        category_id,
        author_id,
        350,
    );
    book2.series_id = Some(series_id.clone());
    book2.position_in_series = Some("2".to_string());
    let book2_id = book2.id.clone();
    storage.add_book(book2);

    // Delete the series
    let result = delete_series(&mut storage, &series_id);
    assert!(result.is_ok());

    // Series should be gone
    assert!(storage.series.get(&series_id).is_none());

    // Books should still exist but have no series
    let b1 = storage.books.get(&book1_id).unwrap();
    assert_eq!(b1.series_id, None);
    assert_eq!(b1.position_in_series, None);

    let b2 = storage.books.get(&book2_id).unwrap();
    assert_eq!(b2.series_id, None);
    assert_eq!(b2.position_in_series, None);
}

#[test]
fn test_delete_series_nonexistent_returns_error() {
    let mut storage = Storage::new();
    let result = delete_series(&mut storage, "nonexistent-id");
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("not found"));
    // Error message should not leak internal IDs
    assert!(
        !err_msg.contains("nonexistent-id"),
        "Error message should not expose internal series ID"
    );
}

// --- Rename series tests ---

#[test]
fn test_rename_series() {
    let mut storage = Storage::new();
    let series = Series::new("Harry Poter".to_string()); // typo
    let series_id = series.id.clone();
    storage.add_series(series);

    let result = rename_series(&mut storage, &series_id, "Harry Potter");
    assert!(result.is_ok());

    let renamed = storage.series.get(&series_id).unwrap();
    assert_eq!(renamed.name, "Harry Potter");
}

#[test]
fn test_rename_series_nonexistent_returns_error() {
    let mut storage = Storage::new();
    let result = rename_series(&mut storage, "nonexistent-id", "New Name");
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("not found"));
    assert!(
        !err_msg.contains("nonexistent-id"),
        "Error message should not expose internal series ID"
    );
}

#[test]
fn test_rename_series_to_duplicate_name_returns_error() {
    let mut storage = Storage::new();
    let series1 = Series::new("Harry Potter".to_string());
    let series1_id = series1.id.clone();
    storage.add_series(series1);

    let series2 = Series::new("Lord of the Rings".to_string());
    storage.add_series(series2);

    // Renaming series1 to match series2's name (case-insensitive) should fail
    let result = rename_series(&mut storage, &series1_id, "lord of the rings");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_rename_series_empty_name_returns_error() {
    let mut storage = Storage::new();
    let series = Series::new("Harry Potter".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    let result = rename_series(&mut storage, &series_id, "");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));

    // Whitespace-only should also fail
    let result = rename_series(&mut storage, &series_id, "   ");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));

    // Original name should be unchanged
    assert_eq!(storage.series.get(&series_id).unwrap().name, "Harry Potter");
}

#[test]
fn test_rename_series_same_name_different_case_ok() {
    let mut storage = Storage::new();
    let series = Series::new("harry potter".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Renaming to fix casing of the same series should work
    let result = rename_series(&mut storage, &series_id, "Harry Potter");
    assert!(result.is_ok());
    assert_eq!(storage.series.get(&series_id).unwrap().name, "Harry Potter");
}

// --- Series filter tests ---

#[test]
fn test_filter_books_by_series_case_insensitive_substring() {
    use bookmon::series::filter_books_by_series;

    let mut storage = Storage::new();

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series_hp = Series::new("Harry Potter".to_string());
    let series_hp_id = series_hp.id.clone();
    storage.add_series(series_hp);

    let series_lotr = Series::new("Lord of the Rings".to_string());
    let series_lotr_id = series_lotr.id.clone();
    storage.add_series(series_lotr);

    // Book in Harry Potter
    let mut book1 = Book::new(
        "Philosopher's Stone".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    book1.series_id = Some(series_hp_id.clone());
    storage.add_book(book1);

    // Book in Lord of the Rings
    let mut book2 = Book::new(
        "Fellowship".to_string(),
        "222".to_string(),
        category_id.clone(),
        author_id.clone(),
        400,
    );
    book2.series_id = Some(series_lotr_id.clone());
    storage.add_book(book2);

    // Standalone book
    let book3 = Book::new(
        "Standalone".to_string(),
        "333".to_string(),
        category_id,
        author_id,
        200,
    );
    storage.add_book(book3);

    let all_books: Vec<&Book> = storage.books.values().collect();

    // Filter by "potter" should match Harry Potter (case-insensitive substring)
    let filtered = filter_books_by_series(&storage, &all_books, "potter");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Philosopher's Stone");

    // Filter by "RING" should match Lord of the Rings
    let filtered = filter_books_by_series(&storage, &all_books, "RING");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Fellowship");

    // Filter by "nonexistent" should return empty
    let filtered = filter_books_by_series(&storage, &all_books, "nonexistent");
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_books_by_series_excludes_standalone_books() {
    use bookmon::series::filter_books_by_series;

    let mut storage = Storage::new();

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let series = Series::new("Test Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Book in series
    let mut book_in_series = Book::new(
        "Series Book".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    book_in_series.series_id = Some(series_id);
    storage.add_book(book_in_series);

    // Standalone book (no series)
    let standalone = Book::new(
        "Standalone".to_string(),
        "222".to_string(),
        category_id,
        author_id,
        200,
    );
    storage.add_book(standalone);

    let all_books: Vec<&Book> = storage.books.values().collect();

    // Even a broad filter should not include standalone books
    let filtered = filter_books_by_series(&storage, &all_books, "test");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Series Book");
}

#[test]
fn test_find_matching_series_names() {
    use bookmon::series::find_matching_series_names;

    let mut storage = Storage::new();
    storage.add_series(Series::new("Harry Potter".to_string()));
    storage.add_series(Series::new("Lord of the Rings".to_string()));
    storage.add_series(Series::new("Discworld".to_string()));

    // "potter" matches "Harry Potter"
    let matches = find_matching_series_names(&storage, "potter");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], "Harry Potter");

    // "nonexistent" matches nothing
    let matches = find_matching_series_names(&storage, "nonexistent");
    assert!(matches.is_empty());

    // "or" matches both "Harry Potter" and "Lord of the Rings" and "Discworld"
    let matches = find_matching_series_names(&storage, "or");
    assert!(matches.len() >= 2); // "Harry P-or-tter", "L-or-d of the Rings", "Discw-or-ld"
}

#[test]
fn test_series_filter_empty_message() {
    use bookmon::series::format_series_filter_empty_message;

    let mut storage = Storage::new();
    storage.add_series(Series::new("Harry Potter".to_string()));
    storage.add_series(Series::new("Discworld".to_string()));

    // When the filter matches known series but no books in the result set
    let msg = format_series_filter_empty_message(&storage, "potter");
    assert!(msg.contains("potter"), "should include the filter term");

    // When the filter matches no series at all
    let msg = format_series_filter_empty_message(&storage, "nonexistent");
    assert!(
        msg.contains("nonexistent"),
        "should include the filter term"
    );
    assert!(
        msg.contains("Harry Potter") || msg.contains("Discworld"),
        "should suggest known series"
    );
}
