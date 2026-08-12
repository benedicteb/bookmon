use bookmon::pages::pages_credited_by_year;
use bookmon::storage::{Reading, ReadingEvent, ReadingMetadata};
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;

// Helper: a DateTime<Utc> at midday on the given date.
fn at(year: i32, month: u32, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
}

// Helper: a reading event for a single book. Ids are irrelevant to the ledger.
fn event(event: ReadingEvent, page: Option<i32>, created_on: DateTime<Utc>) -> Reading {
    Reading {
        id: format!("reading-{}", created_on.timestamp()),
        created_on,
        book_id: "book-1".to_string(),
        event,
        metadata: ReadingMetadata {
            current_page: page,
            note: None,
        },
    }
}

// Helper: run the ledger over owned readings.
fn credits(readings: &[Reading], total_pages: i32) -> HashMap<i32, u32> {
    let refs: Vec<&Reading> = readings.iter().collect();
    pages_credited_by_year(&refs, total_pages)
}

#[test]
fn test_finish_without_updates_credits_full_page_count() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2026, 1, 5)),
        event(ReadingEvent::Finished, None, at(2026, 2, 10)),
    ];

    assert_eq!(credits(&readings, 300).get(&2026), Some(&300));
}

#[test]
fn test_updates_then_finish_credit_each_segment_once() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2026, 1, 5)),
        event(ReadingEvent::Update, Some(100), at(2026, 1, 12)),
        event(ReadingEvent::Update, Some(250), at(2026, 1, 20)),
        event(ReadingEvent::Finished, None, at(2026, 2, 1)),
    ];

    assert_eq!(
        credits(&readings, 300).get(&2026),
        Some(&300),
        "100 + 150 + the remaining 50 should total the book's length, not more"
    );
}

#[test]
fn test_reread_credits_pages_again() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2025, 3, 1)),
        event(ReadingEvent::Finished, None, at(2025, 4, 1)),
        event(ReadingEvent::Started, None, at(2026, 3, 1)),
        event(ReadingEvent::Finished, None, at(2026, 4, 1)),
    ];

    let c = credits(&readings, 300);
    assert_eq!(c.get(&2025), Some(&300));
    assert_eq!(c.get(&2026), Some(&300));
}

#[test]
fn test_downward_correction_credits_nothing_and_does_not_double_count() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2026, 1, 5)),
        event(ReadingEvent::Update, Some(200), at(2026, 1, 10)),
        // Correcting a typo back down credits nothing...
        event(ReadingEvent::Update, Some(150), at(2026, 1, 11)),
        // ...and pages 150-200 must not be credited a second time here.
        event(ReadingEvent::Update, Some(220), at(2026, 1, 15)),
        event(ReadingEvent::Finished, None, at(2026, 2, 1)),
    ];

    assert_eq!(credits(&readings, 300).get(&2026), Some(&300));
}

#[test]
fn test_update_beyond_total_pages_is_clamped() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2026, 1, 5)),
        event(ReadingEvent::Update, Some(500), at(2026, 1, 10)),
        event(ReadingEvent::Finished, None, at(2026, 2, 1)),
    ];

    assert_eq!(
        credits(&readings, 300).get(&2026),
        Some(&300),
        "An over-reported page must not inflate the year past the book's length"
    );
}

#[test]
fn test_unknown_total_pages_credits_updates_only() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2026, 1, 5)),
        event(ReadingEvent::Update, Some(150), at(2026, 1, 10)),
        event(ReadingEvent::Finished, None, at(2026, 2, 1)),
    ];

    assert_eq!(
        credits(&readings, 0).get(&2026),
        Some(&150),
        "With an unknown page count, only logged progress counts"
    );
}

#[test]
fn test_unknown_total_pages_without_updates_credits_nothing() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2026, 1, 5)),
        event(ReadingEvent::Finished, None, at(2026, 2, 1)),
    ];

    assert!(credits(&readings, 0).is_empty());
}

#[test]
fn test_credit_splits_across_year_boundary() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2025, 12, 1)),
        event(ReadingEvent::Update, Some(120), at(2025, 12, 20)),
        event(ReadingEvent::Finished, None, at(2026, 1, 15)),
    ];

    let c = credits(&readings, 400);
    assert_eq!(c.get(&2025), Some(&120));
    assert_eq!(c.get(&2026), Some(&280));
}

#[test]
fn test_update_without_finish_credits_progress_only() {
    let readings = vec![
        event(ReadingEvent::Started, None, at(2026, 1, 5)),
        event(ReadingEvent::Update, Some(120), at(2026, 1, 20)),
    ];

    assert_eq!(credits(&readings, 400).get(&2026), Some(&120));
}

#[test]
fn test_non_progress_events_are_ignored() {
    let readings = vec![
        event(ReadingEvent::Bought, None, at(2026, 1, 1)),
        event(ReadingEvent::WantToRead, None, at(2026, 1, 2)),
        event(ReadingEvent::Started, None, at(2026, 1, 3)),
        event(ReadingEvent::Update, Some(50), at(2026, 1, 4)),
        event(ReadingEvent::UnmarkedAsWantToRead, None, at(2026, 1, 5)),
        event(ReadingEvent::Update, Some(90), at(2026, 1, 6)),
    ];

    assert_eq!(
        credits(&readings, 200).get(&2026),
        Some(&90),
        "Bought/WantToRead events must not disturb the running page position"
    );
}
