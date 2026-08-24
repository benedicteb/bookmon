use bookmon::storage::migrate_reviews;
use serde_json::json;
use std::fs;

/// Writes a storage file containing one book and the given raw review objects.
fn write_fixture(reviews: serde_json::Value) -> (tempfile::NamedTempFile, String) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();

    let contents = json!({
        "books": {
            "book-1": {
                "id": "book-1",
                "title": "1984",
                "added_on": "2026-01-01T00:00:00Z",
                "isbn": "978-0451524935",
                "category_id": "cat-1",
                "author_id": "auth-1",
                "total_pages": 328,
                "series_id": null,
                "position_in_series": null
            }
        },
        "authors": {"auth-1": {"id": "auth-1", "name": "George Orwell", "created_on": "2026-01-01T00:00:00Z"}},
        "categories": {"cat-1": {"id": "cat-1", "name": "Fiction", "description": null, "created_on": "2026-01-01T00:00:00Z"}},
        "readings": {},
        "series": {},
        "reviews": reviews
    });

    fs::write(&path, serde_json::to_string_pretty(&contents).unwrap()).unwrap();
    (tmp, path)
}

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn test_single_review_becomes_a_create_review_event() {
    let (_tmp, path) = write_fixture(json!({
        "rev-1": {
            "id": "rev-1",
            "created_on": "2026-03-04T10:00:00Z",
            "book_id": "book-1",
            "text": "A cold, precise book."
        }
    }));

    assert!(migrate_reviews(&path).unwrap());

    let value = read_json(&path);
    assert!(
        value.get("reviews").is_none(),
        "reviews key must be removed"
    );

    let readings = value["readings"].as_object().unwrap();
    assert_eq!(readings.len(), 1);

    let event = &readings["rev-1"];
    assert_eq!(event["event"], "CreateReview");
    assert_eq!(event["book_id"], "book-1");
    assert_eq!(event["created_on"], "2026-03-04T10:00:00Z");
    assert_eq!(event["metadata"]["review_text"], "A cold, precise book.");

    assert!(fs::metadata(format!("{}.pre-review-migration.bak", path)).is_ok());
}

#[test]
fn test_oldest_review_wins_and_later_ones_are_dropped() {
    let (_tmp, path) = write_fixture(json!({
        "rev-new": {
            "id": "rev-new",
            "created_on": "2026-06-01T00:00:00Z",
            "book_id": "book-1",
            "text": "Second thoughts."
        },
        "rev-old": {
            "id": "rev-old",
            "created_on": "2026-03-04T00:00:00Z",
            "book_id": "book-1",
            "text": "First thoughts."
        }
    }));

    assert!(migrate_reviews(&path).unwrap());

    let value = read_json(&path);
    let readings = value["readings"].as_object().unwrap();
    assert_eq!(readings.len(), 1);
    assert_eq!(
        readings["rev-old"]["metadata"]["review_text"],
        "First thoughts."
    );
}

#[test]
fn test_review_for_missing_book_is_dropped() {
    let (_tmp, path) = write_fixture(json!({
        "rev-1": {
            "id": "rev-1",
            "created_on": "2026-03-04T00:00:00Z",
            "book_id": "no-such-book",
            "text": "Orphaned."
        }
    }));

    assert!(migrate_reviews(&path).unwrap());

    let value = read_json(&path);
    assert!(value["readings"].as_object().unwrap().is_empty());
    assert!(value.get("reviews").is_none());
}

#[test]
fn test_absent_reviews_key_is_a_noop() {
    let (_tmp, path) = write_fixture(json!({}));
    let mut value = read_json(&path);
    value.as_object_mut().unwrap().remove("reviews");
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    let before = fs::read_to_string(&path).unwrap();

    assert!(!migrate_reviews(&path).unwrap());

    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    assert!(
        fs::metadata(format!("{}.pre-review-migration.bak", path)).is_err(),
        "no backup for a no-op"
    );
}

#[test]
fn test_empty_reviews_object_is_a_noop() {
    let (_tmp, path) = write_fixture(json!({}));
    assert!(!migrate_reviews(&path).unwrap());
}

#[test]
fn test_migration_is_idempotent() {
    let (_tmp, path) = write_fixture(json!({
        "rev-1": {
            "id": "rev-1",
            "created_on": "2026-03-04T00:00:00Z",
            "book_id": "book-1",
            "text": "Once."
        }
    }));

    assert!(migrate_reviews(&path).unwrap());
    let after_first = fs::read_to_string(&path).unwrap();

    assert!(!migrate_reviews(&path).unwrap());
    assert_eq!(fs::read_to_string(&path).unwrap(), after_first);
}

/// `chrono` serializes `DateTime<Utc>` with `SecondsFormat::AutoSi`, so the
/// number of fractional-second digits varies with the value. Comparing the
/// raw RFC 3339 strings therefore does not agree with true chronological
/// order: "...100500Z" sorts lexically *before* "...100Z" (because '5' <
/// 'Z'), even though .100500s is later than .100s. The migration must
/// compare parsed instants, not strings, or it silently keeps the wrong
/// review as "oldest".
#[test]
fn test_oldest_review_survives_fractional_digit_count_mismatch() {
    let (_tmp, path) = write_fixture(json!({
        "rev-later": {
            "id": "rev-later",
            "created_on": "2026-03-04T10:00:00.100500Z",
            "book_id": "book-1",
            "text": "Chronologically later, but a string-sort would keep it."
        },
        "rev-earlier": {
            "id": "rev-earlier",
            "created_on": "2026-03-04T10:00:00.100Z",
            "book_id": "book-1",
            "text": "Genuinely the oldest review."
        }
    }));

    assert!(migrate_reviews(&path).unwrap());

    let value = read_json(&path);
    let readings = value["readings"].as_object().unwrap();
    assert_eq!(readings.len(), 1);
    assert!(
        readings.contains_key("rev-earlier"),
        "the genuinely oldest review (by instant, not by string) must survive"
    );
    assert_eq!(
        readings["rev-earlier"]["metadata"]["review_text"],
        "Genuinely the oldest review."
    );
}

#[test]
fn test_backup_does_not_clobber_an_existing_one() {
    let (_tmp, path) = write_fixture(json!({
        "rev-1": {
            "id": "rev-1",
            "created_on": "2026-03-04T00:00:00Z",
            "book_id": "book-1",
            "text": "Once."
        }
    }));
    let original = fs::read_to_string(&path).unwrap();

    assert!(migrate_reviews(&path).unwrap());
    let primary_backup = format!("{}.pre-review-migration.bak", path);
    assert_eq!(fs::read_to_string(&primary_backup).unwrap(), original);

    // Simulate the user restoring their old file over the migrated one, then
    // running the migration again.
    fs::write(&path, &original).unwrap();
    assert!(migrate_reviews(&path).unwrap());

    // The first backup must be untouched, and a second, distinctly named
    // backup must exist alongside it — both holding the same original
    // content, since the file was restored before the second run.
    let fallback_backup = format!("{}.pre-review-migration.2.bak", path);
    assert_eq!(
        fs::read_to_string(&primary_backup).unwrap(),
        original,
        "the first backup must not be overwritten"
    );
    assert_eq!(
        fs::read_to_string(&fallback_backup).unwrap(),
        original,
        "the second run must fall back to a distinctly named backup"
    );
}

#[test]
fn test_orphaned_book_with_multiple_reviews_reports_and_drops_all() {
    let (_tmp, path) = write_fixture(json!({
        "rev-1": {
            "id": "rev-1",
            "created_on": "2026-03-04T00:00:00Z",
            "book_id": "no-such-book",
            "text": "First orphan."
        },
        "rev-2": {
            "id": "rev-2",
            "created_on": "2026-06-01T00:00:00Z",
            "book_id": "no-such-book",
            "text": "Second orphan."
        }
    }));

    assert!(migrate_reviews(&path).unwrap());

    let value = read_json(&path);
    assert!(
        value["readings"].as_object().unwrap().is_empty(),
        "both reviews for the missing book must be dropped"
    );
    assert!(value.get("reviews").is_none());
}
