use std::io::{self, Write};
use uuid::Uuid;
use chrono::Utc;
use crate::storage::{Book, Storage};

pub fn get_book_input() -> io::Result<Book> {
    let mut isbn = String::new();
    let mut category = String::new();

    print!("Enter ISBN: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut isbn)?;

    print!("Enter category: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut category)?;

    Ok(Book {
        id: Uuid::new_v4().to_string(),
        added_on: Utc::now(),
        isbn: isbn.trim().to_string(),
        category: category.trim().to_string(),
    })
}

pub fn store_book(storage: &mut Storage, book: Book) -> Result<(), String> {
    if storage.books.contains_key(&book.isbn) {
        return Err("Book with this ISBN already exists".to_string());
    }
    
    storage.books.insert(book.isbn.clone(), book);
    Ok(())
} 