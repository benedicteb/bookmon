use crate::series::format_position_prefix;
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

/// Builds the structured table data for currently-reading books.
///
/// Returns `Vec<TableRow>` with series grouping when any book has a series,
/// or a flat table when no books have series. Returns empty vec if no started books.
pub fn build_started_books_table(storage: &Storage) -> io::Result<Vec<TableRow>> {
    let started_books = storage.get_started_books();

    if started_books.is_empty() {
        return Ok(vec![]);
    }

    let any_has_series = started_books.iter().any(|b| b.series_id.is_some());

    let header = vec![
        "Title".to_string(),
        "Author".to_string(),
        "Days since started".to_string(),
        "Progress".to_string(),
    ];

    let mut table_rows = vec![TableRow::Header(header)];

    if any_has_series {
        let entries = group_books_by_series(storage, &started_books);

        for entry in &entries {
            match entry {
                BookEntry::SeriesGroup { name, books } => {
                    table_rows.push(TableRow::GroupHeader(name.clone()));
                    for book in books {
                        let title = format!(
                            "{}{}",
                            format_position_prefix(book.position_in_series.as_deref()),
                            book.title
                        );
                        let row = build_started_book_row(storage, book, title)?;
                        table_rows.push(TableRow::Data(row));
                    }
                }
                BookEntry::Standalone(book) => {
                    let row = build_started_book_row(storage, book, book.title.clone())?;
                    table_rows.push(TableRow::Data(row));
                }
            }
        }
    } else {
        let mut sorted_books = started_books;
        sorted_books.sort_by(|a, b| {
            let a_author = storage.author_name_for_book(a);
            let b_author = storage.author_name_for_book(b);
            a_author.cmp(b_author).then(a.title.cmp(&b.title))
        });

        for book in sorted_books {
            let row = build_started_book_row(storage, book, book.title.clone())?;
            table_rows.push(TableRow::Data(row));
        }
    }

    Ok(table_rows)
}

/// Builds a data row for a currently-reading book.
fn build_started_book_row(
    storage: &Storage,
    book: &Book,
    title: String,
) -> io::Result<Vec<String>> {
    let author_name = storage.author_name_for_book(book);

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

    Ok(vec![
        title,
        author_name.to_string(),
        days.to_string(),
        progress,
    ])
}

/// Displays a table of currently-reading books with author, days since started, and progress.
pub fn show_started_books(storage: &Storage) -> io::Result<()> {
    let table_rows = build_started_books_table(storage)?;
    if table_rows.is_empty() {
        println!("No books currently being read.");
    } else {
        print_structured_table(&table_rows);
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
///
/// When books belong to series, they are grouped under a series header row
/// with no separators between books in the same group. The Series column is
/// replaced by position prefixes (e.g. `#1`) on the book title.
pub fn show_finished_books_list(
    storage: &Storage,
    finished_books: Vec<&Book>,
    empty_message: &str,
) -> io::Result<()> {
    if finished_books.is_empty() {
        println!("{}", empty_message);
        return Ok(());
    }

    let any_has_series = finished_books.iter().any(|b| b.series_id.is_some());

    if any_has_series {
        let entries = group_books_by_series(storage, &finished_books);
        let header = vec![
            "Title".to_string(),
            "Author".to_string(),
            "Finished on".to_string(),
        ];
        let mut table_rows = vec![TableRow::Header(header)];

        for entry in &entries {
            match entry {
                BookEntry::SeriesGroup { name, books } => {
                    table_rows.push(TableRow::GroupHeader(name.clone()));
                    for book in books {
                        let title = format!(
                            "{}{}",
                            format_position_prefix(book.position_in_series.as_deref()),
                            book.title
                        );
                        let author_name = storage.author_name_for_book(book);
                        let finished_date = finished_date_for_book(storage, book)?;
                        table_rows.push(TableRow::Data(vec![
                            title,
                            author_name.to_string(),
                            finished_date,
                        ]));
                    }
                }
                BookEntry::Standalone(book) => {
                    let author_name = storage.author_name_for_book(book);
                    let finished_date = finished_date_for_book(storage, book)?;
                    table_rows.push(TableRow::Data(vec![
                        book.title.clone(),
                        author_name.to_string(),
                        finished_date,
                    ]));
                }
            }
        }

        print_structured_table(&table_rows);
    } else {
        // No series — use the flat table
        let mut sorted_books = finished_books;
        sorted_books.sort_by(|a, b| {
            let a_author = storage.author_name_for_book(a);
            let b_author = storage.author_name_for_book(b);
            a_author.cmp(b_author).then(a.title.cmp(&b.title))
        });

        let header = vec![
            "Title".to_string(),
            "Author".to_string(),
            "Finished on".to_string(),
        ];
        let mut table_data = vec![header];

        for book in sorted_books {
            let author_name = storage.author_name_for_book(book);
            let finished_date = finished_date_for_book(storage, book)?;
            table_data.push(vec![
                book.title.clone(),
                author_name.to_string(),
                finished_date,
            ]);
        }

        print_table(&table_data);
    }
    Ok(())
}

/// Returns the formatted finish date for a book (most recent Finished event).
fn finished_date_for_book(storage: &Storage, book: &Book) -> io::Result<String> {
    let most_recent_reading = storage
        .readings
        .values()
        .filter(|r| r.book_id == book.id && r.event == ReadingEvent::Finished)
        .max_by_key(|r| r.created_on)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Reading not found"))?;
    Ok(most_recent_reading
        .created_on
        .format("%Y-%m-%d")
        .to_string())
}

/// Prints a table of books with common columns (Title, Author, Category, Added on, Bought, Want to read).
///
/// When books belong to series, they are grouped under series header rows
/// with position prefixes on titles. The Series column is omitted.
pub fn print_book_list_table(
    storage: &Storage,
    books: Vec<&Book>,
    empty_message: &str,
) -> io::Result<()> {
    if books.is_empty() {
        println!("{}", empty_message);
        return Ok(());
    }

    let any_has_series = books.iter().any(|b| b.series_id.is_some());

    let want_to_read_ids: std::collections::HashSet<&str> = storage
        .get_want_to_read_books()
        .iter()
        .map(|b| b.id.as_str())
        .collect();

    let header = vec![
        "Title".to_string(),
        "Author".to_string(),
        "Category".to_string(),
        "Added on".to_string(),
        "Bought".to_string(),
        "Want to read".to_string(),
    ];

    if any_has_series {
        let entries = group_books_by_series(storage, &books);
        let mut table_rows = vec![TableRow::Header(header)];

        for entry in &entries {
            match entry {
                BookEntry::SeriesGroup { name, books } => {
                    table_rows.push(TableRow::GroupHeader(name.clone()));
                    for book in books {
                        let title = format!(
                            "{}{}",
                            format_position_prefix(book.position_in_series.as_deref()),
                            book.title
                        );
                        let row = build_book_list_row(storage, book, title, &want_to_read_ids)?;
                        table_rows.push(TableRow::Data(row));
                    }
                }
                BookEntry::Standalone(book) => {
                    let row =
                        build_book_list_row(storage, book, book.title.clone(), &want_to_read_ids)?;
                    table_rows.push(TableRow::Data(row));
                }
            }
        }

        print_structured_table(&table_rows);
    } else {
        let mut sorted_books = books;
        sorted_books.sort_by(|a, b| {
            let a_author = storage.author_name_for_book(a);
            let b_author = storage.author_name_for_book(b);
            a_author.cmp(b_author).then(a.title.cmp(&b.title))
        });

        let mut table_data = vec![header];

        for book in sorted_books {
            let row = build_book_list_row(storage, book, book.title.clone(), &want_to_read_ids)?;
            table_data.push(row);
        }

        print_table(&table_data);
    }
    Ok(())
}

/// Builds a data row for the book list table (backlog / want-to-read).
fn build_book_list_row(
    storage: &Storage,
    book: &Book,
    title: String,
    want_to_read_ids: &std::collections::HashSet<&str>,
) -> io::Result<Vec<String>> {
    let author_name = storage.author_name_for_book(book);

    let category = storage
        .categories
        .get(&book.category_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Category not found"))?;

    let has_bought_event = storage
        .readings
        .values()
        .any(|r| r.book_id == book.id && r.event == ReadingEvent::Bought);

    let is_want_to_read = want_to_read_ids.contains(book.id.as_str());

    let added_date = book.added_on.format("%Y-%m-%d").to_string();

    Ok(vec![
        title,
        author_name.to_string(),
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
    ])
}
