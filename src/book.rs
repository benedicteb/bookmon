use std::io::{self, Write};
use uuid::Uuid;
use chrono::Utc;
use dialoguer::{Select, Input};
use crate::storage::{Book, Storage, Category, Author};

pub fn get_book_input(storage: &mut Storage) -> io::Result<Book> {
    let mut isbn = String::new();
    let mut title = String::new();

    print!("Enter title: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut title)?;

    print!("Enter ISBN: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut isbn)?;

    // Get list of categories with their IDs
    let categories: Vec<(String, String)> = storage.categories.iter()
        .map(|(id, c)| (c.name.clone(), id.clone()))
        .collect();

    let category_id = if categories.is_empty() {
        // If no categories exist, prompt for a new one
        let category_name: String = Input::new()
            .with_prompt("Enter new category")
            .interact_text()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Create a new category
        let category = Category::new(
            category_name.trim().to_string(),
            None,
        );
        
        // Store the category and get its ID
        crate::category::store_category(storage, category)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Get the ID of the newly created category
        storage.categories.iter()
            .find(|(_, c)| c.name == category_name.trim())
            .map(|(id, _)| id.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get category ID"))?
    } else {
        // Show category selection dialog
        let selection = Select::new()
            .with_prompt("Select category")
            .items(&categories.iter().map(|(name, _)| name).collect::<Vec<_>>())
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        categories[selection].1.clone()
    };

    // Get list of authors with their IDs
    let authors: Vec<(String, String)> = storage.authors.iter()
        .map(|(id, a)| (a.name.clone(), id.clone()))
        .collect();

    let author_id = if authors.is_empty() {
        // If no authors exist, prompt for a new one
        let author_name: String = Input::new()
            .with_prompt("Enter new author name")
            .interact_text()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Create a new author
        let author = Author::new(author_name.trim().to_string());
        
        // Store the author and get its ID
        storage.add_author(author);
        
        // Get the ID of the newly created author
        storage.authors.iter()
            .find(|(_, a)| a.name == author_name.trim())
            .map(|(id, _)| id.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get author ID"))?
    } else {
        // Show author selection dialog
        let selection = Select::new()
            .with_prompt("Select author")
            .items(&authors.iter().map(|(name, _)| name).collect::<Vec<_>>())
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        authors[selection].1.clone()
    };

    Ok(Book {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        added_on: Utc::now(),
        isbn: isbn.trim().to_string(),
        category_id,
        author_id,
    })
}

pub fn store_book(storage: &mut Storage, book: Book) -> Result<(), String> {
    // Validate that the category exists
    if !storage.categories.contains_key(&book.category_id) {
        return Err(format!("Category with ID {} does not exist", book.category_id));
    }
    
    // Validate that the author exists
    if !storage.authors.contains_key(&book.author_id) {
        return Err(format!("Author with ID {} does not exist", book.author_id));
    }
    
    storage.books.insert(book.id.clone(), book);
    Ok(())
} 
