use crate::storage::{Series, Storage};

/// Stores a series in the storage.
pub fn store_series(storage: &mut Storage, series: Series) -> Result<(), String> {
    storage.add_series(series);
    Ok(())
}

/// Formats a series label for display, e.g. "Harry Potter #3" or "Harry Potter" (if no position).
pub fn format_series_label(series: &Series, position: Option<i32>) -> String {
    match position {
        Some(pos) => format!("{} #{}", series.name, pos),
        None => series.name.clone(),
    }
}

/// Parses a position-in-series input string. Returns `Some(pos)` for valid
/// positive integers, `None` for empty/whitespace, zero, negative, or non-numeric input.
pub fn parse_position_input(input: &str) -> Option<i32> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<i32>() {
        Ok(pos) if pos > 0 => Some(pos),
        _ => None,
    }
}

/// Finds an existing series by name (case-insensitive) or creates a new one.
/// Returns the series ID.
pub fn get_or_create_series(storage: &mut Storage, name: &str) -> String {
    // Look for existing series with case-insensitive name match
    if let Some((id, _)) = storage
        .series
        .iter()
        .find(|(_, s)| s.name.to_lowercase() == name.to_lowercase())
    {
        return id.clone();
    }

    // Create a new series
    let series = Series::new(name.to_string());
    let id = series.id.clone();
    storage.add_series(series);
    id
}

/// Deletes a series and unlinks all books that belong to it.
/// Returns an error if the series does not exist.
pub fn delete_series(storage: &mut Storage, series_id: &str) -> Result<(), String> {
    if storage.series.remove(series_id).is_none() {
        return Err(format!("Series with ID '{}' not found", series_id));
    }

    // Unlink all books from this series
    for book in storage.books.values_mut() {
        if book.series_id.as_deref() == Some(series_id) {
            book.series_id = None;
            book.position_in_series = None;
        }
    }

    Ok(())
}

/// Renames a series. Returns an error if the series does not exist or if another
/// series with the new name already exists (case-insensitive).
pub fn rename_series(storage: &mut Storage, series_id: &str, new_name: &str) -> Result<(), String> {
    // Check that the series exists
    if !storage.series.contains_key(series_id) {
        return Err(format!("Series with ID '{}' not found", series_id));
    }

    // Check for duplicate name (case-insensitive), excluding the series being renamed
    let duplicate = storage
        .series
        .iter()
        .any(|(id, s)| id != series_id && s.name.to_lowercase() == new_name.to_lowercase());
    if duplicate {
        return Err(format!("A series named '{}' already exists", new_name));
    }

    // Rename
    if let Some(series) = storage.series.get_mut(series_id) {
        series.name = new_name.to_string();
    }

    Ok(())
}
