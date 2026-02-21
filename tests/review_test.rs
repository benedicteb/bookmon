use bookmon::review::{show_review_detail, show_reviews, store_review, strip_editor_text};
use bookmon::storage::{Author, Book, Category, Review, Storage};
use chrono::DateTime;

// --- Helper to create a storage with one book ---

fn create_storage_with_book() -> (Storage, String) {
    let mut storage = Storage::new();

    let category = Category::new("Fiction".to_string(), Some("Fictional books".to_string()));
    let category_id = category.id.clone();
    storage.categories.insert(category.id.clone(), category);

    let author = Author::new("George Orwell".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author.id.clone(), author);

    let book = Book::new(
        "1984".to_string(),
        "978-0451524935".to_string(),
        category_id,
        author_id,
        328,
    );
    let book_id = book.id.clone();
    storage.books.insert(book.id.clone(), book);

    (storage, book_id)
}

// --- store_review tests ---

#[test]
fn test_store_review_with_valid_book() {
    let (mut storage, book_id) = create_storage_with_book();

    let review = Review::new(book_id, "A brilliant dystopian novel.".to_string());
    assert!(store_review(&mut storage, review).is_ok());
    assert_eq!(storage.reviews.len(), 1);
}

#[test]
fn test_store_review_with_invalid_book() {
    let mut storage = Storage::new();

    let review = Review::new(
        "nonexistent-book-id".to_string(),
        "This should fail.".to_string(),
    );
    let result = store_review(&mut storage, review);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Book with ID nonexistent-book-id does not exist"));
    assert_eq!(storage.reviews.len(), 0);
}

#[test]
fn test_multiple_reviews_for_same_book() {
    let (mut storage, book_id) = create_storage_with_book();

    let review1 = Review::new(book_id.clone(), "First review.".to_string());
    let review2 = Review::new(
        book_id.clone(),
        "Second review after re-reading.".to_string(),
    );

    assert!(store_review(&mut storage, review1).is_ok());
    assert!(store_review(&mut storage, review2).is_ok());
    assert_eq!(storage.reviews.len(), 2);

    let reviews = storage.get_reviews_for_book(&book_id);
    assert_eq!(reviews.len(), 2);
}

#[test]
fn test_review_id_matches_storage_key() {
    let (mut storage, book_id) = create_storage_with_book();

    let review = Review::new(book_id, "Great book.".to_string());
    let review_id = review.id.clone();
    storage.add_review(review);

    let stored = storage
        .reviews
        .get(&review_id)
        .expect("Review should exist");
    assert_eq!(stored.id, review_id);
}

// --- Review serialization round-trip ---

#[test]
fn test_review_serialization_roundtrip() {
    let review = Review::new(
        "some-book-id".to_string(),
        "A thought-provoking read with\nmultiple lines\nand \"quotes\".".to_string(),
    );

    let json = serde_json::to_string(&review).expect("Failed to serialize review");
    let deserialized: Review = serde_json::from_str(&json).expect("Failed to deserialize review");

    assert_eq!(deserialized.id, review.id);
    assert_eq!(deserialized.book_id, review.book_id);
    assert_eq!(deserialized.text, review.text);
    assert_eq!(deserialized.created_on, review.created_on);
}

#[test]
fn test_review_timestamp_format() {
    let review = Review::new("some-book-id".to_string(), "Test review.".to_string());

    let json = serde_json::to_string(&review).expect("Failed to serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse");
    let timestamp_str = value["created_on"]
        .as_str()
        .expect("created_on should be a string");

    // Must be valid RFC 3339/ISO 8601
    let parsed: DateTime<chrono::Utc> = DateTime::parse_from_rfc3339(timestamp_str)
        .expect("Should be valid RFC 3339")
        .into();
    assert_eq!(parsed.timezone(), chrono::Utc);
}

#[test]
fn test_review_with_special_characters_in_json() {
    let text = "Contains \"quotes\", backslashes \\, newlines\nand tabs\t and unicode: ";
    let review = Review::new("book-id".to_string(), text.to_string());

    let json = serde_json::to_string(&review).expect("Failed to serialize");
    let deserialized: Review = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.text, text);
}

// --- Storage backward compatibility ---

#[test]
fn test_storage_without_reviews_field_loads_with_empty_reviews() {
    // Simulate old JSON without "reviews" key
    let json = r#"{
        "books": {},
        "readings": {},
        "authors": {},
        "categories": {}
    }"#;
    let storage: Storage = serde_json::from_str(json).expect("Should deserialize");
    assert!(storage.reviews.is_empty());
}

#[test]
fn test_storage_with_reviews_field_loads_correctly() {
    let (mut storage, book_id) = create_storage_with_book();
    let review = Review::new(book_id, "Stored review.".to_string());
    storage.add_review(review);

    let json = storage
        .to_sorted_json_string()
        .expect("Failed to serialize");
    let loaded: Storage = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(loaded.reviews.len(), 1);
}

// --- get_reviews_for_book ---

#[test]
fn test_get_reviews_for_book_returns_sorted_newest_first() {
    let (mut storage, book_id) = create_storage_with_book();

    let review1 = Review::new(book_id.clone(), "First.".to_string());
    let id1 = review1.id.clone();
    storage.add_review(review1);

    // Small delay is implicit since Utc::now() gives different timestamps
    let review2 = Review::new(book_id.clone(), "Second.".to_string());
    let id2 = review2.id.clone();
    storage.add_review(review2);

    let reviews = storage.get_reviews_for_book(&book_id);
    assert_eq!(reviews.len(), 2);
    // Newest first — id2 was created after id1
    assert_eq!(reviews[0].id, id2);
    assert_eq!(reviews[1].id, id1);
}

#[test]
fn test_get_reviews_for_book_empty() {
    let (storage, book_id) = create_storage_with_book();
    let reviews = storage.get_reviews_for_book(&book_id);
    assert!(reviews.is_empty());
}

#[test]
fn test_get_reviews_for_book_filters_by_book() {
    let (mut storage, book_id) = create_storage_with_book();

    // Add a second book
    let book2 = Book::new(
        "Animal Farm".to_string(),
        "978-0451526342".to_string(),
        storage.categories.keys().next().unwrap().clone(),
        storage.authors.keys().next().unwrap().clone(),
        112,
    );
    let book2_id = book2.id.clone();
    storage.books.insert(book2.id.clone(), book2);

    // Add reviews for different books
    let review1 = Review::new(book_id.clone(), "Review for 1984.".to_string());
    let review2 = Review::new(book2_id.clone(), "Review for Animal Farm.".to_string());
    storage.add_review(review1);
    storage.add_review(review2);

    let reviews_1984 = storage.get_reviews_for_book(&book_id);
    assert_eq!(reviews_1984.len(), 1);
    assert!(reviews_1984[0].text.contains("1984"));

    let reviews_af = storage.get_reviews_for_book(&book2_id);
    assert_eq!(reviews_af.len(), 1);
    assert!(reviews_af[0].text.contains("Animal Farm"));
}

// --- strip_editor_text tests ---

#[test]
fn test_strip_editor_text_removes_comment_lines() {
    let input = "This is my review.\n# This is a comment.\nSecond line.";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("This is my review.\nSecond line.".to_string()));
}

#[test]
fn test_strip_editor_text_returns_none_for_empty() {
    let input = "# Only comments.\n# Nothing else.\n";
    assert_eq!(strip_editor_text(input), None);
}

#[test]
fn test_strip_editor_text_returns_none_for_whitespace_only() {
    let input = "  \n  \n# comment\n  ";
    assert_eq!(strip_editor_text(input), None);
}

#[test]
fn test_strip_editor_text_trims_surrounding_whitespace() {
    let input = "\n\nMy review.\n\n# comment\n\n";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("My review.".to_string()));
}

#[test]
fn test_strip_editor_text_preserves_internal_whitespace() {
    let input = "First paragraph.\n\nSecond paragraph.\n# comment";
    let result = strip_editor_text(input);
    assert_eq!(
        result,
        Some("First paragraph.\n\nSecond paragraph.".to_string())
    );
}

#[test]
fn test_strip_editor_text_handles_template_format() {
    let input = "A great book about dystopia.\n# Write your review of \"1984\" by George Orwell above.\n# Lines starting with # will be stripped.\n# An empty review (after stripping comments) will abort.\n";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("A great book about dystopia.".to_string()));
}

// --- Display function tests ---

#[test]
fn test_show_reviews_empty() {
    let storage = Storage::new();
    assert!(show_reviews(&storage).is_ok());
}

#[test]
fn test_show_reviews_with_data() {
    let (mut storage, book_id) = create_storage_with_book();
    let review = Review::new(book_id, "A fascinating read.".to_string());
    storage.add_review(review);
    assert!(show_reviews(&storage).is_ok());
}

#[test]
fn test_show_review_detail_valid() {
    let (mut storage, book_id) = create_storage_with_book();
    let review = Review::new(book_id, "Detailed review text here.".to_string());
    let review_id = review.id.clone();
    storage.add_review(review);
    assert!(show_review_detail(&storage, &review_id).is_ok());
}

#[test]
fn test_show_review_detail_not_found() {
    let storage = Storage::new();
    let result = show_review_detail(&storage, "nonexistent-id");
    assert!(result.is_err());
}

// --- Persistence round-trip with reviews ---

#[test]
fn test_write_and_load_storage_with_reviews() {
    use bookmon::storage::{load_storage, write_storage};

    let (mut storage, book_id) = create_storage_with_book();
    let review = Review::new(
        book_id,
        "Multi-line review.\nWith special chars: \"quotes\" & backslashes \\.".to_string(),
    );
    storage.add_review(review);

    let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let path = tmp.path().to_str().expect("Invalid path");

    write_storage(path, &storage).expect("Failed to write");
    let loaded = load_storage(path).expect("Failed to load");

    assert_eq!(loaded.reviews.len(), 1);
    let loaded_review = loaded.reviews.values().next().unwrap();
    assert!(loaded_review.text.contains("Multi-line review."));
    assert!(loaded_review.text.contains("\"quotes\""));
}
