use std::io::{self, Write};
use uuid::Uuid;
use chrono::Utc;
use crate::storage::{Book, Storage};

pub fn get_book_input() -> io::Result<Book> {
    let mut isbn = String::new();
    let mut category = String::new();
    let mut title = String::new();

    print!("Enter title: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut title)?;

    print!("Enter ISBN: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut isbn)?;

    print!("Enter category: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut category)?;

    Ok(Book {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        added_on: Utc::now(),
        isbn: isbn.trim().to_string(),
        category: category.trim().to_string(),
    })
}

pub fn store_book(storage: &mut Storage, book: Book) -> Result<(), String> {
    storage.books.insert(book.id.clone(), book);
    Ok(())
} 
