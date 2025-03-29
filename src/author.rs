use crate::storage::{Storage, Author};

pub fn get_author_input() -> Result<Author, String> {
    println!("Enter author details:");
    
    let name = dialoguer::Input::<String>::new()
        .with_prompt("Name")
        .interact()
        .map_err(|e| e.to_string())?;

    let id = uuid::Uuid::new_v4().to_string();

    Ok(Author { id, name })
}

pub fn store_author(storage: &mut Storage, author: Author) -> Result<(), String> {
    storage.add_author(author);
    Ok(())
}

pub fn get_author_by_id(storage: &Storage, author_id: &str) -> Result<Option<Author>, String> {
    Ok(storage.get_author(author_id).cloned())
} 