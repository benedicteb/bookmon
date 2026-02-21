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

/// Strips comment lines (starting with #) and trims whitespace from editor text.
/// Returns None if the resulting text is empty (indicating the user aborted).
pub fn strip_editor_text(text: &str) -> Option<String> {
    let stripped: String = text
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string();

    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

/// Opens the user's default editor with a temporary file for writing a review.
///
/// The editor is determined by checking $EDITOR, then $VISUAL, falling back to "vi".
/// The temp file is pre-populated with comment instructions that are stripped after editing.
pub fn get_review_text_from_editor(
    book_title: &str,
    author_name: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    // Create a named temp file with .md extension for editor syntax highlighting
    let mut tmp = NamedTempFile::new()?;

    let template = format!(
        "\n# Write your review of \"{}\" by {} above.\n# Lines starting with # will be stripped.\n# An empty review (after stripping comments) will abort.\n",
        book_title, author_name
    );
    write!(tmp, "{}", template)?;
    tmp.flush()?;

    let path = tmp.path().to_path_buf();

    // Split editor command to support values like "code --wait" or "subl -w"
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (editor_bin, editor_args) = parts
        .split_first()
        .ok_or("$EDITOR is empty after splitting")?;

    let status = std::process::Command::new(editor_bin)
        .args(editor_args)
        .arg(&path)
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(format!("Editor '{}' exited with non-zero status", editor).into());
    }

    let content = std::fs::read_to_string(&path)?;
    Ok(strip_editor_text(&content))
}

/// Displays a table of all reviews with book title, author, date, and a text preview.
pub fn show_reviews(storage: &Storage) -> io::Result<()> {
    use crate::table::print_table;

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

    print_table(&table_data);
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
