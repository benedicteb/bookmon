use bookmon::editor::strip_editor_text;
use bookmon::review::{review_template, show_review_detail, show_reviews, store_review};
use bookmon::storage::{Author, Book, Category, ReadingEvent, Storage};

// --- review template abort/round-trip tests ---

#[test]
fn test_untouched_create_review_template_strips_to_none() {
    let template = review_template("1984", "George Orwell", None);
    assert_eq!(strip_editor_text(&template), None);
}

#[test]
fn test_untouched_edit_review_template_strips_back_to_current() {
    let current = "Some review text.";
    let template = review_template("1984", "George Orwell", Some(current));
    assert_eq!(strip_editor_text(&template), Some(current.to_string()));
}

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

#[test]
fn test_no_events_means_no_review() {
    let (storage, book_id) = create_storage_with_book();
    assert!(storage.review_for_book(&book_id).is_none());
}

#[test]
fn test_create_only_yields_one_revision() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First take.".to_string()).unwrap();

    let review = storage.review_for_book(&book_id).unwrap();
    assert_eq!(review.text, "First take.");
    assert_eq!(review.book_id, book_id);
    assert_eq!(review.revisions.len(), 1);
    assert_eq!(review.revisions[0].event, ReadingEvent::CreateReview);
    assert_eq!(review.created_on, review.updated_on);
}

#[test]
fn test_edits_fold_to_the_newest_text() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Second.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Third.".to_string()).unwrap();

    let review = storage.review_for_book(&book_id).unwrap();
    assert_eq!(review.text, "Third.");
    assert_eq!(review.revisions.len(), 3);

    // Oldest first.
    let texts: Vec<&str> = review.revisions.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(texts, vec!["First.", "Second.", "Third."]);

    assert_eq!(review.revisions[0].event, ReadingEvent::CreateReview);
    assert_eq!(review.revisions[1].event, ReadingEvent::EditReview);
    assert_eq!(review.revisions[2].event, ReadingEvent::EditReview);
    assert!(review.updated_on >= review.created_on);
}

#[test]
fn test_only_one_create_review_per_book() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Second.".to_string()).unwrap();

    let creates = storage
        .readings
        .values()
        .filter(|r| r.book_id == book_id && r.event == ReadingEvent::CreateReview)
        .count();
    assert_eq!(creates, 1);
}

#[test]
fn test_unchanged_text_records_no_event() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "Same.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Same.".to_string()).unwrap();

    assert_eq!(
        storage.review_for_book(&book_id).unwrap().revisions.len(),
        1
    );
}

#[test]
fn test_store_review_rejects_unknown_book() {
    let (mut storage, _book_id) = create_storage_with_book();
    let result = store_review(&mut storage, "no-such-book", "Text.".to_string());
    assert!(result.is_err());
}

#[test]
fn test_all_reviews_returns_one_per_reviewed_book() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Edited.".to_string()).unwrap();

    let all = storage.all_reviews();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].text, "Edited.");
    assert_eq!(all[0].revisions.len(), 2);
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
    store_review(&mut storage, &book_id, "A fascinating read.".to_string()).unwrap();
    assert!(show_reviews(&storage).is_ok());
}

#[test]
fn test_show_review_detail_valid() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(
        &mut storage,
        &book_id,
        "Detailed review text here.".to_string(),
    )
    .unwrap();
    assert!(show_review_detail(&storage, &book_id).is_ok());
}

#[test]
fn test_show_review_detail_not_found() {
    let storage = Storage::new();
    let result = show_review_detail(&storage, "nonexistent-id");
    assert!(result.is_err());
}
