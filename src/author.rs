use crate::storage::{Author, Storage};

/// Stores an author in the storage.
pub fn store_author(storage: &mut Storage, author: Author) -> Result<(), String> {
    storage.add_author(author);
    Ok(())
}

/// Retrieves an author by ID, returning a cloned copy if found.
pub fn get_author_by_id(storage: &Storage, author_id: &str) -> Result<Option<Author>, String> {
    Ok(storage.get_author(author_id).cloned())
}
