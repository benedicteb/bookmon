use std::io;
use inquire::{Select, Text};
use crate::storage::{Storage, Reading, ReadingEvent, Book};
use chrono::Utc;
use pretty_table::prelude::*;

pub fn get_reading_input(storage: &Storage) -> io::Result<Reading> {
    // Get list of books with their IDs
    let books: Vec<(String, String)> = storage.books.iter()
        .map(|(id, b)| (b.title.clone(), id.clone()))
        .collect();

    if books.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "No books available. Please add a book first."));
    }

    // Show book selection dialog
    let selection = Select::new("Select book:", books.iter().map(|(title, _)| title).collect())
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    // Find the selected book's ID
    let book_id = books.iter()
        .find(|(title, _)| title == selection)
        .map(|(_, id)| id.clone())
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Selected book not found"))?;

    // Show reading event selection dialog
    let events = vec!["Started", "Finished", "Update"];
    let event_selection = Select::new("Select reading event:", events)
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let event = match event_selection {
        "Started" => ReadingEvent::Started,
        "Finished" => ReadingEvent::Finished,
        "Update" => ReadingEvent::Update,
        _ => unreachable!(),
    };

    // If Update event is selected, get the current page
    if event == ReadingEvent::Update {
        let current_page = Text::new("Enter current page:")
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Invalid page number: {}", e)))?;

        Ok(Reading::with_metadata(book_id, event, current_page))
    } else {
        Ok(Reading::new(book_id, event))
    }
}

pub fn store_reading(storage: &mut Storage, reading: Reading) -> Result<(), String> {
    // Validate that the book exists
    if !storage.books.contains_key(&reading.book_id) {
        return Err(format!("Book with ID {} does not exist", reading.book_id));
    }
    
    storage.add_reading(reading);
    Ok(())
}

pub fn show_started_books(storage: &Storage) -> io::Result<()> {
    // Get all started books using the new method
    let started_books = storage.get_started_books();

    if started_books.is_empty() {
        println!("No books currently being read.");
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec!["Title".to_string(), "Author".to_string(), "Days since started".to_string(), "Progress".to_string()], // header
    ];

    // Sort the started books by author and title
    let mut sorted_books = started_books;
    sorted_books.sort_by(|a, b| {
        let a_author = storage.authors.get(&a.author_id).unwrap();
        let b_author = storage.authors.get(&b.author_id).unwrap();
        
        if a_author.name != b_author.name {
            a_author.name.cmp(&b_author.name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    // For each started book, find the corresponding author and most recent started reading
    for book in sorted_books {
        let author = storage.authors.get(&book.author_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Author not found"))?;

        // Find the most recent started reading for this book
        let most_recent_reading = storage.readings.values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Started)
            .max_by_key(|r| r.created_on)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;

        // Calculate days since started
        let days = (Utc::now() - most_recent_reading.created_on).num_days();

        // Find the most recent update reading for this book
        let most_recent_update = storage.readings.values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Update)
            .max_by_key(|r| r.created_on);

        // Calculate progress percentage if we have both current page and total pages
        let progress = if let Some(update) = most_recent_update {
            if let Some(current_page) = update.metadata.current_page {
                if book.total_pages > 0 {
                    format!("{:.1}%", (current_page as f64 / book.total_pages as f64) * 100.0)
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
            author.name.clone(),
            days.to_string(),
            progress
        ]);
    }

    // Print the table
    print_table!(table_data);

    Ok(())
}

pub fn show_finished_books(storage: &Storage) -> io::Result<()> {
    // Get all finished books using the new method
    let finished_books = storage.get_finished_books();

    if finished_books.is_empty() {
        println!("No finished books found.");
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec!["Title".to_string(), "Author".to_string(), "Finished on".to_string()], // header
    ];

    // Sort the finished books by author and title
    let mut sorted_books = finished_books;
    sorted_books.sort_by(|a, b| {
        let a_author = storage.authors.get(&a.author_id).unwrap();
        let b_author = storage.authors.get(&b.author_id).unwrap();
        
        if a_author.name != b_author.name {
            a_author.name.cmp(&b_author.name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    // For each finished book, find the corresponding author and most recent finished reading
    for book in sorted_books {
        let author = storage.authors.get(&book.author_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Author not found"))?;

        // Find the most recent finished reading for this book
        let most_recent_reading = storage.readings.values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Finished)
            .max_by_key(|r| r.created_on)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;

        // Format the finished date
        let finished_date = most_recent_reading.created_on.format("%Y-%m-%d").to_string();
        
        // Add row to table data
        table_data.push(vec![
            book.title.clone(),
            author.name.clone(),
            finished_date
        ]);
    }

    // Print the table
    print_table!(table_data);

    Ok(())
}

pub fn show_unstarted_books(storage: &Storage) -> io::Result<()> {
    // Get all unstarted books
    let unstarted_books = storage.get_unstarted_books();
    print_book_list_table(storage, unstarted_books, "No unstarted books found.")
}

pub fn show_all_books(storage: &Storage) -> io::Result<()> {
    if storage.books.is_empty() {
        println!("No books found in the library.");
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec!["Title".to_string(), "Author".to_string(), "Category".to_string(), "Status".to_string(), "Progress".to_string()], // header
    ];

    // Use the common sorting method
    let books = storage.sort_books();

    // For each book, find the corresponding author and category
    for book in books {
        let author = storage.authors.get(&book.author_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Author not found"))?;
        
        let category = storage.categories.get(&book.category_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Category not found"))?;

        // Determine book status
        let status = if storage.is_book_finished(&book.id) {
            "Finished"
        } else if storage.is_book_started(&book.id) {
            "In Progress"
        } else {
            "Not Started"
        };

        // Calculate progress if book is in progress
        let progress = if storage.is_book_started(&book.id) && !storage.is_book_finished(&book.id) {
            // Find the most recent update reading for this book
            let most_recent_update = storage.readings.values()
                .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Update)
                .max_by_key(|r| r.created_on);

            if let Some(update) = most_recent_update {
                if let Some(current_page) = update.metadata.current_page {
                    if book.total_pages > 0 {
                        format!("{:.1}%", (current_page as f64 / book.total_pages as f64) * 100.0)
                    } else {
                        "".to_string()
                    }
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
            author.name.clone(),
            category.name.clone(),
            status.to_string(),
            progress
        ]);
    }

    // Print the table
    print_table!(table_data);

    Ok(())
}

/// Prints a table of books with common columns (Title, Author, Category, Added on, Bought)
pub fn print_book_list_table(storage: &Storage, books: Vec<&Book>, empty_message: &str) -> io::Result<()> {
    if books.is_empty() {
        println!("{}", empty_message);
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec!["Title".to_string(), "Author".to_string(), "Category".to_string(), "Added on".to_string(), "Bought".to_string(), "Want to read".to_string()], // header
    ];

    // Sort the books by author and title
    let mut sorted_books = books;
    sorted_books.sort_by(|a, b| {
        let a_author = storage.authors.get(&a.author_id).unwrap();
        let b_author = storage.authors.get(&b.author_id).unwrap();
        
        if a_author.name != b_author.name {
            a_author.name.cmp(&b_author.name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    // For each book, find the corresponding author and category
    for book in sorted_books {
        let author = storage.authors.get(&book.author_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Author not found"))?;
        
        let category = storage.categories.get(&book.category_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Category not found"))?;

        // Check if the book has a bought event
        let has_bought_event = storage.readings.values()
            .any(|r| r.book_id == book.id && r.event == ReadingEvent::Bought);
            
        // Check if the book has a want to read event
        let has_want_to_read_event = storage.readings.values()
            .any(|r| r.book_id == book.id && r.event == ReadingEvent::WantToRead);

        // Format the added date
        let added_date = book.added_on.format("%Y-%m-%d").to_string();

        // Add row to table data
        table_data.push(vec![
            book.title.clone(),
            author.name.clone(),
            category.name.clone(),
            added_date,
            if has_bought_event { "x".to_string() } else { "".to_string() },
            if has_want_to_read_event { "x".to_string() } else { "".to_string() }
        ]);
    }

    // Print the table
    print_table!(table_data);

    Ok(())
} 
