use std::fs;
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub added_on: DateTime<Utc>,
    pub isbn: String,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reading {
    pub id: String,
    pub created_on: DateTime<Utc>,
    pub book_id: String,
    pub event: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    pub books: HashMap<String, Book>,
    pub readings: HashMap<String, Reading>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            books: HashMap::new(),
            readings: HashMap::new(),
        }
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