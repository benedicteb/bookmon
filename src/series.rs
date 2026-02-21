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
