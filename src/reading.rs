use crate::series::format_series_label;
use crate::storage::{Book, Reading, ReadingEvent, Storage};
use crate::table::print_table;
use chrono::Utc;
use std::io;

/// Validates and stores a reading event. Returns an error if the referenced book doesn't exist.
pub fn store_reading(storage: &mut Storage, reading: Reading) -> Result<(), String> {
    // Validate that the book exists
    if !storage.books.contains_key(&reading.book_id) {
        return Err(format!("Book with ID {} does not exist", reading.book_id));
    }

    storage.add_reading(reading);
    Ok(())
}

/// Displays a table of currently-reading books with author, days since started, and progress.
pub fn show_started_books(storage: &Storage) -> io::Result<()> {
    // Get all started books using the new method
    let started_books = storage.get_started_books();

    if started_books.is_empty() {
        println!("No books currently being read.");
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec![
            "Title".to_string(),
            "Author".to_string(),
            "Series".to_string(),
            "Days since started".to_string(),
            "Progress".to_string(),
        ], // header
    ];

    // Sort the started books by author and title
    let mut sorted_books = started_books;
    sorted_books.sort_by(|a, b| {
        let a_author_name = storage.author_name_for_book(a);
        let b_author_name = storage.author_name_for_book(b);

        if a_author_name != b_author_name {
            a_author_name.cmp(b_author_name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    // For each started book, find the corresponding author and most recent started reading
    for book in sorted_books {
        let author_name = storage.author_name_for_book(book);

        // Format series label
        let series_label = book
            .series_id
            .as_ref()
            .and_then(|sid| storage.get_series(sid))
            .map(|s| format_series_label(s, book.position_in_series))
            .unwrap_or_default();

        // Find the most recent started reading for this book
        let most_recent_reading = storage
            .readings
            .values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Started)
            .max_by_key(|r| r.created_on)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;

        // Calculate days since started
        let days = (Utc::now() - most_recent_reading.created_on).num_days();

        // Find the most recent update reading for this book
        let most_recent_update = storage
            .readings
            .values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Update)
            .max_by_key(|r| r.created_on);

        // Calculate progress percentage if we have both current page and total pages
        let progress = if let Some(update) = most_recent_update {
            if let Some(current_page) = update.metadata.current_page {
                if book.total_pages > 0 {
                    format!(
                        "{:.1}%",
                        (current_page as f64 / book.total_pages as f64) * 100.0
                    )
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // Add row to table data
        table_data.push(vec![
            book.title.clone(),
            author_name.to_string(),
            series_label,
            days.to_string(),
            progress,
        ]);
    }

    // Print the table
    print_table(&table_data);

    Ok(())
}

/// Displays a table of finished books with author and finish date.
pub fn show_finished_books(storage: &Storage) -> io::Result<()> {
    // Get all finished books using the new method
    let finished_books = storage.get_finished_books();

    if finished_books.is_empty() {
        println!("No finished books found.");
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec![
            "Title".to_string(),
            "Author".to_string(),
            "Series".to_string(),
            "Finished on".to_string(),
        ], // header
    ];

    // Sort the finished books by author and title
    let mut sorted_books = finished_books;
    sorted_books.sort_by(|a, b| {
        let a_author_name = storage.author_name_for_book(a);
        let b_author_name = storage.author_name_for_book(b);

        if a_author_name != b_author_name {
            a_author_name.cmp(b_author_name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    // For each finished book, find the corresponding author and most recent finished reading
    for book in sorted_books {
        let author_name = storage.author_name_for_book(book);

        // Format series label
        let series_label = book
            .series_id
            .as_ref()
            .and_then(|sid| storage.get_series(sid))
            .map(|s| format_series_label(s, book.position_in_series))
            .unwrap_or_default();

        // Find the most recent finished reading for this book
        let most_recent_reading = storage
            .readings
            .values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Finished)
            .max_by_key(|r| r.created_on)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;

        // Format the finished date
        let finished_date = most_recent_reading
            .created_on
            .format("%Y-%m-%d")
            .to_string();

        // Add row to table data
        table_data.push(vec![
            book.title.clone(),
            author_name.to_string(),
            series_label,
            finished_date,
        ]);
    }

    // Print the table
    print_table(&table_data);

    Ok(())
}

/// Displays a table of books that haven't been started yet.
pub fn show_unstarted_books(storage: &Storage) -> io::Result<()> {
    // Get all unstarted books
    let unstarted_books = storage.get_unstarted_books();
    print_book_list_table(storage, unstarted_books, "No unstarted books found.")
}

/// Prints a table of books with common columns (Title, Author, Category, Added on, Bought)
pub fn print_book_list_table(
    storage: &Storage,
    books: Vec<&Book>,
    empty_message: &str,
) -> io::Result<()> {
    if books.is_empty() {
        println!("{}", empty_message);
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec![
            "Title".to_string(),
            "Author".to_string(),
            "Series".to_string(),
            "Category".to_string(),
            "Added on".to_string(),
            "Bought".to_string(),
            "Want to read".to_string(),
        ], // header
    ];

    // Sort the books by author and title
    let mut sorted_books = books;
    sorted_books.sort_by(|a, b| {
        let a_author_name = storage.author_name_for_book(a);
        let b_author_name = storage.author_name_for_book(b);

        if a_author_name != b_author_name {
            a_author_name.cmp(b_author_name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    // Pre-compute want-to-read book IDs to avoid O(n*m) repeated calls
    let want_to_read_ids: std::collections::HashSet<&str> = storage
        .get_want_to_read_books()
        .iter()
        .map(|b| b.id.as_str())
        .collect();

    // For each book, find the corresponding author and category
    for book in sorted_books {
        let author_name = storage.author_name_for_book(book);

        let category = storage
            .categories
            .get(&book.category_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Category not found"))?;

        // Format series label
        let series_label = book
            .series_id
            .as_ref()
            .and_then(|sid| storage.get_series(sid))
            .map(|s| format_series_label(s, book.position_in_series))
            .unwrap_or_default();

        // Check if the book has a bought event
        let has_bought_event = storage
            .readings
            .values()
            .any(|r| r.book_id == book.id && r.event == ReadingEvent::Bought);

        // Check if the book is marked as want to read (using pre-computed set)
        let is_want_to_read = want_to_read_ids.contains(book.id.as_str());

        // Format the added date
        let added_date = book.added_on.format("%Y-%m-%d").to_string();

        // Add row to table data
        table_data.push(vec![
            book.title.clone(),
            author_name.to_string(),
            series_label,
            category.name.clone(),
            added_date,
            if has_bought_event {
                "x".to_string()
            } else {
                "".to_string()
            },
            if is_want_to_read {
                "x".to_string()
            } else {
                "".to_string()
            },
        ]);
    }

    // Print the table
    print_table(&table_data);

    Ok(())
}
