use crate::storage::{Series, Storage};

/// Stores a series in the storage.
pub fn store_series(storage: &mut Storage, series: Series) -> Result<(), String> {
    storage.add_series(series);
    Ok(())
}

/// Formats a series label for display, e.g. "Harry Potter #3" or "Harry Potter" (if no position).
pub fn format_series_label(series: &Series, position: Option<&str>) -> String {
    match position {
        Some(pos) => format!("{} #{}", series.name, pos),
        None => series.name.clone(),
    }
}

/// Formats a position prefix for a book title within a grouped series display.
///
/// Returns e.g. `"#3 "` for position "3", or `""` if no position is set.
/// Used in table rows where the series name is shown in a group header,
/// so only the position number is needed next to the title.
pub fn format_position_prefix(position: Option<&str>) -> String {
    match position {
        Some(pos) => format!("#{} ", pos),
        None => String::new(),
    }
}

/// Parses a position-in-series input string. Returns `Some(position)` for valid
/// non-negative numbers (integers like "1", "0" or decimals like "2.5").
/// Returns `None` for empty/whitespace, negative numbers, or non-numeric input.
pub fn parse_position_input(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<f64>() {
        Ok(val) if val >= 0.0 => Some(trimmed.to_string()),
        _ => None,
    }
}

/// Formats a rich display of a series with reading status indicators.
///
/// Output includes:
/// - Header line with series name and progress (e.g. "Harry Potter (3/7 read)")
/// - Unicode separator line
/// - Books listed with status indicators: `✓` finished, `▸` reading, blank for unread
///
/// Returns the formatted string (no trailing newline).
pub fn format_series_display(storage: &Storage, series_id: &str) -> String {
    let series = match storage.get_series(series_id) {
        Some(s) => s,
        None => return String::new(),
    };

    let books = storage.get_books_in_series(series_id);
    let mut lines = Vec::new();

    if books.is_empty() {
        // Header with no progress
        lines.push(series.name.to_string());
        lines.push("\u{2500}".repeat(series.name.len()));
        lines.push("  (no books)".to_string());
        return lines.join("\n");
    }

    // Count reading status
    let finished_count = books
        .iter()
        .filter(|b| storage.is_book_finished(&b.id))
        .count();
    let reading_count = books
        .iter()
        .filter(|b| storage.is_book_started(&b.id))
        .count();

    // Build header with progress
    let progress = match series.total_books {
        Some(total) => {
            let mut parts = vec![format!("{}/{} read", finished_count, total)];
            if reading_count > 0 {
                parts.push(format!("{} reading", reading_count));
            }
            parts.join(", ")
        }
        None => {
            let mut parts = vec![format!("{} read", finished_count)];
            if reading_count > 0 {
                parts.push(format!("{} reading", reading_count));
            }
            parts.join(", ")
        }
    };

    let header = format!("{} ({})", series.name, progress);
    lines.push(header.clone());
    lines.push("\u{2500}".repeat(header.len()));

    // List books with status indicators
    for book in &books {
        let is_finished = storage.is_book_finished(&book.id);
        let is_started = storage.is_book_started(&book.id);

        let status_indicator = if is_finished {
            "\u{2713}" // ✓
        } else if is_started {
            "\u{25b8}" // ▸
        } else {
            " "
        };

        let author_name = storage.author_name_for_book(book);
        let author_name = if author_name.is_empty() {
            "Unknown Author"
        } else {
            author_name
        };

        let pos = book
            .position_in_series
            .as_deref()
            .map(|p| format!("#{} ", p))
            .unwrap_or_default();

        lines.push(format!(
            "  {}{} \"{}\" by {}",
            pos, status_indicator, book.title, author_name
        ));
    }

    lines.join("\n")
}

/// Checks if a position is already occupied by another book in the series.
/// Returns the title of the book at that position, or None if the position is free.
pub fn is_position_occupied(storage: &Storage, series_id: &str, position: &str) -> Option<String> {
    storage
        .books
        .values()
        .find(|b| {
            b.series_id.as_deref() == Some(series_id)
                && b.position_in_series.as_deref() == Some(position)
        })
        .map(|b| b.title.clone())
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
        return Err("Series not found. It may have already been deleted.".to_string());
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

/// Filters a list of books to only those belonging to a series whose name
/// contains `filter` (case-insensitive substring match).
/// Standalone books (no series) are always excluded.
pub fn filter_books_by_series<'a>(
    storage: &Storage,
    books: &[&'a crate::storage::Book],
    filter: &str,
) -> Vec<&'a crate::storage::Book> {
    let filter_lower = filter.to_lowercase();
    books
        .iter()
        .filter(|book| {
            book.series_id
                .as_ref()
                .and_then(|sid| storage.get_series(sid))
                .map(|s| s.name.to_lowercase().contains(&filter_lower))
                .unwrap_or(false)
        })
        .copied()
        .collect()
}

/// Returns the names of all series whose name contains `filter` (case-insensitive substring).
/// Useful for suggesting alternatives when a filter matches no books.
pub fn find_matching_series_names(storage: &Storage, filter: &str) -> Vec<String> {
    let filter_lower = filter.to_lowercase();
    let mut names: Vec<String> = storage
        .series
        .values()
        .filter(|s| s.name.to_lowercase().contains(&filter_lower))
        .map(|s| s.name.clone())
        .collect();
    names.sort_by_key(|a| a.to_lowercase());
    names
}

/// Builds a helpful empty-result message when a `--series` filter yields no books.
///
/// - If the filter term matches known series names, tells the user no books matched.
/// - If the filter term matches no series at all, lists known series as suggestions.
pub fn format_series_filter_empty_message(storage: &Storage, filter: &str) -> String {
    let matching_series = find_matching_series_names(storage, filter);
    if !matching_series.is_empty() {
        format!("No books found matching series \"{}\".", filter)
    } else {
        let mut all_names: Vec<String> = storage.series.values().map(|s| s.name.clone()).collect();
        all_names.sort_by_key(|a| a.to_lowercase());
        if all_names.is_empty() {
            format!(
                "No series matching \"{}\" found. No series exist yet.",
                filter
            )
        } else {
            format!(
                "No series matching \"{}\" found. Known series: {}.",
                filter,
                all_names.join(", ")
            )
        }
    }
}

/// Renames a series. Returns an error if the series does not exist, if the new
/// name is empty, or if another series with the new name already exists (case-insensitive).
pub fn rename_series(storage: &mut Storage, series_id: &str, new_name: &str) -> Result<(), String> {
    let new_name_trimmed = new_name.trim();
    if new_name_trimmed.is_empty() {
        return Err("Series name cannot be empty".to_string());
    }

    // Check that the series exists
    if !storage.series.contains_key(series_id) {
        return Err("Series not found. It may have already been deleted.".to_string());
    }

    // Check for duplicate name (case-insensitive), excluding the series being renamed
    let duplicate = storage
        .series
        .iter()
        .any(|(id, s)| id != series_id && s.name.to_lowercase() == new_name_trimmed.to_lowercase());
    if duplicate {
        return Err(format!(
            "A series named '{}' already exists",
            new_name_trimmed
        ));
    }

    // Rename
    if let Some(series) = storage.series.get_mut(series_id) {
        series.name = new_name_trimmed.to_string();
    }

    Ok(())
}
