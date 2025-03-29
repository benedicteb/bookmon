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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub added_on: DateTime<Utc>,
    pub isbn: String,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reading {
    pub id: String,
    pub created_on: DateTime<Utc>,
    pub book_id: String,
    pub event: String,
}

impl Author {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
        }
    }
}

impl Book {
    pub fn new(title: String, isbn: String, category: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            added_on: Utc::now(),
            isbn,
            category,
        }
    }
}

impl Reading {
    pub fn new(book_id: String, event: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    pub books: HashMap<String, Book>,
    pub readings: HashMap<String, Reading>,
    pub authors: HashMap<String, Author>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            books: HashMap::new(),
            readings: HashMap::new(),
            authors: HashMap::new(),
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

    pub fn get_book(&self, id: &str) -> Option<&Book> {
        self.books.get(id)
    }

    pub fn get_reading(&self, id: &str) -> Option<&Reading> {
        self.readings.get(id)
    }

    pub fn get_author(&self, id: &str) -> Option<&Author> {
        self.authors.get(id)
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