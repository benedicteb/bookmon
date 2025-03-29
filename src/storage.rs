use std::fs;
use std::path::Path;
use serde_json::{json, Value};

pub fn initialize_storage_file(storage_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(storage_path);
    
    if !path.exists() {
        let initial_data = json!({
            "books": {},
            "readings": {}
        });
        
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the initial data
        fs::write(
            path,
            serde_json::to_string_pretty(&initial_data)?,
        )?;
    }
    
    Ok(())
}

pub fn load_storage(storage_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let data: Value = serde_json::from_str(&contents)?;
    Ok(data)
} 