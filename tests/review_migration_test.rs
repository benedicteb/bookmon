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
