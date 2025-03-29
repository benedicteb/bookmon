use std::io;
use dialoguer::Select;
use crate::storage::{Storage, Reading, ReadingEvent};
use chrono::Utc;

pub fn get_reading_input(storage: &Storage) -> io::Result<Reading> {
    // Get list of books with their IDs
    let books: Vec<(String, String)> = storage.books.iter()
        .map(|(id, b)| (b.title.clone(), id.clone()))
        .collect();

    if books.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "No books available. Please add a book first."));
    }

    // Show book selection dialog
    let selection = Select::new()
        .with_prompt("Select book")
        .items(&books.iter().map(|(title, _)| title).collect::<Vec<_>>())
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let book_id = books[selection].1.clone();

    // Show reading event selection dialog
    let events = vec!["Started", "Finished"];
    let event_selection = Select::new()
        .with_prompt("Select reading event")
        .items(&events)
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let event = match event_selection {
        0 => ReadingEvent::Started,
        1 => ReadingEvent::Finished,
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
    // Get all started readings
    let started_readings: Vec<&Reading> = storage.readings.values()
        .filter(|r| matches!(r.event, ReadingEvent::Started))
        .collect();

    if started_readings.is_empty() {
        println!("No books currently being read.");
        return Ok(());
    }

    // Create a table header
    println!("\nCurrently Reading:");
    println!("{:<40} {:<30} {:<15}", "Title", "Author", "Days Started");
    println!("{:-<85}", "");

    // For each started reading, find the corresponding book and author
    for reading in started_readings {
        let book = storage.books.get(&reading.book_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Book not found"))?;
        
        let author = storage.authors.get(&book.author_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Author not found"))?;

        // Calculate days since started
        let days = (Utc::now() - reading.created_on).num_days();
        
        // Print the row
        println!("{:<40} {:<30} {:<15}", 
            book.title,
            author.name,
            days
        );
    }

    println!(); // Add a blank line at the end
    Ok(())
} 