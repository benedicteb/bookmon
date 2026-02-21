use crate::series::{format_position_prefix, format_series_label};
use crate::storage::{compare_positions, Book, Reading, ReadingEvent, Storage};
use crate::table::{print_structured_table, print_table, TableRow};
use chrono::Utc;
use std::io;

/// A book entry in a sorted, grouped list — either a standalone book
/// or a group of books belonging to the same series.
#[derive(Debug)]
pub enum BookEntry<'a> {
    /// A book not belonging to any series.
    Standalone(&'a Book),
    /// A series group: series name + books sorted by position.
    SeriesGroup { name: String, books: Vec<&'a Book> },
}

/// Groups books by series and sorts the result for table display.
///
/// - Series groups are sorted internally by position (using `compare_positions`).
/// - Groups and standalone books are interleaved by the author name of the
///   "sort author" (first book in the series group, or the standalone book itself),
///   then by series name / book title.
/// - Within the same author, series groups come before standalone books.
pub fn group_books_by_series<'a>(storage: &'a Storage, books: &[&'a Book]) -> Vec<BookEntry<'a>> {
    use std::collections::HashMap;

    // Partition into series groups and standalone books
    let mut series_map: HashMap<&str, Vec<&'a Book>> = HashMap::new();
    let mut standalone: Vec<&'a Book> = Vec::new();

    for &book in books {
        if let Some(ref sid) = book.series_id {
            if storage.get_series(sid).is_some() {
                series_map.entry(sid.as_str()).or_default().push(book);
            } else {
                // Orphaned series_id — treat as standalone
                standalone.push(book);
            }
        } else {
            standalone.push(book);
        }
    }

    // Sort books within each series group by position
    for group_books in series_map.values_mut() {
        group_books.sort_by(|a, b| {
            compare_positions(
                a.position_in_series.as_deref(),
                b.position_in_series.as_deref(),
            )
        });
    }

    // Build entries with sort keys
    // Sort key: (author_name_lowercase, is_standalone (0=series, 1=standalone), group_name/title_lowercase)
    let mut entries: Vec<(String, u8, String, BookEntry<'a>)> = Vec::new();

    for (sid, group_books) in series_map {
        let series = storage.get_series(sid).unwrap(); // safe: checked above
                                                       // Sort author = author of the first (lowest position) book
        let sort_author = group_books
            .first()
            .map(|b| storage.author_name_for_book(b))
            .unwrap_or("");
        entries.push((
            sort_author.to_lowercase(),
            0, // series groups before standalone
            series.name.to_lowercase(),
            BookEntry::SeriesGroup {
                name: series.name.clone(),
                books: group_books,
            },
        ));
    }

    for book in standalone {
        let author = storage.author_name_for_book(book);
        entries.push((
            author.to_lowercase(),
            1, // standalone after series
            book.title.to_lowercase(),
            BookEntry::Standalone(book),
        ));
    }

    // Sort by (author, is_standalone, name/title)
    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

    entries.into_iter().map(|(_, _, _, entry)| entry).collect()
}

/// Validates and stores a reading event. Returns an error if the referenced book doesn't exist.
pub fn store_reading(storage: &mut Storage, reading: Reading) -> Result<(), String> {
    // Validate that the book exists
    if !storage.books.contains_key(&reading.book_id) {
        return Err(format!("Book with ID {} does not exist", reading.book_id));
    }

    storage.add_reading(reading);
    Ok(())
}

/// Builds the table data for currently-reading books.
/// Returns the table as a Vec of rows (first row is header).
/// The Series column is only included when at least one book has a series.
pub fn build_started_books_table(storage: &Storage) -> io::Result<Vec<Vec<String>>> {
    let started_books = storage.get_started_books();

    if started_books.is_empty() {
        return Ok(vec![]);
    }

    let mut sorted_books = started_books;
    sorted_books.sort_by(|a, b| {
        let a_author_name = storage.author_name_for_book(a);
        let b_author_name = storage.author_name_for_book(b);
        if a_author_name != b_author_name {
            a_author_name.cmp(b_author_name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    let any_has_series = sorted_books.iter().any(|b| b.series_id.is_some());

    let mut header = vec!["Title".to_string(), "Author".to_string()];
    if any_has_series {
        header.push("Series".to_string());
    }
    header.push("Days since started".to_string());
    header.push("Progress".to_string());

    let mut table_data = vec![header];

    for book in sorted_books {
        let author_name = storage.author_name_for_book(book);

        let series_label = book
            .series_id
            .as_ref()
            .and_then(|sid| storage.get_series(sid))
            .map(|s| format_series_label(s, book.position_in_series.as_deref()))
            .unwrap_or_default();

        let most_recent_reading = storage
            .readings
            .values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Started)
            .max_by_key(|r| r.created_on)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;

        let days = (Utc::now() - most_recent_reading.created_on).num_days();

        let most_recent_update = storage
            .readings
            .values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Update)
            .max_by_key(|r| r.created_on);

        let progress = if let Some(update) = most_recent_update {
            if let Some(current_page) = update.metadata.current_page {
                if book.total_pages > 0 {
                    format!(
                        "{:.1}%",
                        (current_page as f64 / book.total_pages as f64) * 100.0
                    )
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let mut row = vec![book.title.clone(), author_name.to_string()];
        if any_has_series {
            row.push(series_label);
        }
        row.push(days.to_string());
        row.push(progress);

        table_data.push(row);
    }

    Ok(table_data)
}

/// Displays a table of currently-reading books with author, days since started, and progress.
pub fn show_started_books(storage: &Storage) -> io::Result<()> {
    let table_data = build_started_books_table(storage)?;
    if table_data.is_empty() {
        println!("No books currently being read.");
    } else {
        print_table(&table_data);
    }
    Ok(())
}

/// Displays a table of finished books with author and finish date.
pub fn show_finished_books(storage: &Storage) -> io::Result<()> {
    show_finished_books_list(
        storage,
        storage.get_finished_books(),
        "No finished books found.",
    )
}

/// Displays a table of the given finished books with author and finish date.
pub fn show_finished_books_list(
    storage: &Storage,
    finished_books: Vec<&Book>,
    empty_message: &str,
) -> io::Result<()> {
    if finished_books.is_empty() {
        println!("{}", empty_message);
        return Ok(());
    }

    let mut sorted_books = finished_books;
    sorted_books.sort_by(|a, b| {
        let a_author_name = storage.author_name_for_book(a);
        let b_author_name = storage.author_name_for_book(b);
        if a_author_name != b_author_name {
            a_author_name.cmp(b_author_name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    let any_has_series = sorted_books.iter().any(|b| b.series_id.is_some());

    let mut header = vec!["Title".to_string(), "Author".to_string()];
    if any_has_series {
        header.push("Series".to_string());
    }
    header.push("Finished on".to_string());

    let mut table_data = vec![header];

    for book in sorted_books {
        let author_name = storage.author_name_for_book(book);

        let series_label = book
            .series_id
            .as_ref()
            .and_then(|sid| storage.get_series(sid))
            .map(|s| format_series_label(s, book.position_in_series.as_deref()))
            .unwrap_or_default();

        let most_recent_reading = storage
            .readings
            .values()
            .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Finished)
            .max_by_key(|r| r.created_on)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;

        let finished_date = most_recent_reading
            .created_on
            .format("%Y-%m-%d")
            .to_string();

        let mut row = vec![book.title.clone(), author_name.to_string()];
        if any_has_series {
            row.push(series_label);
        }
        row.push(finished_date);

        table_data.push(row);
    }

    print_table(&table_data);
    Ok(())
}

/// Prints a table of books with common columns (Title, Author, Category, Added on, Bought)
/// The Series column is only included when at least one book has a series.
pub fn print_book_list_table(
    storage: &Storage,
    books: Vec<&Book>,
    empty_message: &str,
) -> io::Result<()> {
    if books.is_empty() {
        println!("{}", empty_message);
        return Ok(());
    }

    let mut sorted_books = books;
    sorted_books.sort_by(|a, b| {
        let a_author_name = storage.author_name_for_book(a);
        let b_author_name = storage.author_name_for_book(b);
        if a_author_name != b_author_name {
            a_author_name.cmp(b_author_name)
        } else {
            a.title.cmp(&b.title)
        }
    });

    let any_has_series = sorted_books.iter().any(|b| b.series_id.is_some());

    let mut header = vec!["Title".to_string(), "Author".to_string()];
    if any_has_series {
        header.push("Series".to_string());
    }
    header.extend([
        "Category".to_string(),
        "Added on".to_string(),
        "Bought".to_string(),
        "Want to read".to_string(),
    ]);

    let mut table_data = vec![header];

    let want_to_read_ids: std::collections::HashSet<&str> = storage
        .get_want_to_read_books()
        .iter()
        .map(|b| b.id.as_str())
        .collect();

    for book in sorted_books {
        let author_name = storage.author_name_for_book(book);

        let category = storage
            .categories
            .get(&book.category_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Category not found"))?;

        let series_label = book
            .series_id
            .as_ref()
            .and_then(|sid| storage.get_series(sid))
            .map(|s| format_series_label(s, book.position_in_series.as_deref()))
            .unwrap_or_default();

        let has_bought_event = storage
            .readings
            .values()
            .any(|r| r.book_id == book.id && r.event == ReadingEvent::Bought);

        let is_want_to_read = want_to_read_ids.contains(book.id.as_str());

        let added_date = book.added_on.format("%Y-%m-%d").to_string();

        let mut row = vec![book.title.clone(), author_name.to_string()];
        if any_has_series {
            row.push(series_label);
        }
        row.extend([
            category.name.clone(),
            added_date,
            if has_bought_event {
                "x".to_string()
            } else {
                "".to_string()
            },
            if is_want_to_read {
                "x".to_string()
            } else {
                "".to_string()
            },
        ]);

        table_data.push(row);
    }

    print_table(&table_data);
    Ok(())
}
