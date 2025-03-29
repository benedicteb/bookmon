use std::fs;
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

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
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ReadingEvent {
    Finished,
    Started,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reading {
    pub id: String,
    pub created_on: DateTime<Utc>,
    pub book_id: String,
    pub event: ReadingEvent,
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
    pub fn new(title: String, isbn: String, category_id: String, author_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            added_on: Utc::now(),
            isbn,
            category_id,
            author_id,
        }
    }
}

impl Reading {
    pub fn new(book_id: String, event: ReadingEvent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
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
}

pub fn initialize_storage_file(storage_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(storage_path);
    
    if !path.exists() {
        let initial_storage = Storage::new();
        
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the initial data
        fs::write(
            path,
            serde_json::to_string_pretty(&initial_storage)?,
        )?;
    }
    
    Ok(())
}

pub fn load_storage(storage_path: &str) -> Result<Storage, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let storage: Storage = serde_json::from_str(&contents)?;
    Ok(storage)
}

pub fn save_storage(storage_path: &str, storage: &Storage) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(storage_path);
    
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Write the storage data
    fs::write(
        path,
        serde_json::to_string_pretty(storage)?,
    )?;
    
    Ok(())
} 