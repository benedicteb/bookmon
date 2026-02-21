use bookmon::goal::motivational_pace_text;
use chrono::TimeZone;
use chrono::Utc;

// Helper: create a DateTime<Utc> for a given year/month/day.
fn utc(year: i32, month: u32, day: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
}

// ── Goal exceeded ────────────────────────────────────────────────

#[test]
fn test_exceeded_goal_shows_exceeded_message() {
    let now = utc(2026, 6, 15);
    let text = motivational_pace_text(30, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("exceeded") && text.contains("6"),
        "Expected 'exceeded' and '6' in message, got: {}",
        text
    );
}

// ── Goal exactly reached ─────────────────────────────────────────

#[test]
fn test_goal_exactly_reached() {
    let now = utc(2026, 8, 1);
    let text = motivational_pace_text(24, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("reached"),
        "Expected 'reached' in message, got: {}",
        text
    );
}

// ── Past year with unmet goal ────────────────────────────────────

#[test]
fn test_past_year_returns_none() {
    let now = utc(2026, 3, 10);
    // 2025 is in the past, goal was not met
    let text = motivational_pace_text(8, 12, 2025, now);
    assert!(
        text.is_none(),
        "Should return None for a past year with unmet goal"
    );
}

// ── Past year with met goal ──────────────────────────────────────

#[test]
fn test_past_year_goal_met_returns_reached() {
    let now = utc(2026, 3, 10);
    let text = motivational_pace_text(15, 12, 2025, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("exceeded"),
        "Past year with exceeded goal should still celebrate, got: {}",
        text
    );
}

#[test]
fn test_past_year_goal_exactly_met_returns_reached() {
    let now = utc(2026, 3, 10);
    let text = motivational_pace_text(12, 12, 2025, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("reached"),
        "Past year with exactly met goal should celebrate, got: {}",
        text
    );
}

// ── Last month (December) ────────────────────────────────────────

#[test]
fn test_december_shows_this_month_message() {
    let now = utc(2026, 12, 5);
    let text = motivational_pace_text(21, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("this month"),
        "In December should mention 'this month', got: {}",
        text
    );
    assert!(
        text.contains("3"),
        "Should show 3 remaining books, got: {}",
        text
    );
}

// ── Comfortable pace (1 book/month or less) ──────────────────────

#[test]
fn test_comfortable_pace_less_than_one_per_month() {
    // 20 finished, 24 target, in January => 4 remaining over 12 months = 0.33/month rounds up to 1
    let now = utc(2026, 1, 15);
    let text = motivational_pace_text(20, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("1 book per month"),
        "Should say '1 book per month' for comfortable pace, got: {}",
        text
    );
    assert!(
        text.contains("smooth sailing"),
        "Should say 'smooth sailing' for <=1 book/month, got: {}",
        text
    );
}

// ── On track ─────────────────────────────────────────────────────

#[test]
fn test_on_track_pace() {
    // 12 finished out of 24, in July (month 7). Months left = 6. Need 12 more in 6 months = 2/month.
    // Original pace = 24/12 = 2/month. Current need = 2/month. On track.
    let now = utc(2026, 7, 1);
    let text = motivational_pace_text(12, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("2 books per month"),
        "Should show '2 books per month', got: {}",
        text
    );
    assert!(
        text.contains("on track"),
        "Should indicate 'on track', got: {}",
        text
    );
}

// ── Behind pace ──────────────────────────────────────────────────

#[test]
fn test_behind_pace() {
    // 2 finished out of 24, in July (month 7). Months left = 6. Need 22 more in 6 months = 3.67 -> 4/month.
    // Original pace = 24/12 = 2/month. 4 > 2, so behind.
    let now = utc(2026, 7, 1);
    let text = motivational_pace_text(2, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("4 books per month"),
        "Should show '4 books per month', got: {}",
        text
    );
    assert!(
        text.contains("pick up the pace"),
        "Should say 'pick up the pace' when behind, got: {}",
        text
    );
}

// ── Future year ──────────────────────────────────────────────────

#[test]
fn test_future_year_shows_full_year_pace() {
    // Goal for 2027 viewed in 2026. 12 months to go, 0 finished.
    let now = utc(2026, 6, 15);
    let text = motivational_pace_text(0, 24, 2027, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("2 books per month"),
        "Future year should show pace over 12 months, got: {}",
        text
    );
}

// ── Edge: January of current year ────────────────────────────────

#[test]
fn test_january_full_year_ahead() {
    // January 1st, 0 finished, target 12 => 1/month
    let now = utc(2026, 1, 1);
    let text = motivational_pace_text(0, 12, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("1 book per month"),
        "Should show '1 book per month', got: {}",
        text
    );
}

// ── Edge: zero target ────────────────────────────────────────────

#[test]
fn test_zero_target_returns_reached() {
    let now = utc(2026, 6, 15);
    let text = motivational_pace_text(0, 0, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("reached"),
        "Zero target should count as reached, got: {}",
        text
    );
}

// ── Singular book wording ────────────────────────────────────────

#[test]
fn test_singular_book_wording() {
    // 11 finished out of 12, in January => 1 remaining over 12 months rounds up to 1
    let now = utc(2026, 1, 15);
    let text = motivational_pace_text(11, 12, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("1 book per month"),
        "Should use singular 'book' not 'books' when pace is 1, got: {}",
        text
    );
}

#[test]
fn test_plural_books_wording() {
    let now = utc(2026, 1, 15);
    let text = motivational_pace_text(0, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("2 books per month"),
        "Should use plural 'books' when pace > 1, got: {}",
        text
    );
}

// ── December with 1 book left ────────────────────────────────────

#[test]
fn test_december_singular_remaining() {
    let now = utc(2026, 12, 20);
    let text = motivational_pace_text(23, 24, 2026, now);
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(
        text.contains("1 more book this month"),
        "December with 1 left should say '1 more book this month', got: {}",
        text
    );
}
