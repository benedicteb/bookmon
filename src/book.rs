use crate::lookup::book_lookup_dto::BookLookupDTO;
use crate::lookup::http_client::HttpClient;
use crate::storage::{Author, Book, Category, ReadingEvent, Storage};
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use std::io;
use std::time::Duration;
use uuid::Uuid;

pub fn get_book_input(storage: &mut Storage) -> io::Result<(Book, Vec<ReadingEvent>)> {
    // First get ISBN
    let isbn = Text::new("Enter ISBN:")
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Create a spinner for the lookup
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner} Looking up book details...")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));

    // Look up book details
    let client = HttpClient::new();
    let book_info = match tokio::runtime::Runtime::new()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .block_on(client.get_book_by_isbn(&isbn))
    {
        Ok(Some(info)) => {
            spinner.finish_and_clear();
            info
        }
        Ok(None) | Err(_) => {
            spinner.finish_and_clear();
            BookLookupDTO {
                title: String::new(),
                authors: vec![],
                description: None,
                isbn: String::new(),
                publish_date: None,
                cover_url: None,
            }
        }
    };

    // Suggest title from lookup or prompt for new one
    let title = if !book_info.title.is_empty() {
        Text::new("Enter title:")
            .with_default(&book_info.title)
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    } else {
        Text::new("Enter title:")
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    };

    let total_pages = Text::new("Enter total pages:")
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .trim()
        .parse::<i32>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Get list of categories with their IDs
    let categories: Vec<(String, String)> = storage
        .categories
        .iter()
        .map(|(id, c)| (c.name.clone(), id.clone()))
        .collect();

    let category_id = if categories.is_empty() {
        // If no categories exist, prompt for a new one
        let category_name = Text::new("Enter new category:")
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Create a new category
        let category = Category::new(category_name.trim().to_string(), None);

        // Store the category and get its ID
        crate::category::store_category(storage, category)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Get the ID of the newly created category
        storage
            .categories
            .iter()
            .find(|(_, c)| c.name == category_name.trim())
            .map(|(id, _)| id.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get category ID"))?
    } else {
        // Show category selection dialog with option to create new
        let mut options = categories
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<&str>>();
        options.push("+ Create new category");

        let selection = Select::new("Select category:", options)
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if selection == "+ Create new category" {
            // Prompt for new category name
            let category_name = Text::new("Enter new category name:")
                .prompt()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            // Create a new category
            let category = Category::new(category_name.trim().to_string(), None);

            // Store the category and get its ID
            crate::category::store_category(storage, category)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            // Get the ID of the newly created category
            storage
                .categories
                .iter()
                .find(|(_, c)| c.name == category_name.trim())
                .map(|(id, _)| id.clone())
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get category ID"))?
        } else {
            // Find the selected category's ID
            categories
                .iter()
                .find(|(name, _)| name.as_str() == selection)
                .map(|(_, id)| id.clone())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Selected category not found")
                })?
        }
    };

    // Get list of authors with their IDs
    let authors: Vec<(String, String)> = storage
        .authors
        .iter()
        .map(|(id, a)| (a.name.clone(), id.clone()))
        .collect();

    let author_id = if authors.is_empty() {
        // If no authors exist, suggest the first author from lookup or prompt for new one
        let suggested_author = book_info
            .authors
            .first()
            .and_then(|a| Some(a.name.clone()))
            .unwrap_or_default();

        let author_name = if !suggested_author.is_empty() {
            Text::new("Enter new author name:")
                .with_default(&suggested_author)
                .prompt()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        } else {
            Text::new("Enter new author name:")
                .prompt()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        };

        // Create a new author
        let author = Author::new(author_name.trim().to_string());

        // Store the author and get its ID
        storage.add_author(author);

        // Get the ID of the newly created author
        storage
            .authors
            .iter()
            .find(|(_, a)| a.name == author_name.trim())
            .map(|(id, _)| id.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get author ID"))?
    } else {
        // Show author selection dialog with option to create new
        let mut options = authors
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<&str>>();
        options.sort(); // Sort alphabetically
        options.push("+ Create new author");

        // Get suggested author from lookup
        let suggested_author = book_info
            .authors
            .first()
            .and_then(|a| Some(a.name.clone()))
            .unwrap_or_default();

        // Track if we added the suggested author to options
        let suggested_author_added =
            if !suggested_author.is_empty() && !options.contains(&suggested_author.as_str()) {
                options.insert(0, &suggested_author);
                true
            } else {
                false
            };

        let selection = Select::new("Select author:", options)
            .prompt()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if selection == "+ Create new author" {
            // Suggest the first author from lookup or prompt for new one
            let suggested_author = book_info
                .authors
                .first()
                .and_then(|a| Some(a.name.clone()))
                .unwrap_or_default();

            let author_name = if !suggested_author.is_empty() {
                Text::new("Enter new author name:")
                    .with_default(&suggested_author)
                    .prompt()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            } else {
                Text::new("Enter new author name:")
                    .prompt()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            };

            // Create a new author
            let author = Author::new(author_name.trim().to_string());

            // Store the author and get its ID
            storage.add_author(author);

            // Get the ID of the newly created author
            storage
                .authors
                .iter()
                .find(|(_, a)| a.name == author_name.trim())
                .map(|(id, _)| id.clone())
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get author ID"))?
        } else if suggested_author_added && selection == suggested_author {
            // User selected the suggested author, add it to storage
            let author = Author::new(suggested_author.trim().to_string());
            storage.add_author(author);

            // Get the ID of the newly created author
            storage
                .authors
                .iter()
                .find(|(_, a)| a.name == suggested_author.trim())
                .map(|(id, _)| id.clone())
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get author ID"))?
        } else {
            // Find the selected author's ID from existing authors
            authors
                .iter()
                .find(|(name, _)| name.as_str() == selection)
                .map(|(_, id)| id.clone())
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Selected author not found"))?
        }
    };

    // Ask about book status
    let options = vec!["Already bought", "Want to read", "Both", "Neither"];
    let selection = Select::new("What is the status of this book?", options)
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let event = match selection {
        "Already bought" => vec![ReadingEvent::Bought],
        "Want to read" => vec![ReadingEvent::WantToRead],
        "Both" => vec![ReadingEvent::Bought, ReadingEvent::WantToRead],
        _ => vec![],
    };

    let book = Book {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        added_on: Utc::now(),
        isbn: isbn.trim().to_string(),
        category_id,
        author_id,
        total_pages,
    };

    Ok((book, event))
}

pub fn store_book(storage: &mut Storage, book: Book) -> Result<(), String> {
    // Validate that the category exists
    if !storage.categories.contains_key(&book.category_id) {
        return Err(format!(
            "Category with ID {} does not exist",
            book.category_id
        ));
    }

    // Validate that the author exists
    if !storage.authors.contains_key(&book.author_id) {
        return Err(format!("Author with ID {} does not exist", book.author_id));
    }

    storage.books.insert(book.id.clone(), book);
    Ok(())
}
