use crate::diff::{line_diff, DiffLine};
use crate::storage::{Reading, ReadingEvent, Storage};
use std::io;

/// Records a review for a book, creating it or revising the existing one.
///
/// The first review becomes a `CreateReview` event; every later one becomes an
/// `EditReview`. Text identical to the current version records nothing, so the
/// timeline never shows an empty diff.
///
/// # Errors
///
/// Returns an error if `book_id` does not refer to an existing book.
pub fn store_review(storage: &mut Storage, book_id: &str, text: String) -> Result<(), String> {
    if !storage.books.contains_key(book_id) {
        return Err(format!("Book with ID {} does not exist", book_id));
    }

    let event = match storage.review_for_book(book_id) {
        Some(existing) if existing.text == text => return Ok(()),
        Some(_) => ReadingEvent::EditReview,
        None => ReadingEvent::CreateReview,
    };

    let reading = Reading::with_review(book_id.to_string(), event, text);
    storage.readings.insert(reading.id.clone(), reading);
    Ok(())
}

/// Builds the editor template shown when writing or revising a review.
///
/// `current` pre-fills the buffer when revising, and switches the heading
/// verb from "Write" to "Edit". An untouched template strips to `None`
/// (create) or back to `current` unchanged (edit), which is how the user
/// aborts or leaves the review as-is.
pub fn review_template(book_title: &str, author_name: &str, current: Option<&str>) -> String {
    let verb = if current.is_some() { "Edit" } else { "Write" };
    // Bound to a let, not inlined: an array literal must have one element
    // type, and `&format!(..)` is a `&String` beside the `&str` literals.
    let heading = format!(
        "{} your review of \"{}\" by {} above.",
        verb, book_title, author_name
    );
    format!(
        "{}\n\n{}",
        current.unwrap_or(""),
        crate::editor::instruction_block(&[
            heading.as_str(),
            "Everything below this line is ignored.",
            "An empty review aborts. Unchanged text records no edit.",
        ])
    )
}

/// Opens the user's default editor for writing or revising a review.
///
/// `current` pre-fills the buffer when revising. Returns None if the body is
/// empty (user aborted).
pub fn get_review_text_from_editor(
    book_title: &str,
    author_name: &str,
    current: Option<&str>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let template = review_template(book_title, author_name, current);
    crate::editor::get_text_from_editor(&template)
}

/// Displays a table of all reviews with book title, author, date, and a text preview.
pub fn show_reviews(storage: &Storage) -> io::Result<()> {
    use crate::table::{print_table, Alignment};

    let reviews = storage.all_reviews();

    if reviews.is_empty() {
        println!("No reviews found.");
        return Ok(());
    }

    let mut table_data = vec![vec![
        "Title".to_string(),
        "Author".to_string(),
        "Date".to_string(),
        "Edits".to_string(),
        "Preview".to_string(),
    ]];

    for review in reviews {
        let book = storage.books.get(&review.book_id);
        let title = book.map(|b| b.title.as_str()).unwrap_or("Unknown Book");
        let author_name = book
            .map(|b| storage.author_name_for_book(b))
            .unwrap_or("Unknown Author");
        let date = review.created_on.format("%Y-%m-%d").to_string();
        let edits = review.edit_count();
        let edits_cell = if edits == 0 {
            String::new()
        } else {
            edits.to_string()
        };
        let preview = truncate_text(&review.text, 60);

        table_data.push(vec![
            title.to_string(),
            author_name.to_string(),
            date,
            edits_cell,
            preview,
        ]);
    }

    let alignments = [
        Alignment::Left,  // Title
        Alignment::Left,  // Author
        Alignment::Right, // Date
        Alignment::Right, // Edits
        Alignment::Left,  // Preview
    ];
    print_table(&table_data, &alignments);
    Ok(())
}

/// Renders the full review detail view: current text, then the history.
///
/// Returns `None` if the book has no review. Returned rather than printed so
/// the layout can be tested without capturing stdout.
pub fn format_review_detail(storage: &Storage, book_id: &str) -> Option<String> {
    let review = storage.review_for_book(book_id)?;
    let book = storage.books.get(book_id);
    let title = book.map(|b| b.title.as_str()).unwrap_or("Unknown Book");
    let author_name = book
        .map(|b| storage.author_name_for_book(b))
        .unwrap_or("Unknown Author");

    let rule = "-".repeat(60);
    let mut out = String::new();

    out.push_str(&format!("\nReview of \"{}\" by {}\n", title, author_name));
    out.push_str(&format!(
        "Written on {}\n",
        review.created_on.format("%Y-%m-%d")
    ));

    let edits = review.edit_count();
    if edits > 0 {
        out.push_str(&format!(
            "Last edited on {} ({} edit{})\n",
            review.updated_on.format("%Y-%m-%d"),
            edits,
            if edits == 1 { "" } else { "s" }
        ));
    }

    out.push_str(&format!("{}\n{}\n", rule, review.text));

    if edits > 0 {
        out.push_str(&format!("\nHistory\n{}\n", rule));

        // Newest first, matching how reviews are listed elsewhere.
        for index in (1..review.revisions.len()).rev() {
            let previous = &review.revisions[index - 1];
            let current = &review.revisions[index];

            out.push_str(&format!(
                "Edited on {}\n",
                current.created_on.format("%Y-%m-%d")
            ));
            for line in line_diff(&previous.text, &current.text) {
                out.push_str(&match line {
                    DiffLine::Context(text) => format!("    {}\n", text),
                    DiffLine::Added(text) => format!("  + {}\n", text),
                    DiffLine::Removed(text) => format!("  - {}\n", text),
                });
            }
            out.push('\n');
        }

        let original = &review.revisions[0];
        out.push_str(&format!(
            "Written on {}\n",
            original.created_on.format("%Y-%m-%d")
        ));
        for line in original.text.lines() {
            out.push_str(&format!("    {}\n", line));
        }
    }

    out.push('\n');
    Some(out)
}

/// Displays the full text of a single review, with its edit history.
///
/// # Errors
///
/// Returns an error if the book has never been reviewed.
pub fn show_review_detail(storage: &Storage, book_id: &str) -> io::Result<()> {
    let rendered = format_review_detail(storage, book_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Review not found"))?;
    print!("{}", rendered);
    Ok(())
}

/// Truncates text to a maximum number of characters, appending "..." if truncated.
/// Replaces newlines with spaces for single-line display.
/// Uses char count (not byte count) to avoid panicking on multi-byte UTF-8 characters.
fn truncate_text(text: &str, max_chars: usize) -> String {
    let single_line = text.replace('\n', " ");
    if single_line.chars().count() <= max_chars {
        single_line
    } else {
        let truncated: String = single_line
            .chars()
            .take(max_chars.saturating_sub(3))
            .collect();
        format!("{}...", truncated)
    }
}
