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
    todo!()
}
