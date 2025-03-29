use std::io;
use uuid::Uuid;
use chrono::Utc;
use inquire::{Select, Text};
use crate::storage::{Book, Storage, Category, Author};

pub fn get_book_input(storage: &mut Storage) -> io::Result<Book> {
    let title = Text::new("Enter title:")
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let isbn = Text::new("Enter ISBN:")
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let total_pages = Text::new("Enter total pages:")
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .trim()
        .parse::<i32>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Get list of categories with their IDs
    let categories: Vec<(String, String)> = storage.categories.iter()
        .map(|(id, c)| (c.name.clone(), id.clone()))
        .collect();

    let category_id = if categories.is_empty() {
        // If no categories exist, prompt for a new one
        let category_name = Text::new("Enter new category:")
            .prompt()
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
        let selection = Select::new("Select category:", categories.iter().map(|(name, _)| name).collect())
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Find the selected category's ID
        categories.iter()
            .find(|(name, _)| name == selection)
            .map(|(_, id)| id.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Selected category not found"))?
    };

    // Get list of authors with their IDs
    let authors: Vec<(String, String)> = storage.authors.iter()
        .map(|(id, a)| (a.name.clone(), id.clone()))
        .collect();

    let author_id = if authors.is_empty() {
        // If no authors exist, prompt for a new one
        let author_name = Text::new("Enter new author name:")
            .prompt()
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
        let selection = Select::new("Select author:", authors.iter().map(|(name, _)| name).collect())
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Find the selected author's ID
        authors.iter()
            .find(|(name, _)| name == selection)
            .map(|(_, id)| id.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Selected author not found"))?
    };

    Ok(Book {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        added_on: Utc::now(),
        isbn: isbn.trim().to_string(),
        category_id,
        author_id,
        total_pages,
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
