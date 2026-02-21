use chrono::{DateTime, Datelike, Utc};

/// Generates motivational text about the reading pace needed to reach a yearly goal.
///
/// Takes the current progress (`finished`, `target`) and the goal `year`, plus
/// the current time `now` (injected for testability). Returns `None` when no
/// motivational text applies (e.g. past years with unmet goals).
pub fn motivational_pace_text(
    finished: u32,
    target: u32,
    year: i32,
    now: DateTime<Utc>,
) -> Option<String> {
    let remaining = target.saturating_sub(finished);
    let current_year = now.year();

    // Goal reached or exceeded — always celebrate, regardless of year
    if finished > target && target > 0 {
        let exceeded_by = finished - target;
        return Some(format!(
            "You've exceeded your goal by {} {}!",
            exceeded_by,
            pluralize_book(exceeded_by)
        ));
    }

    if finished >= target {
        // Exactly reached (or target is 0)
        return Some("You've reached your goal \u{2014} amazing!".to_string());
    }

    // Past year with unmet goal — no pace advice makes sense
    if year < current_year {
        return None;
    }

    // Future year: use full 12 months
    // Current year: months remaining = 13 - current_month (so January = 12, December = 1)
    let months_left = if year > current_year {
        12
    } else {
        // Current year
        let current_month = now.month();
        13 - current_month
    };

    // December (last month) — special wording
    if months_left == 1 {
        return Some(format!(
            "Just {} more {} this month \u{2014} you can do it!",
            remaining,
            pluralize_book(remaining)
        ));
    }

    // Calculate books per month, rounded up to whole number
    let books_per_month = (remaining as f64 / months_left as f64).ceil() as u32;

    // Determine tone by comparing needed pace to the original yearly pace
    let original_pace_per_month = (target as f64 / 12.0).ceil() as u32;

    let pace_str = format!(
        "{} {} per month",
        books_per_month,
        pluralize_book(books_per_month)
    );

    if books_per_month <= 1 {
        Some(format!(
            "That's about {} \u{2014} smooth sailing!",
            pace_str
        ))
    } else if books_per_month <= original_pace_per_month {
        Some(format!(
            "That's about {} \u{2014} right on track!",
            pace_str
        ))
    } else {
        Some(format!(
            "That's about {} \u{2014} time to pick up the pace!",
            pace_str
        ))
    }
}

/// Returns "book" or "books" depending on the count.
fn pluralize_book(count: u32) -> &'static str {
    if count == 1 {
        "book"
    } else {
        "books"
    }
}
