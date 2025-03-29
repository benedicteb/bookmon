use std::io;
use crate::storage::{Category, Storage};
use inquire::Text;

pub fn get_category_input() -> io::Result<Category> {
    let name = Text::new("Enter category name:")
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let description = Text::new("Enter category description (optional, press Enter to skip):")
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(Category::new(
        name.trim().to_string(),
        if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_string())
        },
    ))
}

pub fn store_category(storage: &mut Storage, category: Category) -> Result<(), String> {
    storage.categories.insert(category.id.clone(), category);
    Ok(())
} 