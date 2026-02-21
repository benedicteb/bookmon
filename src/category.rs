use crate::storage::{Category, Storage};

/// Stores a category in the storage.
pub fn store_category(storage: &mut Storage, category: Category) -> Result<(), String> {
    storage.categories.insert(category.id.clone(), category);
    Ok(())
}
