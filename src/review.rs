use crate::storage::{Review, Storage};
use std::io;

/// Validates and stores a review. Returns an error if the referenced book doesn't exist.
pub fn store_review(storage: &mut Storage, review: Review) -> Result<(), String> {
    if !storage.books.contains_key(&review.book_id) {
        return Err(format!("Book with ID {} does not exist", review.book_id));
    }

    storage.add_review(review);
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

    let mut reviews: Vec<&Review> = storage.reviews.values().collect();

    if reviews.is_empty() {
        println!("No reviews found.");
        return Ok(());
    }

    // Sort by creation date, newest first
    reviews.sort_by(|a, b| b.created_on.cmp(&a.created_on));

    let mut table_data = vec![vec![
        "Title".to_string(),
        "Author".to_string(),
        "Date".to_string(),
        "Preview".to_string(),
    ]];

    for review in reviews {
        let book = storage.books.get(&review.book_id);
        let title = book.map(|b| b.title.as_str()).unwrap_or("Unknown Book");
        let author_name = book
            .map(|b| storage.author_name_for_book(b))
            .unwrap_or("Unknown Author");
        let date = review.created_on.format("%Y-%m-%d").to_string();
        let preview = truncate_text(&review.text, 60);

        table_data.push(vec![
            title.to_string(),
            author_name.to_string(),
            date,
            preview,
        ]);
    }

    let alignments = [
        Alignment::Left,  // Title
        Alignment::Left,  // Author
        Alignment::Right, // Date
        Alignment::Left,  // Preview
    ];
    print_table(&table_data, &alignments);
    Ok(())
}

/// Displays the full text of a single review.
pub fn show_review_detail(storage: &Storage, review_id: &str) -> io::Result<()> {
    let review = storage
        .reviews
        .get(review_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Review not found"))?;

    let book = storage.books.get(&review.book_id);
    let title = book.map(|b| b.title.as_str()).unwrap_or("Unknown Book");
    let author_name = book
        .map(|b| storage.author_name_for_book(b))
        .unwrap_or("Unknown Author");
    let date = review.created_on.format("%Y-%m-%d").to_string();

    println!();
    println!("Review of \"{}\" by {}", title, author_name);
    println!("Written on {}", date);
    println!("{}", "-".repeat(60));
    println!("{}", review.text);
    println!();

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
