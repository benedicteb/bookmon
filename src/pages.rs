use std::collections::HashMap;

use chrono::Datelike;

use crate::storage::{Reading, ReadingEvent};

/// Credits pages to years from one book's reading history.
///
/// `readings` must be that book's readings sorted ascending by `created_on`.
/// `total_pages` is the book's page count; values <= 0 mean "unknown", in which
/// case only explicitly logged progress can be credited.
///
/// Each credit is attributed to the year of the event that produced it, so a
/// book started in December and finished in January splits across both years.
pub fn pages_credited_by_year(readings: &[&Reading], total_pages: i32) -> HashMap<i32, u32> {
    let mut credits: HashMap<i32, u32> = HashMap::new();
    // The furthest page reached so far in the current read-through.
    let mut last_page: i32 = 0;

    for reading in readings {
        let year = reading.created_on.year();
        match reading.event {
            // A re-read starts over and earns its pages again.
            ReadingEvent::Started => last_page = 0,
            ReadingEvent::Update => {
                if let Some(page) = reading.metadata.current_page {
                    // An over-reported page cannot exceed the book's length.
                    let page = if total_pages > 0 {
                        page.min(total_pages)
                    } else {
                        page
                    };
                    credit(&mut credits, year, page - last_page);
                    // A downward correction credits nothing and must not let the
                    // same pages be credited again on the way back up.
                    last_page = last_page.max(page);
                }
            }
            ReadingEvent::Finished => {
                if total_pages > 0 {
                    credit(&mut credits, year, total_pages - last_page);
                }
                last_page = 0;
            }
            ReadingEvent::Bought
            | ReadingEvent::WantToRead
            | ReadingEvent::UnmarkedAsWantToRead => {}
        }
    }

    credits
}

/// Adds `pages` to `year`'s total, ignoring zero and negative amounts.
fn credit(credits: &mut HashMap<i32, u32>, year: i32, pages: i32) {
    if pages > 0 {
        *credits.entry(year).or_insert(0) += pages as u32;
    }
}

/// The pages line under the books block in `print-goal`.
/// A `target` of 0 means no pages target is set.
pub fn format_goal_pages_line(pages_read: u32, target: u32) -> String {
    if target == 0 {
        return format!(
            "Pages: {} read \u{2014} no target set, use set-goal <books> <pages>",
            pages_read
        );
    }
    format!(
        "Pages: {}/{} ({:.0}%)",
        pages_read,
        target,
        percentage(pages_read, target)
    )
}

/// The pages line under a year in `print-statistics`, indented to sit beneath
/// the books line. A `target` of 0 means no pages target is set.
pub fn format_statistics_pages_line(pages_read: u32, target: u32) -> String {
    if target == 0 {
        return format!("      {} pages", pages_read);
    }
    format!(
        "      {} pages (Goal: {} \u{2014} {:.0}% complete)",
        pages_read,
        target,
        percentage(pages_read, target)
    )
}

/// Percentage of `target` reached. Callers guarantee `target` is greater than 0.
fn percentage(pages_read: u32, target: u32) -> f64 {
    (pages_read as f64 / target as f64) * 100.0
}
