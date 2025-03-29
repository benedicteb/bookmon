use std::io::{self, Write};
use uuid::Uuid;
use chrono::Utc;
use dialoguer::{Select, Input};
use crate::storage::{Book, Storage};

pub fn get_book_input(storage: &Storage) -> io::Result<Book> {
    let mut isbn = String::new();
    let mut title = String::new();

    print!("Enter title: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut title)?;

    print!("Enter ISBN: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut isbn)?;

    // Get list of categories
    let categories: Vec<String> = storage.categories.values()
        .map(|c| c.name.clone())
        .collect();

    let category = if categories.is_empty() {
        // If no categories exist, prompt for a new one
        Input::new()
            .with_prompt("Enter new category")
            .interact_text()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    } else {
        // Show category selection dialog
        let selection = Select::new()
            .with_prompt("Select category")
            .items(&categories)
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        categories[selection].clone()
    };

    Ok(Book {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        added_on: Utc::now(),
        isbn: isbn.trim().to_string(),
        category,
    })
}

pub fn store_book(storage: &mut Storage, book: Book) -> Result<(), String> {
    storage.books.insert(book.id.clone(), book);
    Ok(())
} 
