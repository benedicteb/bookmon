use std::io;
use dialoguer::Select;
use crate::storage::{Storage, Reading, ReadingEvent};

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