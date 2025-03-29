use bookmon::storage::{Storage, Reading, ReadingEvent, Book, Author, Category};

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
    let options: Vec<String> = storage.books.iter()
        .filter(|(id, _)| !storage.is_book_finished(id))
        .map(|(_, b)| {
            let author = storage.authors.get(&b.author_id).unwrap();
            let status = if storage.is_book_started(&b.id) {
                "[Started]"
            } else {
                "[Not Started]"
            };
            format!("{} {} by {}", status, b.title, author.name)
        })
        .collect();

    assert_eq!(options.len(), 2, "Should only include unstarted and started books");
    assert!(options.iter().any(|opt| opt.contains("Unstarted Book")));
    assert!(options.iter().any(|opt| opt.contains("Started Book")));
    assert!(!options.iter().any(|opt| opt.contains("Finished Book")));
} 