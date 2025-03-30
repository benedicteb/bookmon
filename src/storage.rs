use std::fs;
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use inquire::Text;
use serde_json::value::Value;
use std::collections::BTreeMap;
use serde_json::Map;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub added_on: DateTime<Utc>,
    pub isbn: String,
    pub category_id: String,
    pub author_id: String,
    #[serde(default)]
    pub total_pages: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ReadingEvent {
    Finished,
    Started,
    Update,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ReadingMetadata {
    #[serde(default)]
    pub current_page: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reading {
    pub id: String,
    pub created_on: DateTime<Utc>,
    pub book_id: String,
    pub event: ReadingEvent,
    #[serde(default)]
    pub metadata: ReadingMetadata,
}

impl Author {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_on: Utc::now(),
        }
    }
}

impl Book {
    pub fn new(title: String, isbn: String, category_id: String, author_id: String, total_pages: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            added_on: Utc::now(),
            isbn,
            category_id,
            author_id,
            total_pages,
        }
    }

    /// Creates a display string for a book with its status and author name
    pub fn to_display_string(&self, storage: &Storage, status: &str) -> String {
        let author = storage.authors.get(&self.author_id).unwrap();
        format!("[{}] \"{}\" by {}", status, self.title, author.name)
    }

    /// Extracts a book title from a display string
    pub fn title_from_display_string(display: &str) -> String {
        display.split(" by ")
            .next()
            .unwrap()
            .split("] ")
            .nth(1)
            .unwrap()
            .trim_matches('"')
            .to_string()
    }
}

impl Reading {
    pub fn new(book_id: String, event: ReadingEvent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata { current_page: None },
        }
    }

    pub fn with_metadata(book_id: String, event: ReadingEvent, current_page: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata { current_page: Some(current_page) },
        }
    }
}

impl Category {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_on: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    pub books: HashMap<String, Book>,
    pub readings: HashMap<String, Reading>,
    pub authors: HashMap<String, Author>,
    pub categories: HashMap<String, Category>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            books: HashMap::new(),
            readings: HashMap::new(),
            authors: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    pub fn add_book(&mut self, book: Book) -> Option<Book> {
        self.books.insert(book.id.clone(), book)
    }

    pub fn add_reading(&mut self, reading: Reading) -> Option<Reading> {
        self.readings.insert(reading.id.clone(), reading)
    }

    pub fn add_author(&mut self, author: Author) -> Option<Author> {
        self.authors.insert(author.id.clone(), author)
    }

    pub fn add_category(&mut self, category: Category) -> Option<Category> {
        self.categories.insert(category.id.clone(), category)
    }

    pub fn get_book(&self, id: &str) -> Option<&Book> {
        self.books.get(id)
    }

    pub fn get_reading(&self, id: &str) -> Option<&Reading> {
        self.readings.get(id)
    }

    pub fn get_author(&self, id: &str) -> Option<&Author> {
        self.authors.get(id)
    }

    pub fn get_category(&self, id: &str) -> Option<&Category> {
        self.categories.get(id)
    }

    pub fn get_readings_by_event(&self, event_type: ReadingEvent) -> Vec<&Reading> {
        self.readings.values()
            .filter(|r| r.event == event_type)
            .collect()
    }

    pub fn get_unstarted_books(&self) -> Vec<&Book> {
        // Get all book IDs that have either started or finished readings
        let started_or_finished: std::collections::HashSet<String> = self.readings.iter()
            .map(|(_, r)| r.book_id.clone())
            .collect();

        // Find books that have no readings
        self.books.values()
            .filter(|book| !started_or_finished.contains(&book.id))
            .collect()
    }

    pub fn get_started_books(&self) -> Vec<&Book> {
        // Group readings by book_id
        let mut book_readings: HashMap<String, Vec<&Reading>> = HashMap::new();
        for reading in self.readings.values() {
            book_readings.entry(reading.book_id.clone())
                .or_default()
                .push(reading);
        }

        // Filter books to only those that are currently being read
        self.books.values()
            .filter(|book| {
                if let Some(readings) = book_readings.get(&book.id) {
                    // Sort readings by created_on in descending order
                    let mut sorted_readings = readings.clone();
                    sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));

                    // Find the most recent Started event that isn't followed by a Finished event
                    for reading in sorted_readings {
                        match reading.event {
                            ReadingEvent::Started => return true,
                            ReadingEvent::Finished => return false,
                            ReadingEvent::Update => continue,
                        }
                    }
                    false
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_finished_books(&self) -> Vec<&Book> {
        // Group readings by book_id
        let mut book_readings: HashMap<String, Vec<&Reading>> = HashMap::new();
        for reading in self.readings.values() {
            book_readings.entry(reading.book_id.clone())
                .or_default()
                .push(reading);
        }

        // Filter books to only those that have been finished
        self.books.values()
            .filter(|book| {
                if let Some(readings) = book_readings.get(&book.id) {
                    // Sort readings by created_on in descending order
                    let mut sorted_readings = readings.clone();
                    sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));

                    // Check if the most recent reading is Finished
                    if let Some(most_recent) = sorted_readings.first() {
                        most_recent.event == ReadingEvent::Finished
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn is_book_started(&self, book_id: &str) -> bool {
        let readings: Vec<_> = self.readings.values()
            .filter(|r| r.book_id == book_id)
            .collect();
        
        if !readings.is_empty() {
            let mut sorted_readings = readings;
            sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));
            
            // Check if there's a Started event that isn't followed by a Finished event
            for reading in sorted_readings {
                match reading.event {
                    ReadingEvent::Started => return true,
                    ReadingEvent::Finished => return false,
                    ReadingEvent::Update => continue,
                }
            }
            false
        } else {
            false
        }
    }

    pub fn is_book_finished(&self, book_id: &str) -> bool {
        let readings: Vec<_> = self.readings.values()
            .filter(|r| r.book_id == book_id)
            .collect();
        
        if !readings.is_empty() {
            let mut sorted_readings = readings;
            sorted_readings.sort_by(|a, b| b.created_on.cmp(&a.created_on));
            if let Some(most_recent) = sorted_readings.first() {
                most_recent.event == ReadingEvent::Finished
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub fn sort_json_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted_map = BTreeMap::new();
            for (k, v) in map {
                sorted_map.insert(k, sort_json_value(v));
            }
            Value::Object(Map::from_iter(sorted_map))
        }
        Value::Array(vec) => {
            Value::Array(vec.into_iter().map(sort_json_value).collect())
        }
        _ => value,
    }
}

pub fn initialize_storage_file(storage_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(storage_path);
    
    if !path.exists() {
        let initial_storage = Storage::new();
        
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Convert to JSON value, sort keys, then convert back to string
        let json_value = serde_json::to_value(&initial_storage)?;
        let sorted_value = sort_json_value(json_value);
        let json_string = serde_json::to_string_pretty(&sorted_value)?;
        
        // Write the initial data
        fs::write(path, json_string)?;
    }
    
    Ok(())
}

pub fn handle_missing_fields(storage: &mut Storage, storage_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // First, collect all missing references
    let mut missing_authors: Vec<(String, String)> = Vec::new(); // (book_title, author_id)
    let mut missing_categories: Vec<(String, String)> = Vec::new(); // (book_title, category_id)
    let mut missing_books: Vec<(String, String)> = Vec::new(); // (reading_id, book_id)
    let mut books_missing_fields: Vec<String> = Vec::new(); // book_ids

    // Check books for missing fields and references
    for (book_id, book) in storage.books.iter() {
        if !storage.authors.contains_key(&book.author_id) {
            missing_authors.push((book.title.clone(), book.author_id.clone()));
        }
        if !storage.categories.contains_key(&book.category_id) {
            missing_categories.push((book.title.clone(), book.category_id.clone()));
        }
        if book.total_pages <= 0 {
            books_missing_fields.push(book_id.clone());
        }
    }

    // Check readings for missing book references
    for (reading_id, reading) in storage.readings.iter() {
        if !storage.books.contains_key(&reading.book_id) {
            missing_books.push((reading_id.clone(), reading.book_id.clone()));
        }
    }

    // Handle missing authors
    for (book_title, _author_id) in missing_authors {
        println!("Book '{}' references a missing author. Please provide the author name:", book_title);
        let author_name = Text::new("Enter author name:")
            .prompt()
            .map_err(|e| format!("Failed to get author input: {}", e))?;
        
        let author = Author::new(author_name.trim().to_string());
        storage.add_author(author);
        
        // Save after each author is added
        save_storage(storage_path, storage)?;
    }

    // Handle missing categories
    for (book_title, _category_id) in missing_categories {
        println!("Book '{}' references a missing category. Please provide the category name:", book_title);
        let category_name = Text::new("Enter category name:")
            .prompt()
            .map_err(|e| format!("Failed to get category input: {}", e))?;
        
        let category = Category::new(
            category_name.trim().to_string(),
            None,
        );
        storage.add_category(category);
        
        // Save after each category is added
        save_storage(storage_path, storage)?;
    }

    // Handle books with missing fields
    for book_id in books_missing_fields {
        let book = storage.books.get(&book_id).unwrap();
        println!("Book '{}' is missing total pages. Please provide the total pages:", book.title);
        let total_pages = Text::new("Enter total pages:")
            .prompt()
            .map_err(|e| format!("Failed to get total pages: {}", e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("Invalid total pages: {}", e))?;

        if let Some(book) = storage.books.get_mut(&book_id) {
            book.total_pages = total_pages;
        }
        
        // Save after each book's total_pages is updated
        save_storage(storage_path, storage)?;
    }

    // Handle missing books
    for (reading_id, _book_id) in missing_books {
        println!("Reading event {} references a missing book. Please provide the book details:", reading_id);
        
        let title = Text::new("Enter book title:")
            .prompt()
            .map_err(|e| format!("Failed to get book title: {}", e))?;
        
        let isbn = Text::new("Enter book ISBN:")
            .prompt()
            .map_err(|e| format!("Failed to get book ISBN: {}", e))?;
        
        let total_pages = Text::new("Enter total pages:")
            .prompt()
            .map_err(|e| format!("Failed to get total pages: {}", e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("Invalid total pages: {}", e))?;

        // Get or create author
        let author_name = Text::new("Enter author name:")
            .prompt()
            .map_err(|e| format!("Failed to get author name: {}", e))?;
        
        let author = Author::new(author_name.trim().to_string());
        let author_id = author.id.clone();
        storage.add_author(author);
        
        // Save after author is added
        save_storage(storage_path, storage)?;

        // Get or create category
        let category_name = Text::new("Enter category name:")
            .prompt()
            .map_err(|e| format!("Failed to get category name: {}", e))?;
        
        let category = Category::new(
            category_name.trim().to_string(),
            None,
        );
        let category_id = category.id.clone();
        storage.add_category(category);
        
        // Save after category is added
        save_storage(storage_path, storage)?;

        // Create and add the book
        let book = Book::new(
            title.trim().to_string(),
            isbn.trim().to_string(),
            category_id,
            author_id,
            total_pages,
        );
        storage.add_book(book);
        
        // Save after book is added
        save_storage(storage_path, storage)?;
    }

    Ok(())
}

pub fn load_storage(storage_path: &str) -> Result<Storage, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let mut storage: Storage = serde_json::from_str(&contents)?;
    
    // Handle any missing fields
    handle_missing_fields(&mut storage, storage_path)?;
    
    Ok(storage)
}

pub fn save_storage(storage_path: &str, storage: &Storage) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(storage_path);
    
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Convert to JSON value, sort keys, then convert back to string
    let json_value = serde_json::to_value(storage)?;
    let sorted_value = sort_json_value(json_value);
    let json_string = serde_json::to_string_pretty(&sorted_value)?;
    
    // Write the storage data
    fs::write(path, json_string)?;
    
    Ok(())
} 