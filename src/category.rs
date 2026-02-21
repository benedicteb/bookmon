use crate::storage::{Category, Storage};

pub fn store_category(storage: &mut Storage, category: Category) -> Result<(), String> {
    storage.categories.insert(category.id.clone(), category);
    Ok(())
}
