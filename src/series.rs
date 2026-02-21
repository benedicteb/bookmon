use crate::storage::{Series, Storage};

/// Stores a series in the storage.
pub fn store_series(storage: &mut Storage, series: Series) -> Result<(), String> {
    storage.add_series(series);
    Ok(())
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
