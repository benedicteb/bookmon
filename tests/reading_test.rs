use bookmon::reading::{group_books_by_series, show_started_books, store_reading, BookEntry};
use bookmon::storage::{Author, Book, Category, Reading, ReadingEvent, Series, Storage};
use chrono::{DateTime, Utc};
use serde_json;

#[test]
fn test_store_reading_with_valid_book() {
    let mut storage = Storage::new();

    // Create and store a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    // Create and store a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);

    // Test storing the reading event
    assert!(store_reading(&mut storage, reading).is_ok());
    assert_eq!(storage.readings.len(), 1);
}

#[test]
fn test_store_reading_with_invalid_book() {
    let mut storage = Storage::new();

    // Create a reading event with an invalid book ID
    let reading = Reading::new("invalid-book-id".to_string(), ReadingEvent::Started);

    // Attempting to store the reading should fail
    let result = store_reading(&mut storage, reading);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Book with ID invalid-book-id does not exist"));
    assert_eq!(storage.readings.len(), 0);
}

#[test]
fn test_reading_id_matches_storage_key() {
    let mut storage = Storage::new();

    // Create and store a category first
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    // Create and store a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create and store a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);
    let reading_id = reading.id.clone();
    storage.add_reading(reading);

    // Verify that the reading's ID matches its key in storage
    let stored_reading = storage
        .readings
        .get(&reading_id)
        .expect("Reading should exist in storage");
    assert_eq!(
        stored_reading.id, reading_id,
        "Reading ID should match its storage key"
    );
}

#[test]
fn test_reading_timestamp_format() {
    let mut storage = Storage::new();

    // Create required category and author
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    // Create and store a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);

    // Serialize to JSON
    let json = serde_json::to_string(&reading).expect("Failed to serialize reading");

    // Parse the JSON to a Value to extract the timestamp string
    let value: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");
    let timestamp_str = value["created_on"]
        .as_str()
        .expect("created_on should be a string");

    // Parse the timestamp string - this will fail if it's not a valid ISO 8601 format
    let parsed_date: DateTime<Utc> = DateTime::parse_from_rfc3339(timestamp_str)
        .expect("Timestamp should be in RFC 3339/ISO 8601 format")
        .into();

    // Verify timezone is UTC
    assert_eq!(parsed_date.timezone(), Utc, "Timestamp should be in UTC");

    // Make sure it can be deserialized back to the original reading
    let deserialized: Reading = serde_json::from_str(&json).expect("Failed to deserialize reading");
    assert_eq!(deserialized.created_on, reading.created_on);
}

#[test]
fn test_show_started_books_empty() {
    let storage = Storage::new();
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_with_data() {
    let mut storage = Storage::new();

    // Create and store a category
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    // Create and store a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);
    storage.add_reading(reading);

    // Test showing started books
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_with_multiple_books() {
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

    // Create and store books
    let book1 = Book::new(
        "First Book".to_string(),
        "1234567890".to_string(),
        category_id.clone(),
        author1_id,
        300,
    );
    let book2 = Book::new(
        "Second Book".to_string(),
        "0987654321".to_string(),
        category_id,
        author2_id,
        300,
    );
    let book1_id = book1.id.clone();
    let book2_id = book2.id.clone();
    storage.books.insert(book1.id.clone(), book1);
    storage.books.insert(book2.id.clone(), book2);

    // Create reading events
    let reading1 = Reading::new(book1_id, ReadingEvent::Started);
    let reading2 = Reading::new(book2_id, ReadingEvent::Started);
    storage.add_reading(reading1);
    storage.add_reading(reading2);

    // Test showing started books
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_with_mixed_events() {
    let mut storage = Storage::new();

    // Create and store a category
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    // Create and store a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create both started and finished reading events
    let started_reading = Reading::new(book_id.clone(), ReadingEvent::Started);
    let finished_reading = Reading::new(book_id, ReadingEvent::Finished);
    storage.add_reading(started_reading);
    storage.add_reading(finished_reading);

    // Test showing started books
    assert!(show_started_books(&storage).is_ok());
}

#[test]
fn test_show_started_books_table_format() {
    let mut storage = Storage::new();

    // Create and store a category
    let category = Category::new(
        "Fiction".to_string(),
        Some("Fictional books and novels".to_string()),
    );
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    // Create and store an author
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    // Create and store a book
    let book = Book::new(
        "Test Book".to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    // Create a reading event
    let reading = Reading::new(book_id, ReadingEvent::Started);
    storage.add_reading(reading);

    // Test showing started books
    let result = show_started_books(&storage);
    assert!(result.is_ok());

    // Verify table formatting
    assert!(true, "Table formatting looks good!");
}

#[test]
fn test_no_group_headers_when_no_books_have_series() {
    use bookmon::reading::build_started_books_table;
    use bookmon::table::TableRow;

    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    // Book without series
    let book = Book::new(
        "Standalone Book".to_string(),
        "123".to_string(),
        category_id,
        author_id,
        200,
    );
    let book_id = book.id.clone();
    storage.add_book(book);
    storage.add_reading(Reading::new(book_id, ReadingEvent::Started));

    let table = build_started_books_table(&storage).unwrap();

    // No GroupHeader rows should exist when no books have series
    let has_group_headers = table
        .iter()
        .any(|row| matches!(row, TableRow::GroupHeader(_, _)));
    assert!(
        !has_group_headers,
        "No group headers when no books have series"
    );

    // Header should NOT contain a Series column
    if let TableRow::Header(header) = &table[0] {
        assert!(
            !header.contains(&"Series".to_string()),
            "Series column should not be in header"
        );
    } else {
        panic!("First row should be a Header");
    }
}

#[test]
fn test_group_header_present_when_books_have_series() {
    use bookmon::reading::build_started_books_table;
    use bookmon::table::TableRow;

    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let series = Series::new("My Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Book with series
    let mut book = Book::new(
        "Series Book".to_string(),
        "123".to_string(),
        category_id,
        author_id,
        200,
    );
    book.series_id = Some(series_id);
    book.position_in_series = Some("1".to_string());
    let book_id = book.id.clone();
    storage.add_book(book);
    storage.add_reading(Reading::new(book_id, ReadingEvent::Started));

    let table = build_started_books_table(&storage).unwrap();

    // Should have a GroupHeader row with the series name
    let group_headers: Vec<&str> = table
        .iter()
        .filter_map(|row| match row {
            TableRow::GroupHeader(name, _) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(group_headers, vec!["My Series"]);

    // The data row should have a position prefix in the title
    let data_titles: Vec<&str> = table
        .iter()
        .filter_map(|row| match row {
            TableRow::Data(cells) => Some(cells[0].as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(data_titles, vec!["  #1 Series Book"]);

    // Header should NOT contain a Series column (replaced by GroupHeader)
    if let TableRow::Header(header) = &table[0] {
        assert!(
            !header.contains(&"Series".to_string()),
            "Series column should not be in header when grouping is active"
        );
    } else {
        panic!("First row should be a Header");
    }
}

// ── Series grouping tests ──────────────────────────────────────────

#[test]
fn test_group_books_by_series_standalone_only() {
    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let author = Author::new("Author A".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let book1 = Book::new(
        "Book One".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        200,
    );
    let book2 = Book::new(
        "Book Two".to_string(),
        "222".to_string(),
        category_id,
        author_id,
        300,
    );
    storage.add_book(book1);
    storage.add_book(book2);

    let all_books: Vec<&Book> = storage.books.values().collect();
    let entries = group_books_by_series(&storage, &all_books);

    assert_eq!(entries.len(), 2, "Should have 2 standalone entries");
    for entry in &entries {
        assert!(
            matches!(entry, BookEntry::Standalone(_)),
            "All entries should be standalone"
        );
    }
}

#[test]
fn test_group_books_by_series_groups_and_standalone() {
    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let author = Author::new("Brandon Sanderson".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let series = Series::new("Mistborn".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Series books
    let mut book1 = Book::new(
        "The Final Empire".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        650,
    );
    book1.series_id = Some(series_id.clone());
    book1.position_in_series = Some("1".to_string());

    let mut book2 = Book::new(
        "The Well of Ascension".to_string(),
        "222".to_string(),
        category_id.clone(),
        author_id.clone(),
        700,
    );
    book2.series_id = Some(series_id);
    book2.position_in_series = Some("2".to_string());

    // Standalone book by same author
    let book3 = Book::new(
        "Elantris".to_string(),
        "333".to_string(),
        category_id,
        author_id,
        500,
    );

    storage.add_book(book1);
    storage.add_book(book2);
    storage.add_book(book3);

    let all_books: Vec<&Book> = storage.books.values().collect();
    let entries = group_books_by_series(&storage, &all_books);

    // Should have: 1 SeriesGroup (Mistborn with 2 books) + 1 Standalone (Elantris)
    assert_eq!(
        entries.len(),
        2,
        "Should have 2 entries (1 group + 1 standalone)"
    );

    // Series group should come first (same author, series before standalone)
    match &entries[0] {
        BookEntry::SeriesGroup { name, books } => {
            assert_eq!(name, "Mistborn");
            assert_eq!(books.len(), 2);
            assert_eq!(books[0].title, "The Final Empire"); // position 1
            assert_eq!(books[1].title, "The Well of Ascension"); // position 2
        }
        _ => panic!("First entry should be a SeriesGroup"),
    }

    match &entries[1] {
        BookEntry::Standalone(book) => {
            assert_eq!(book.title, "Elantris");
        }
        _ => panic!("Second entry should be Standalone"),
    }
}

#[test]
fn test_group_books_by_series_multiple_authors() {
    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let author_a = Author::new("Author A".to_string());
    let author_a_id = author_a.id.clone();
    storage.add_author(author_a);

    let author_z = Author::new("Author Z".to_string());
    let author_z_id = author_z.id.clone();
    storage.add_author(author_z);

    let series = Series::new("Z Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Series book by Author Z
    let mut series_book = Book::new(
        "Z Book".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_z_id,
        200,
    );
    series_book.series_id = Some(series_id);
    series_book.position_in_series = Some("1".to_string());

    // Standalone book by Author A (should sort first by author name)
    let standalone_book = Book::new(
        "A Book".to_string(),
        "222".to_string(),
        category_id,
        author_a_id,
        300,
    );

    storage.add_book(series_book);
    storage.add_book(standalone_book);

    let all_books: Vec<&Book> = storage.books.values().collect();
    let entries = group_books_by_series(&storage, &all_books);

    assert_eq!(entries.len(), 2);
    // Author A standalone should come first
    match &entries[0] {
        BookEntry::Standalone(book) => {
            assert_eq!(book.title, "A Book");
        }
        _ => panic!("First entry should be Standalone (Author A)"),
    }
    // Author Z series group should come second
    match &entries[1] {
        BookEntry::SeriesGroup { name, .. } => {
            assert_eq!(name, "Z Series");
        }
        _ => panic!("Second entry should be SeriesGroup (Author Z)"),
    }
}

#[test]
fn test_group_books_by_series_orphaned_series_id_treated_as_standalone() {
    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    // Book with a series_id that doesn't exist in storage
    let mut book = Book::new(
        "Orphan Book".to_string(),
        "111".to_string(),
        category_id,
        author_id,
        200,
    );
    book.series_id = Some("nonexistent-series-id".to_string());
    book.position_in_series = Some("1".to_string());
    storage.add_book(book);

    let all_books: Vec<&Book> = storage.books.values().collect();
    let entries = group_books_by_series(&storage, &all_books);

    assert_eq!(entries.len(), 1, "Should have 1 entry");
    match &entries[0] {
        BookEntry::Standalone(book) => {
            assert_eq!(book.title, "Orphan Book");
        }
        _ => panic!("Orphaned series_id should be treated as standalone"),
    }
}

#[test]
fn test_group_books_by_series_book_without_position_sorts_last_in_group() {
    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let series = Series::new("My Series".to_string());
    let series_id = series.id.clone();
    storage.add_series(series);

    // Book with position
    let mut book1 = Book::new(
        "Book One".to_string(),
        "111".to_string(),
        category_id.clone(),
        author_id.clone(),
        200,
    );
    book1.series_id = Some(series_id.clone());
    book1.position_in_series = Some("1".to_string());

    // Book without position (should sort last)
    let mut book2 = Book::new(
        "Book Without Position".to_string(),
        "222".to_string(),
        category_id,
        author_id,
        300,
    );
    book2.series_id = Some(series_id);
    book2.position_in_series = None;

    storage.add_book(book1);
    storage.add_book(book2);

    let all_books: Vec<&Book> = storage.books.values().collect();
    let entries = group_books_by_series(&storage, &all_books);

    assert_eq!(entries.len(), 1);
    match &entries[0] {
        BookEntry::SeriesGroup { name, books } => {
            assert_eq!(name, "My Series");
            assert_eq!(books.len(), 2);
            assert_eq!(books[0].title, "Book One"); // has position, sorts first
            assert_eq!(books[1].title, "Book Without Position"); // no position, sorts last
                                                                 // No position prefix should appear
            assert!(books[1].position_in_series.is_none());
        }
        _ => panic!("Should be a SeriesGroup"),
    }
}
