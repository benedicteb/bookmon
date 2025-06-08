use crate::storage::{Author, Storage};
use inquire::Text;

pub fn get_author_input() -> Result<Author, String> {
    let name = Text::new("Enter author name:")
        .prompt()
        .map_err(|e| e.to_string())?;

    Ok(Author::new(name))
}

pub fn store_author(storage: &mut Storage, author: Author) -> Result<(), String> {
    storage.add_author(author);
    Ok(())
}

pub fn get_author_by_id(storage: &Storage, author_id: &str) -> Result<Option<Author>, String> {
    Ok(storage.get_author(author_id).cloned())
}
