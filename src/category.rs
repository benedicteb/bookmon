use std::io::{self, Write};
use crate::storage::{Category, Storage};

pub fn get_category_input() -> io::Result<Category> {
    let mut name = String::new();
    let mut description = String::new();

    print!("Enter category name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut name)?;

    print!("Enter category description (optional, press Enter to skip): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut description)?;

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