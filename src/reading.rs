use std::io;
use inquire::Select;
use crate::storage::{Storage, Reading, ReadingEvent};
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
    let events = vec!["Started", "Finished"];
    let event_selection = Select::new("Select reading event:", events)
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let event = match event_selection {
        "Started" => ReadingEvent::Started,
        "Finished" => ReadingEvent::Finished,
        _ => unreachable!(),
    };

    Ok(Reading::new(book_id, event))
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
        vec!["Title".to_string(), "Author".to_string(), "Days since started".to_string()], // header
    ];

    // For each started book, find the corresponding author and most recent started reading
    for book in started_books {
        let author = storage.authors.get(&book.author_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Author not found"))?;

        // Find the most recent started reading for this book
        let most_recent_reading = storage.readings.values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Started)
            .max_by_key(|r| r.created_on)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;

        // Calculate days since started
        let days = (Utc::now() - most_recent_reading.created_on).num_days();
        
        // Add row to table data
        table_data.push(vec![
            book.title.clone(),
            author.name.clone(),
            days.to_string()
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

    // For each finished book, find the corresponding author and most recent finished reading
    for book in finished_books {
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

    if unstarted_books.is_empty() {
        println!("No unstarted books found.");
        return Ok(());
    }

    // Create table data
    let mut table_data = vec![
        vec!["Title".to_string(), "Author".to_string(), "Category".to_string()], // header
    ];

    // For each unstarted book, find the corresponding author and category
    for book in unstarted_books {
        let author = storage.authors.get(&book.author_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Author not found"))?;
        
        let category = storage.categories.get(&book.category_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Category not found"))?;

        // Add row to table data
        table_data.push(vec![
            book.title.clone(),
            author.name.clone(),
            category.name.clone()
        ]);
    }

    // Print the table
    print_table!(table_data);

    Ok(())
} 
