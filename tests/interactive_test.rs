use bookmon::storage::{Author, Book, Category, Reading, ReadingEvent, Storage};

#[test]
fn test_to_display_string_with_missing_author_returns_error() {
    let storage = Storage::new();
    let book = Book::new(
        "Test Book".to_string(),
        "123".to_string(),
        "cat-id".to_string(),
        "nonexistent-author".to_string(),
        100,
    );

    let result = book.to_display_string(&storage, "Started");
    assert!(
        result.is_err(),
        "to_display_string should return Err for missing author"
    );
}

#[test]
fn test_title_from_display_string_with_title_containing_by() {
    // A title containing " by " should be parsed correctly
    let display = "[Started] \"Stand by Me\" by Stephen King";
    let title = Book::title_from_display_string(display);
    assert!(title.is_ok());
    assert_eq!(title.unwrap(), "Stand by Me");
}

#[test]
fn test_interactive_mode_book_selection() {
    let mut storage = Storage::new();

    // Create test data
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author_id.clone(), author);

    let category = Category::new("Test Category".to_string(), None);
    let category_id = category.id.clone();
    storage.categories.insert(category_id.clone(), category);

    // Create three books: unstarted, started, and finished
    let unstarted_book = Book::new(
        "Unstarted Book".to_string(),
        "978-0-000000-00-0".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let unstarted_id = unstarted_book.id.clone();
    storage.books.insert(unstarted_id.clone(), unstarted_book);

    let started_book = Book::new(
        "Started Book".to_string(),
        "978-0-000000-01-0".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let started_id = started_book.id.clone();
    storage.books.insert(started_id.clone(), started_book);

    let finished_book = Book::new(
        "Finished Book".to_string(),
        "978-0-000000-02-0".to_string(),
        category_id.clone(),
        author_id.clone(),
        300,
    );
    let finished_id = finished_book.id.clone();
    storage.books.insert(finished_id.clone(), finished_book);

    // Add reading events
    storage.add_reading(Reading::new(started_id.clone(), ReadingEvent::Started));
    storage.add_reading(Reading::new(finished_id.clone(), ReadingEvent::Started));
    storage.add_reading(Reading::new(finished_id.clone(), ReadingEvent::Finished));

    // Verify that only unstarted and started books are included in the selection
    let options: Vec<String> = storage
        .books
        .iter()
        .filter(|(id, _)| !storage.is_book_finished(id))
        .map(|(_, b)| {
            let status = if storage.is_book_started(&b.id) {
                "Started"
            } else {
                "Not Started"
            };
            b.to_display_string(&storage, status)
        })
        .collect();

    assert_eq!(
        options.len(),
        2,
        "Should only include unstarted and started books"
    );
    assert!(options.iter().any(|opt| opt.contains("Unstarted Book")));
    assert!(options.iter().any(|opt| opt.contains("Started Book")));
    assert!(!options.iter().any(|opt| opt.contains("Finished Book")));
}

#[test]
fn test_book_selection_from_display_string() {
    let mut storage = Storage::new();

    // Create test data
    let author = Author::new("Rebecca Yarros".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author_id.clone(), author);

    let category = Category::new("Test Category".to_string(), None);
    let category_id = category.id.clone();
    storage.categories.insert(category_id.clone(), category);

    // Create a book with a title that matches the failing case
    let book = Book::new(
        "Fourth Wing".to_string(),
        "978-0-000000-00-0".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book_id.clone(), book);

    // Add a reading event to mark it as started
    storage.add_reading(Reading::new(book_id.clone(), ReadingEvent::Started));

    // Create the display string using the new method
    let display = storage
        .books
        .get(&book_id)
        .unwrap()
        .to_display_string(&storage, "Started");

    // Extract the title using the new method
    let title = Book::title_from_display_string(&display);

    // Find the book by title
    let selected_book = storage
        .books
        .values()
        .find(|b| b.title == title)
        .expect("Selected book not found");

    assert_eq!(
        selected_book.id, book_id,
        "Should find the correct book by title"
    );
}

#[test]
fn test_book_selection_with_quoted_titles() {
    let mut storage = Storage::new();

    // Create test data
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.authors.insert(author_id.clone(), author);

    let category = Category::new("Test Category".to_string(), None);
    let category_id = category.id.clone();
    storage.categories.insert(category_id.clone(), category);

    // Create a book with a quoted title
    let book = Book::new(
        "The \"Great\" Gatsby".to_string(),
        "978-0-000000-00-0".to_string(),
        category_id,
        author_id,
        300,
    );
    let book_id = book.id.clone();
    storage.books.insert(book_id.clone(), book);

    // Create the display string using the new method
    let display = storage
        .books
        .get(&book_id)
        .unwrap()
        .to_display_string(&storage, "Not Started");

    // Extract the title using the new method
    let title = Book::title_from_display_string(&display);

    // Find the book by title
    let selected_book = storage
        .books
        .values()
        .find(|b| b.title == title)
        .expect("Selected book not found");

    assert_eq!(
        selected_book.id, book_id,
        "Should find the correct book by title"
    );
}
