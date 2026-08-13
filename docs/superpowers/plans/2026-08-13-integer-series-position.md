# Integer Series Position Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change `Book.position_in_series` from `Option<String>` to `Option<i32>`, migrating non-integer legacy values interactively on load, and add an `edit-series` command for repairing series order afterwards.

**Architecture:** The type flip lands atomically in Task 1 (Rust will not compile a half-changed field type). Tasks 2–4 build the migration bottom-up: pure position-mutation helpers, then a pure scan over raw JSON, then the prompting layer that drives them. Task 5 adds the CLI command that reuses the Task 2 helpers. Task 6 updates the schema and README.

**Tech Stack:** Rust, `serde` / `serde_json`, `inquire` (CLI prompts), `tempfile` (dev). Integration tests live in `tests/` and import through the `bookmon::` crate path.

**Spec:** `docs/superpowers/specs/2026-08-13-integer-series-position-design.md`

## Global Constraints

- Positions are **non-negative integers**. `0` is legal (prequel convention). Negatives are invalid everywhere.
- The migration's "needs fixing" predicate MUST be exactly the deserializer's reject set. If they diverge, a value the migration skips will make the file permanently unloadable.
- **Clippy is not clean in this repo** (~39 pre-existing warnings). The gate is **no new warnings**, never zero. Capture the count before you start: `cargo clippy 2>&1 | grep -c "^warning"`.
- Run `cargo fmt` before every commit.
- Full suite: `cargo test`. Single test: `cargo test <test_name>`.
- Never silently round or discard a user's position value. Every lossy change is a prompt.

---

### Task 1: Flip the field to `Option<i32>`

This task is large because Rust cannot compile a partially-changed field type — the field, the deserializer, the comparator and every call site must move together. Do it as one commit.

**Files:**
- Modify: `src/storage.rs` (field at :74-80, `deserialize_position` at :85-106, `compare_positions` at :740-758, `get_books_in_series` at :424-436)
- Modify: `src/series.rs` (`format_series_label` :10, `format_position_prefix` :22, `parse_position_input` :32, `format_series_display` :120-124, `is_position_occupied` :137)
- Modify: `src/reading.rs` (:47-52, and `format_position_prefix` calls at :158, :295, :417)
- Modify: `src/book.rs` (`select_series` :296-403)
- Modify: `src/main.rs` (:940-975)
- Test: `tests/storage_test.rs`, `tests/series_test.rs`, `tests/reading_test.rs`

**Interfaces:**
- Consumes: nothing (first task).
- Produces:
  - `bookmon::storage::parse_integral_position(raw: &str) -> Option<i32>`
  - `bookmon::storage::compare_positions(a: Option<i32>, b: Option<i32>) -> std::cmp::Ordering`
  - `Book.position_in_series: Option<i32>`
  - `bookmon::series::parse_position_input(input: &str) -> Option<i32>`
  - `bookmon::series::format_position_prefix(position: Option<i32>) -> String`
  - `bookmon::series::format_series_label(series: &Series, position: Option<i32>) -> String`
  - `bookmon::series::is_position_occupied(storage: &Storage, series_id: &str, position: i32) -> Option<String>`

- [ ] **Step 1: Write the failing tests for the parser and comparator**

Add to `tests/storage_test.rs` (the file already has `use bookmon::storage::{...}` — add `parse_integral_position` and `compare_positions` to it):

```rust
#[test]
fn test_parse_integral_position_accepts_non_negative_integers() {
    assert_eq!(parse_integral_position("3"), Some(3));
    assert_eq!(parse_integral_position("0"), Some(0));
    // A whole number written with a decimal point is lossless, so accept it.
    assert_eq!(parse_integral_position("3.0"), Some(3));
}

#[test]
fn test_parse_integral_position_rejects_invalid() {
    assert_eq!(parse_integral_position("2.5"), None, "fractional");
    assert_eq!(parse_integral_position("-1"), None, "negative");
    assert_eq!(parse_integral_position("abc"), None, "non-numeric");
    assert_eq!(parse_integral_position(""), None, "empty");
    assert_eq!(parse_integral_position("inf"), None, "non-finite");
    assert_eq!(parse_integral_position("NaN"), None, "non-finite");
}

#[test]
fn test_compare_positions_orders_numerically_with_none_last() {
    use std::cmp::Ordering;
    assert_eq!(compare_positions(Some(2), Some(10)), Ordering::Less);
    assert_eq!(compare_positions(Some(0), Some(1)), Ordering::Less);
    assert_eq!(compare_positions(Some(3), Some(3)), Ordering::Equal);
    assert_eq!(compare_positions(None, Some(1)), Ordering::Greater);
    assert_eq!(compare_positions(Some(1), None), Ordering::Less);
    assert_eq!(compare_positions(None, None), Ordering::Equal);
}
```

- [ ] **Step 2: Write the failing tests for the deserializer's accept/reject matrix**

Add to `tests/storage_test.rs`. This helper keeps each row to one line:

```rust
fn book_json(position_field: &str) -> String {
    format!(
        r#"{{
            "id": "b1",
            "title": "T",
            "isbn": "1",
            "added_on": "2024-01-01T00:00:00Z",
            "category_id": "c1",
            "author_id": "a1",
            "total_pages": 100,
            "series_id": "s1"
            {}
        }}"#,
        position_field
    )
}

#[test]
fn test_deserialize_position_accepts_integral_shapes() {
    let cases = [
        ("", None),
        (r#", "position_in_series": null"#, None),
        (r#", "position_in_series": """#, None),
        (r#", "position_in_series": 3"#, Some(3)),
        (r#", "position_in_series": "3""#, Some(3)),
        (r#", "position_in_series": 3.0"#, Some(3)),
        (r#", "position_in_series": "3.0""#, Some(3)),
        (r#", "position_in_series": 0"#, Some(0)),
    ];
    for (field, expected) in cases {
        let book: Book = serde_json::from_str(&book_json(field))
            .unwrap_or_else(|e| panic!("should accept {:?}: {}", field, e));
        assert_eq!(book.position_in_series, expected, "for {:?}", field);
    }
}

#[test]
fn test_deserialize_position_rejects_non_integral_shapes() {
    let cases = [
        r#", "position_in_series": 2.5"#,
        r#", "position_in_series": "2.5""#,
        r#", "position_in_series": "Book Three""#,
        r#", "position_in_series": -1"#,
        r#", "position_in_series": "-1""#,
        r#", "position_in_series": true"#,
    ];
    for field in cases {
        let result: Result<Book, _> = serde_json::from_str(&book_json(field));
        assert!(result.is_err(), "should reject {:?}", field);
    }
}

#[test]
fn test_deserialize_position_error_mentions_value_and_migration() {
    let err = serde_json::from_str::<Book>(&book_json(r#", "position_in_series": "2.5""#))
        .unwrap_err()
        .to_string();
    assert!(err.contains("2.5"), "error should name the value: {}", err);
    assert!(err.contains("bookmon"), "error should say how to fix it: {}", err);
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --test storage_test parse_integral_position`
Expected: FAIL to compile — `parse_integral_position` not found in `bookmon::storage`.

- [ ] **Step 4: Implement the parser, deserializer and comparator in `src/storage.rs`**

Replace `deserialize_position` (:85-106) and add the parser above it:

```rust
/// Parses a position value that must be a non-negative integer.
///
/// Accepts `"3"` and the lossless decimal form `"3.0"`. Rejects fractional
/// values, negatives, non-finite values and anything non-numeric — these are
/// exactly the values the interactive migration prompts about, and the two
/// must not disagree.
pub fn parse_integral_position(raw: &str) -> Option<i32> {
    let value: f64 = raw.trim().parse().ok()?;
    if !value.is_finite() || value.fract() != 0.0 || value < 0.0 {
        return None;
    }
    if value > i32::MAX as f64 {
        return None;
    }
    Some(value as i32)
}

/// Custom deserializer for `position_in_series`.
///
/// Absent, `null` and `""` all mean "no position". A JSON number and a numeric
/// string are both accepted when integral — the number form is the pre-`efca510`
/// legacy shape, the string form is the shape written between then and the move
/// to `i32`. Anything else is an error rather than a silent coercion, so that no
/// unmigrated value can reach the program as an altered integer.
fn deserialize_position<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    let raw = match value {
        None | Some(serde_json::Value::Null) => return Ok(None),
        Some(serde_json::Value::String(s)) => {
            if s.trim().is_empty() {
                return Ok(None);
            }
            s
        }
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
    };

    parse_integral_position(&raw).map(Some).ok_or_else(|| {
        serde::de::Error::custom(format!(
            "invalid position_in_series {:?}: must be a non-negative whole number. \
             Run `bookmon` to migrate this file interactively.",
            raw
        ))
    })
}
```

Change the field (:74-80) to:

```rust
    /// Optional position within the series (e.g. 1, 2, or 0 for a prequel).
    /// Non-negative whole numbers only; books without a position sort last.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_position")]
    pub position_in_series: Option<i32>,
```

Replace `compare_positions` (:740-758) with:

```rust
/// Orders two series positions, placing books without a position last.
pub fn compare_positions(a: Option<i32>, b: Option<i32>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a_pos), Some(b_pos)) => a_pos.cmp(&b_pos),
    }
}
```

In `get_books_in_series` (:430-434), drop the `as_deref()` — `Option<i32>` is `Copy`:

```rust
        books.sort_by(|a, b| compare_positions(a.position_in_series, b.position_in_series));
```

Update the doc comment above it (:420-423) to say positions are integers and books without one are placed at the end — the `f64` and lexicographic sentences no longer describe the code.

- [ ] **Step 5: Update `src/series.rs` signatures**

```rust
/// Formats a series label for display, e.g. "Harry Potter #3" or "Harry Potter" (if no position).
pub fn format_series_label(series: &Series, position: Option<i32>) -> String {
    match position {
        Some(pos) => format!("{} #{}", series.name, pos),
        None => series.name.clone(),
    }
}

/// Formats a position prefix for a book title within a grouped series display.
///
/// Returns e.g. `"#3 "` for position 3, or `""` if no position is set.
pub fn format_position_prefix(position: Option<i32>) -> String {
    match position {
        Some(pos) => format!("#{} ", pos),
        None => String::new(),
    }
}

/// Parses a position-in-series input string. Returns `Some(position)` for
/// non-negative whole numbers ("0", "1", "3"). Returns `None` for empty or
/// whitespace input, negatives, fractions, and non-numeric input.
pub fn parse_position_input(input: &str) -> Option<i32> {
    crate::storage::parse_integral_position(input)
}

/// Checks if a position is already occupied by another book in the series.
/// Returns the title of the book at that position, or None if the position is free.
pub fn is_position_occupied(storage: &Storage, series_id: &str, position: i32) -> Option<String> {
    storage
        .books
        .values()
        .find(|b| {
            b.series_id.as_deref() == Some(series_id) && b.position_in_series == Some(position)
        })
        .map(|b| b.title.clone())
}
```

In `format_series_display` (:120-124) drop the `as_deref()`:

```rust
        let pos = book
            .position_in_series
            .map(|p| format!("#{} ", p))
            .unwrap_or_default();
```

- [ ] **Step 6: Update the remaining call sites**

`src/reading.rs` :47-52:

```rust
        group_books.sort_by(|a, b| compare_positions(a.position_in_series, b.position_in_series));
```

`src/reading.rs` :158, :295, :417 — drop `.as_deref()` in each:

```rust
                            format_position_prefix(book.position_in_series),
```

`src/book.rs` — `select_series` return type (:299) becomes `io::Result<(Option<String>, Option<i32>)>`, and its doc comment at :295 stays accurate. Note `book_info.series_position` is `Option<String>` from `BookLookupDTO` (external lookup data) and **stays a String** — it is only used as a text-prompt default at :380, which still takes `&str`. Only the parsed result changes. The occupancy check at :392-400 becomes:

```rust
    if let Some(pos) = position {
        if let Some(existing_title) = crate::series::is_position_occupied(storage, &series_id, pos) {
            println!("Note: '{}' is already #{} in this series.", existing_title, pos);
        }
    }
```

`src/main.rs` :943-970 — same shape, plus the label:

```rust
            if let Some(pos) = position {
                if let Some(existing_title) =
                    bookmon::series::is_position_occupied(&storage, &series_id, pos)
                {
                    if storage.books.get(selected_book_id).map(|b| b.title.as_str())
                        != Some(existing_title.as_str())
                    {
                        println!("Note: '{}' is already #{} in this series.", existing_title, pos);
                    }
                }
            }
```

and:

```rust
            let pos_label = position.map(|p| format!(" #{}", p)).unwrap_or_default();
```

- [ ] **Step 7: Update the existing tests to the new type**

Mechanical across `tests/series_test.rs`, `tests/storage_test.rs`, `tests/reading_test.rs`: every `position_in_series = Some("N".to_string())` becomes `Some(N)`, every `position_in_series: Some("N".to_string())` in struct literals becomes `Some(N)`, and assertions like `assert_eq!(stored_book.position_in_series, Some("1".to_string()))` become `Some(1)`. `position_in_series: None` is unchanged.

Two tests need real edits, not substitution:

Replace `test_get_books_in_series_with_fractional_positions` in `tests/series_test.rs` (:276) — keep its setup, change the positions and the name, since fractional positions no longer exist:

```rust
#[test]
fn test_get_books_in_series_orders_numerically_including_prequel() {
    // Build the same four books as before, but at integer positions:
    // book0 -> 0 (prequel), book1 -> 1, book2 -> 2, book10 -> 10.
    // 10 must sort AFTER 2 — the bug a lexicographic ordering would reintroduce.
    // ...existing storage/author/category setup...

    let books = storage.get_books_in_series(&series_id);
    let positions: Vec<Option<i32>> = books.iter().map(|b| b.position_in_series).collect();
    assert_eq!(positions, vec![Some(0), Some(1), Some(2), Some(10)]);
}
```

Update `test_book_series_fields_backward_compatibility_with_integer_position` (:118) — it now asserts the legacy JSON-number path lands as an integer:

```rust
    assert_eq!(book.position_in_series, Some(3));
```

- [ ] **Step 8: Run the full suite**

Run: `cargo test`
Expected: PASS. Fix any call site the compiler flags — the compiler finds all of them, so a clean build plus green tests means the flip is complete.

- [ ] **Step 9: Check clippy for new warnings**

Run: `cargo clippy 2>&1 | grep -c "^warning"`
Expected: no more than the count you captured before starting.

- [ ] **Step 10: Commit**

```bash
cargo fmt
git add src/ tests/
git commit -m "feat: change position_in_series from String to i32"
```

---

### Task 2: Position mutation helpers

Pure functions over `Storage`, so both the migration and `edit-series` resolve collisions through the same code.

**Files:**
- Modify: `src/series.rs`
- Test: `tests/series_test.rs`

**Interfaces:**
- Consumes: `Book.position_in_series: Option<i32>` (Task 1).
- Produces:
  - `bookmon::series::shift_positions_from(storage: &mut Storage, series_id: &str, from: i32, except: &str)`
  - `bookmon::series::swap_positions(storage: &mut Storage, series_id: &str, book_a: &str, book_b: &str) -> Result<(), String>`

- [ ] **Step 1: Write the failing tests**

Add to `tests/series_test.rs`, importing `shift_positions_from` and `swap_positions`:

```rust
#[test]
fn test_shift_positions_from_moves_only_positions_at_or_after() {
    let mut storage = Storage::new();
    let (series_id, ids) = seed_series_with_positions(&mut storage, "Mistborn", &[1, 2, 3]);

    // Insert at 2: books at 2 and 3 move up, book at 1 stays.
    shift_positions_from(&mut storage, &series_id, 2, "");

    assert_eq!(storage.books[&ids[0]].position_in_series, Some(1));
    assert_eq!(storage.books[&ids[1]].position_in_series, Some(3));
    assert_eq!(storage.books[&ids[2]].position_in_series, Some(4));
}

#[test]
fn test_shift_positions_from_skips_the_excepted_book() {
    let mut storage = Storage::new();
    let (series_id, ids) = seed_series_with_positions(&mut storage, "Mistborn", &[1, 2, 3]);

    // The book just placed at 2 must not shift itself out of its own slot.
    shift_positions_from(&mut storage, &series_id, 2, &ids[1]);

    assert_eq!(storage.books[&ids[1]].position_in_series, Some(2));
    assert_eq!(storage.books[&ids[2]].position_in_series, Some(4));
}

#[test]
fn test_shift_positions_from_leaves_other_series_alone() {
    let mut storage = Storage::new();
    let (series_a, ids_a) = seed_series_with_positions(&mut storage, "Mistborn", &[1, 2]);
    let (_series_b, ids_b) = seed_series_with_positions(&mut storage, "Stormlight", &[1, 2]);

    shift_positions_from(&mut storage, &series_a, 1, "");

    assert_eq!(storage.books[&ids_a[0]].position_in_series, Some(2));
    assert_eq!(storage.books[&ids_b[0]].position_in_series, Some(1));
}

#[test]
fn test_shift_positions_from_ignores_books_without_a_position() {
    let mut storage = Storage::new();
    let (series_id, ids) = seed_series_with_positions(&mut storage, "Mistborn", &[1]);
    let unplaced = add_book_to_series(&mut storage, &series_id, "Unplaced", None);

    shift_positions_from(&mut storage, &series_id, 0, "");

    assert_eq!(storage.books[&ids[0]].position_in_series, Some(2));
    assert_eq!(storage.books[&unplaced].position_in_series, None);
}

#[test]
fn test_swap_positions_exchanges_two_books() {
    let mut storage = Storage::new();
    let (series_id, ids) = seed_series_with_positions(&mut storage, "Mistborn", &[1, 2]);

    swap_positions(&mut storage, &series_id, &ids[0], &ids[1]).unwrap();

    assert_eq!(storage.books[&ids[0]].position_in_series, Some(2));
    assert_eq!(storage.books[&ids[1]].position_in_series, Some(1));
}

#[test]
fn test_swap_positions_errors_when_a_book_is_not_in_the_series() {
    let mut storage = Storage::new();
    let (series_id, ids) = seed_series_with_positions(&mut storage, "Mistborn", &[1, 2]);
    let (_other, other_ids) = seed_series_with_positions(&mut storage, "Stormlight", &[1]);

    let result = swap_positions(&mut storage, &series_id, &ids[0], &other_ids[0]);

    assert!(result.is_err());
    assert_eq!(storage.books[&ids[0]].position_in_series, Some(1), "unchanged on error");
}
```

Add these helpers near the top of `tests/series_test.rs` (the file already builds books this way inline; extracting it keeps the new tests readable):

```rust
/// Creates a series with one book per position and returns (series_id, book_ids in order).
fn seed_series_with_positions(
    storage: &mut Storage,
    series_name: &str,
    positions: &[i32],
) -> (String, Vec<String>) {
    let series_id = get_or_create_series(storage, series_name);
    let ids = positions
        .iter()
        .map(|p| add_book_to_series(storage, &series_id, &format!("Book {}", p), Some(*p)))
        .collect();
    (series_id, ids)
}

/// Adds one book to a series at the given position and returns its id.
fn add_book_to_series(
    storage: &mut Storage,
    series_id: &str,
    title: &str,
    position: Option<i32>,
) -> String {
    let author = Author::new("Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fantasy".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let mut book = Book::new(
        title.to_string(),
        "1234567890".to_string(),
        category_id,
        author_id,
        300,
    );
    book.series_id = Some(series_id.to_string());
    book.position_in_series = position;
    let book_id = book.id.clone();
    storage.add_book(book);
    book_id
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test series_test shift_positions`
Expected: FAIL to compile — `shift_positions_from` not found in `bookmon::series`.

- [ ] **Step 3: Implement the helpers in `src/series.rs`**

```rust
/// Increments the position of every book in `series_id` whose position is `>= from`,
/// making room for a book being inserted at `from`.
///
/// `except` is the id of the book being placed; it is skipped so it is not shifted
/// out of the slot it was just given. Pass `""` when no book is exempt. Books with
/// no position are untouched.
pub fn shift_positions_from(storage: &mut Storage, series_id: &str, from: i32, except: &str) {
    for (book_id, book) in storage.books.iter_mut() {
        if book.series_id.as_deref() != Some(series_id) || book_id == except {
            continue;
        }
        if let Some(pos) = book.position_in_series {
            if pos >= from {
                book.position_in_series = Some(pos + 1);
            }
        }
    }
}

/// Exchanges the positions of two books in the same series.
/// Returns an error if either book is missing or does not belong to `series_id`.
pub fn swap_positions(
    storage: &mut Storage,
    series_id: &str,
    book_a: &str,
    book_b: &str,
) -> Result<(), String> {
    let pos_of = |id: &str| -> Result<Option<i32>, String> {
        let book = storage
            .books
            .get(id)
            .ok_or_else(|| format!("Book {} not found.", id))?;
        if book.series_id.as_deref() != Some(series_id) {
            return Err(format!("Book '{}' is not in this series.", book.title));
        }
        Ok(book.position_in_series)
    };

    // Read both positions before writing either, so a failure leaves storage untouched.
    let a_pos = pos_of(book_a)?;
    let b_pos = pos_of(book_b)?;

    if let Some(book) = storage.books.get_mut(book_a) {
        book.position_in_series = b_pos;
    }
    if let Some(book) = storage.books.get_mut(book_b) {
        book.position_in_series = a_pos;
    }
    Ok(())
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test series_test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add src/series.rs tests/series_test.rs
git commit -m "feat: add shift_positions_from and swap_positions helpers"
```

---

### Task 3: Scan raw JSON for positions needing migration

A pure function over `serde_json::Value`, with no prompting and no IO, so the "which values need fixing" rule is testable on its own.

**Files:**
- Modify: `src/storage.rs`
- Test: `tests/storage_test.rs`

**Interfaces:**
- Consumes: `parse_integral_position` (Task 1).
- Produces:
  - `bookmon::storage::PendingPositionFix { book_id: String, title: String, series_id: Option<String>, series_name: String, raw: String, suggested: Option<i32> }`
  - `bookmon::storage::scan_invalid_positions(value: &serde_json::Value) -> Vec<PendingPositionFix>`
  - `bookmon::storage::taken_positions(value: &serde_json::Value, series_id: &str) -> Vec<i32>`

- [ ] **Step 1: Write the failing tests**

Add to `tests/storage_test.rs`. This builds raw JSON directly, because the whole point is values that cannot deserialize:

```rust
fn storage_json_with_positions(entries: &[(&str, &str, &str)]) -> serde_json::Value {
    // entries: (book_id, series_id_json, position_json) — raw JSON fragments.
    let books: String = entries
        .iter()
        .map(|(id, series, position)| {
            format!(
                r#""{id}": {{
                    "id": "{id}", "title": "Title {id}", "isbn": "1",
                    "added_on": "2024-01-01T00:00:00Z",
                    "category_id": "c1", "author_id": "a1", "total_pages": 100,
                    "series_id": {series}, "position_in_series": {position}
                }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    serde_json::from_str(&format!(
        r#"{{
            "books": {{{books}}},
            "readings": {{}}, "authors": {{}}, "categories": {{}},
            "series": {{ "s1": {{ "id": "s1", "name": "Mistborn" }} }}
        }}"#
    ))
    .unwrap()
}

#[test]
fn test_scan_invalid_positions_finds_only_invalid_values() {
    let value = storage_json_with_positions(&[
        ("b1", r#""s1""#, "1"),        // valid integer — skipped
        ("b2", r#""s1""#, r#""2""#),   // valid integral string — skipped
        ("b3", r#""s1""#, r#""2.5""#), // fractional — found
        ("b4", r#""s1""#, "-1"),       // negative — found
        ("b5", r#""s1""#, r#""Book Three""#), // non-numeric — found
        ("b6", r#""s1""#, "null"),     // absent — skipped
    ]);

    let mut found: Vec<String> = scan_invalid_positions(&value)
        .into_iter()
        .map(|f| f.book_id)
        .collect();
    found.sort();

    assert_eq!(found, vec!["b3", "b4", "b5"]);
}

#[test]
fn test_scan_invalid_positions_suggests_ceiling_clamped_to_zero() {
    let value = storage_json_with_positions(&[
        ("b1", r#""s1""#, r#""2.5""#),
        ("b2", r#""s1""#, r#""0.5""#),
        ("b3", r#""s1""#, "-1"),
        ("b4", r#""s1""#, r#""Book Three""#),
    ]);

    let fixes = scan_invalid_positions(&value);
    let suggestion = |id: &str| fixes.iter().find(|f| f.book_id == id).unwrap().suggested;

    // Ceiling, because 2.5 sorts AFTER book 2 — slot 3 preserves reading order.
    assert_eq!(suggestion("b1"), Some(3));
    assert_eq!(suggestion("b2"), Some(1));
    assert_eq!(suggestion("b3"), Some(0), "negatives clamp to 0");
    assert_eq!(suggestion("b4"), None, "non-numeric has no anchor to suggest from");
}

#[test]
fn test_scan_invalid_positions_marks_books_with_no_usable_series() {
    let value = storage_json_with_positions(&[
        ("b1", "null", r#""2.5""#),       // no series at all
        ("b2", r#""missing""#, r#""2.5""#), // dangling series_id
        ("b3", r#""s1""#, r#""2.5""#),    // real series
    ]);

    let fixes = scan_invalid_positions(&value);
    let series_of = |id: &str| {
        fixes
            .iter()
            .find(|f| f.book_id == id)
            .unwrap()
            .series_id
            .clone()
    };

    // These get cleared silently — handle_missing_fields would discard the answer anyway.
    assert_eq!(series_of("b1"), None);
    assert_eq!(series_of("b2"), None);
    assert_eq!(series_of("b3"), Some("s1".to_string()));
}

#[test]
fn test_scan_invalid_positions_carries_title_and_series_name() {
    let value = storage_json_with_positions(&[("b1", r#""s1""#, r#""2.5""#)]);
    let fixes = scan_invalid_positions(&value);

    assert_eq!(fixes[0].title, "Title b1");
    assert_eq!(fixes[0].series_name, "Mistborn");
    assert_eq!(fixes[0].raw, "2.5", "raw value shown to the user, unquoted");
}

#[test]
fn test_taken_positions_lists_valid_positions_in_the_series() {
    let value = storage_json_with_positions(&[
        ("b1", r#""s1""#, "1"),
        ("b2", r#""s1""#, "3"),
        ("b3", r#""s1""#, r#""2.5""#), // invalid — not "taken"
        ("b4", "null", "9"),           // not in this series — excluded
    ]);

    let mut taken = taken_positions(&value, "s1");
    taken.sort();

    assert_eq!(taken, vec![1, 3]);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test storage_test scan_invalid_positions`
Expected: FAIL to compile — `scan_invalid_positions` not found.

- [ ] **Step 3: Implement the scan in `src/storage.rs`**

```rust
/// One book whose stored position is not a valid non-negative integer.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPositionFix {
    pub book_id: String,
    pub title: String,
    /// `None` when the book has no series, or a `series_id` that does not resolve.
    /// Such a book is cleared silently rather than prompted about, because
    /// `handle_missing_fields` clears the positions of orphaned books anyway.
    pub series_id: Option<String>,
    pub series_name: String,
    /// The offending value as written in the file, shown to the user.
    pub raw: String,
    pub suggested: Option<i32>,
}

/// Renders a JSON position value the way it should be shown to a user:
/// strings without their quotes, everything else as written.
fn raw_position_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Suggests an integer slot for a legacy position.
///
/// Uses the ceiling, clamped at 0: a `2.5` sorts after book 2, so slot 3 (with
/// everything from 3 up shifted by one) preserves the original reading order.
/// Non-numeric values have no anchor, so they get no suggestion.
fn suggest_position(raw: &str) -> Option<i32> {
    let value: f64 = raw.trim().parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    let ceiling = value.ceil().max(0.0);
    if ceiling > i32::MAX as f64 {
        return None;
    }
    Some(ceiling as i32)
}

/// Finds every book whose `position_in_series` is not a valid non-negative integer.
///
/// Operates on raw JSON because these values cannot be deserialized into `Book` —
/// that is the whole reason the migration runs before deserialization.
pub fn scan_invalid_positions(value: &serde_json::Value) -> Vec<PendingPositionFix> {
    let series = value.get("series");
    let books = match value.get("books").and_then(|b| b.as_object()) {
        Some(books) => books,
        None => return Vec::new(),
    };

    let mut fixes = Vec::new();
    for (book_id, book) in books {
        let position = match book.get("position_in_series") {
            None | Some(serde_json::Value::Null) => continue,
            Some(position) => position,
        };

        let raw = raw_position_text(position);
        if raw.trim().is_empty() || parse_integral_position(&raw).is_some() {
            continue;
        }

        // Resolve the series; an absent or dangling id means "clear, don't ask".
        let resolved = book
            .get("series_id")
            .and_then(|s| s.as_str())
            .and_then(|sid| series.and_then(|s| s.get(sid)).map(|s| (sid, s)));

        fixes.push(PendingPositionFix {
            book_id: book_id.clone(),
            title: book
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string(),
            series_id: resolved.map(|(sid, _)| sid.to_string()),
            series_name: resolved
                .and_then(|(_, s)| s.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or_default()
                .to_string(),
            suggested: suggest_position(&raw),
            raw,
        });
    }

    // Group by series, then by suggested slot, so the user is asked about a series'
    // books in reading order rather than in HashMap order.
    fixes.sort_by(|a, b| {
        a.series_name
            .cmp(&b.series_name)
            .then(a.suggested.cmp(&b.suggested))
            .then(a.book_id.cmp(&b.book_id))
    });
    fixes
}

/// Returns the positions already validly occupied in a series, read from raw JSON.
pub fn taken_positions(value: &serde_json::Value, series_id: &str) -> Vec<i32> {
    let books = match value.get("books").and_then(|b| b.as_object()) {
        Some(books) => books,
        None => return Vec::new(),
    };

    books
        .values()
        .filter(|book| book.get("series_id").and_then(|s| s.as_str()) == Some(series_id))
        .filter_map(|book| book.get("position_in_series"))
        .filter_map(|position| parse_integral_position(&raw_position_text(position)))
        .collect()
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test storage_test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add src/storage.rs tests/storage_test.rs
git commit -m "feat: scan raw storage JSON for invalid series positions"
```

---

### Task 4: Interactive migration on load

**Files:**
- Modify: `src/storage.rs` (`RepairPrompter` at :805-813, `load_and_repair_storage` at :969-976)
- Modify: `src/main.rs` (`InquirePrompter` at :14)
- Test: `tests/storage_test.rs` (`TestPrompter` at :10-54)

**Interfaces:**
- Consumes: `scan_invalid_positions`, `taken_positions`, `PendingPositionFix` (Task 3); `shift_positions_from` semantics (Task 2, re-implemented here over JSON).
- Produces:
  - `bookmon::storage::PositionChoice { Insert(i32), Set(i32), Clear }`
  - `RepairPrompter::prompt_series_position(&self, book_title: &str, series_name: &str, old_value: &str, suggested: Option<i32>, taken: &[i32]) -> Result<PositionChoice, Box<dyn std::error::Error>>`
  - `bookmon::storage::migrate_positions(storage_path: &str, prompter: &dyn RepairPrompter) -> Result<bool, Box<dyn std::error::Error>>`

- [ ] **Step 1: Write the failing tests**

Add to `tests/storage_test.rs`. First a prompter that replays scripted answers and records what it was asked:

```rust
use std::cell::RefCell;

/// Replays a scripted answer per prompt and records the (title, suggested, taken)
/// it was called with, so tests can assert on what the user was actually shown.
struct ScriptedPositionPrompter {
    answers: RefCell<Vec<PositionChoice>>,
    seen: RefCell<Vec<(String, Option<i32>, Vec<i32>)>>,
}

impl ScriptedPositionPrompter {
    fn new(answers: Vec<PositionChoice>) -> Self {
        Self {
            answers: RefCell::new(answers.into_iter().rev().collect()),
            seen: RefCell::new(Vec::new()),
        }
    }
}

impl RepairPrompter for ScriptedPositionPrompter {
    fn prompt_author_name(&self, _: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("Author".to_string())
    }
    fn prompt_category_name(&self, _: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("Fantasy".to_string())
    }
    fn prompt_total_pages(&self, _: &str) -> Result<i32, Box<dyn std::error::Error>> {
        Ok(100)
    }
    fn prompt_book_details(&self, _: &str) -> Result<BookRepairInput, Box<dyn std::error::Error>> {
        Ok(BookRepairInput {
            title: "Repaired Book".to_string(),
            isbn: "000".to_string(),
            total_pages: 100,
            author_name: "Author".to_string(),
            category_name: "Fantasy".to_string(),
        })
    }
    fn prompt_series_position(
        &self,
        book_title: &str,
        _series_name: &str,
        _old_value: &str,
        suggested: Option<i32>,
        taken: &[i32],
    ) -> Result<PositionChoice, Box<dyn std::error::Error>> {
        self.seen
            .borrow_mut()
            .push((book_title.to_string(), suggested, taken.to_vec()));
        Ok(self
            .answers
            .borrow_mut()
            .pop()
            .expect("prompted more times than the test scripted"))
    }
}

/// Writes raw JSON to a temp file and returns (dir, path). Keep `dir` alive.
fn write_raw_storage(json: &serde_json::Value) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("storage.json");
    std::fs::write(&path, serde_json::to_string_pretty(json).unwrap()).unwrap();
    (dir, path.to_string_lossy().to_string())
}
```

Then the behaviour tests:

```rust
#[test]
fn test_migrate_positions_insert_shifts_later_books() {
    let value = storage_json_with_positions(&[
        ("b1", r#""s1""#, "1"),
        ("b2", r#""s1""#, "2"),
        ("novella", r#""s1""#, r#""2.5""#),
        ("b3", r#""s1""#, "3"),
    ]);
    let (_dir, path) = write_raw_storage(&value);
    let prompter = ScriptedPositionPrompter::new(vec![PositionChoice::Insert(3)]);

    let migrated = migrate_positions(&path, &prompter).unwrap();
    assert!(migrated);

    let storage = load_storage(&path).unwrap();
    assert_eq!(storage.books["b1"].position_in_series, Some(1));
    assert_eq!(storage.books["b2"].position_in_series, Some(2));
    assert_eq!(storage.books["novella"].position_in_series, Some(3));
    assert_eq!(storage.books["b3"].position_in_series, Some(4), "shifted up");
}

#[test]
fn test_migrate_positions_set_leaves_other_books_alone() {
    let value = storage_json_with_positions(&[
        ("b3", r#""s1""#, "3"),
        ("novella", r#""s1""#, r#""2.5""#),
    ]);
    let (_dir, path) = write_raw_storage(&value);
    let prompter = ScriptedPositionPrompter::new(vec![PositionChoice::Set(3)]);

    migrate_positions(&path, &prompter).unwrap();

    let storage = load_storage(&path).unwrap();
    // Duplicates are permitted — is_position_occupied only ever warned.
    assert_eq!(storage.books["novella"].position_in_series, Some(3));
    assert_eq!(storage.books["b3"].position_in_series, Some(3));
}

#[test]
fn test_migrate_positions_clear_drops_the_position() {
    let value = storage_json_with_positions(&[("novella", r#""s1""#, r#""2.5""#)]);
    let (_dir, path) = write_raw_storage(&value);
    let prompter = ScriptedPositionPrompter::new(vec![PositionChoice::Clear]);

    migrate_positions(&path, &prompter).unwrap();

    let storage = load_storage(&path).unwrap();
    assert_eq!(storage.books["novella"].position_in_series, None);
    assert_eq!(
        storage.books["novella"].series_id,
        Some("s1".to_string()),
        "still in the series, just unnumbered"
    );
}

#[test]
fn test_migrate_positions_clears_orphaned_books_without_prompting() {
    let value = storage_json_with_positions(&[
        ("orphan", r#""missing""#, r#""2.5""#),
        ("no_series", "null", r#""2.5""#),
    ]);
    let (_dir, path) = write_raw_storage(&value);
    let prompter = ScriptedPositionPrompter::new(vec![]); // panics if prompted

    migrate_positions(&path, &prompter).unwrap();

    assert!(prompter.seen.borrow().is_empty(), "must not prompt");
    let storage = load_storage(&path).unwrap();
    assert_eq!(storage.books["orphan"].position_in_series, None);
    assert_eq!(storage.books["no_series"].position_in_series, None);
}

#[test]
fn test_migrate_positions_migrates_legacy_negative() {
    // Integral but invalid: skipped by a "is it fractional" check, and the
    // deserializer would then reject the file forever.
    let value = storage_json_with_positions(&[("b1", r#""s1""#, "-1")]);
    let (_dir, path) = write_raw_storage(&value);
    let prompter = ScriptedPositionPrompter::new(vec![PositionChoice::Set(0)]);

    migrate_positions(&path, &prompter).unwrap();

    assert_eq!(prompter.seen.borrow()[0].1, Some(0), "suggests 0");
    let storage = load_storage(&path).unwrap();
    assert_eq!(storage.books["b1"].position_in_series, Some(0));
}

#[test]
fn test_migrate_positions_shows_currently_taken_positions() {
    let value = storage_json_with_positions(&[
        ("b1", r#""s1""#, "1"),
        ("b3", r#""s1""#, "3"),
        ("novella", r#""s1""#, r#""2.5""#),
    ]);
    let (_dir, path) = write_raw_storage(&value);
    let prompter = ScriptedPositionPrompter::new(vec![PositionChoice::Insert(3)]);

    migrate_positions(&path, &prompter).unwrap();

    let mut taken = prompter.seen.borrow()[0].2.clone();
    taken.sort();
    assert_eq!(taken, vec![1, 3]);
}

#[test]
fn test_migrate_positions_is_a_noop_on_a_clean_file() {
    let value = storage_json_with_positions(&[("b1", r#""s1""#, "1")]);
    let (_dir, path) = write_raw_storage(&value);
    let prompter = ScriptedPositionPrompter::new(vec![]);

    assert!(!migrate_positions(&path, &prompter).unwrap());
    assert!(prompter.seen.borrow().is_empty());
}

#[test]
fn test_migrate_positions_second_run_prompts_nothing() {
    let value = storage_json_with_positions(&[
        ("b2", r#""s1""#, "2"),
        ("novella", r#""s1""#, r#""2.5""#),
    ]);
    let (_dir, path) = write_raw_storage(&value);

    let first = ScriptedPositionPrompter::new(vec![PositionChoice::Insert(3)]);
    assert!(migrate_positions(&path, &first).unwrap());

    let second = ScriptedPositionPrompter::new(vec![]);
    assert!(!migrate_positions(&path, &second).unwrap(), "already migrated");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test storage_test migrate_positions`
Expected: FAIL to compile — `PositionChoice` and `migrate_positions` not found.

- [ ] **Step 3: Add `PositionChoice` and the trait method in `src/storage.rs`**

```rust
/// What to do with a book whose stored position could not be migrated automatically.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionChoice {
    /// Place the book here, shifting occupants at or after this position up by one.
    Insert(i32),
    /// Place the book here, leaving other books alone. May leave two books sharing
    /// a position — permitted, matching `is_position_occupied`, which only warns.
    Set(i32),
    /// Drop the position. The book stays in the series and sorts last.
    Clear,
}
```

Add to the `RepairPrompter` trait (:805-813):

```rust
    /// Asks where a book with an unmigratable position should go.
    ///
    /// `old_value` is the value as written in the file, `suggested` is the
    /// proposed slot (`None` when the old value was non-numeric and offers no
    /// anchor), and `taken` lists positions currently occupied in the series.
    fn prompt_series_position(
        &self,
        book_title: &str,
        series_name: &str,
        old_value: &str,
        suggested: Option<i32>,
        taken: &[i32],
    ) -> Result<PositionChoice, Box<dyn std::error::Error>>;
```

- [ ] **Step 4: Implement `migrate_positions` in `src/storage.rs`**

```rust
/// Interactively migrates any `position_in_series` that is not a valid non-negative
/// integer. Returns `true` if the file was changed.
///
/// Runs on raw JSON and before deserialization, because once the field is `i32` a
/// file holding "2.5" cannot be loaded at all. The file is rewritten after every
/// answer, so an interrupted migration keeps the answers already given and
/// re-running resumes with the rest.
pub fn migrate_positions(
    storage_path: &str,
    prompter: &dyn RepairPrompter,
) -> Result<bool, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let mut value: serde_json::Value = serde_json::from_str(&contents)?;

    let pending = scan_invalid_positions(&value);
    if pending.is_empty() {
        return Ok(false);
    }

    for fix in pending {
        let choice = match &fix.series_id {
            // No usable series: clear without asking. handle_missing_fields would
            // discard any answer we collected here moments later.
            None => PositionChoice::Clear,
            Some(series_id) => {
                // Recompute what is taken each time — an earlier Insert may have
                // shifted books since the scan.
                let taken = taken_positions(&value, series_id);
                prompter.prompt_series_position(
                    &fix.title,
                    &fix.series_name,
                    &fix.raw,
                    fix.suggested,
                    &taken,
                )?
            }
        };

        apply_position_choice(&mut value, &fix.book_id, fix.series_id.as_deref(), choice);
        write_json_value(storage_path, &value)?;
    }

    Ok(true)
}

/// Applies one migration answer to the raw JSON tree.
fn apply_position_choice(
    value: &mut serde_json::Value,
    book_id: &str,
    series_id: Option<&str>,
    choice: PositionChoice,
) {
    // Shift first, while the book being placed still holds its old invalid value —
    // that value never parses as an integer, so it is never shifted by accident.
    if let (PositionChoice::Insert(position), Some(series_id)) = (choice, series_id) {
        shift_json_positions_from(value, series_id, position, book_id);
    }

    let book = match value
        .get_mut("books")
        .and_then(|books| books.get_mut(book_id))
        .and_then(|book| book.as_object_mut())
    {
        Some(book) => book,
        None => return,
    };

    match choice {
        PositionChoice::Clear => {
            // Removed rather than nulled: the field is skip_serializing_if = is_none.
            book.remove("position_in_series");
        }
        PositionChoice::Insert(position) | PositionChoice::Set(position) => {
            book.insert(
                "position_in_series".to_string(),
                serde_json::Value::from(position),
            );
        }
    }
}

/// Raw-JSON counterpart of `series::shift_positions_from`, for use before
/// deserialization. Increments every valid position `>= from` in the series,
/// skipping `except`.
fn shift_json_positions_from(
    value: &mut serde_json::Value,
    series_id: &str,
    from: i32,
    except: &str,
) {
    let books = match value.get_mut("books").and_then(|b| b.as_object_mut()) {
        Some(books) => books,
        None => return,
    };

    for (book_id, book) in books.iter_mut() {
        if book_id == except || book.get("series_id").and_then(|s| s.as_str()) != Some(series_id) {
            continue;
        }
        let current = match book.get("position_in_series") {
            Some(position) => parse_integral_position(&raw_position_text(position)),
            None => None,
        };
        if let (Some(position), Some(book)) = (current, book.as_object_mut()) {
            if position >= from {
                book.insert(
                    "position_in_series".to_string(),
                    serde_json::Value::from(position + 1),
                );
            }
        }
    }
}

/// Writes a raw JSON tree using the same sorted-key, pretty formatting as
/// `write_storage`, so a migrated file is byte-identical to a normally saved one.
fn write_json_value(
    storage_path: &str,
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let sorted = sort_json_value(value.clone());
    fs::write(storage_path, serde_json::to_string_pretty(&sorted)?)?;
    Ok(())
}
```

Wire it into `load_and_repair_storage` (:969-976) so it runs **before** `load_storage`:

```rust
/// Loads storage, migrating legacy series positions and repairing any missing
/// references, using the given prompter.
pub fn load_and_repair_storage(
    storage_path: &str,
    prompter: &dyn RepairPrompter,
) -> Result<Storage, Box<dyn std::error::Error>> {
    // Must precede load_storage: an unmigrated position cannot be deserialized.
    migrate_positions(storage_path, prompter)?;
    let mut storage = load_storage(storage_path)?;
    handle_missing_fields(&mut storage, storage_path, prompter)?;
    Ok(storage)
}
```

- [ ] **Step 5: Implement the prompt in `src/main.rs`**

Add to `impl RepairPrompter for InquirePrompter`. `Select` and `Text` are already imported there; add `use bookmon::storage::PositionChoice;` to the existing `bookmon::storage` import.

```rust
    fn prompt_series_position(
        &self,
        book_title: &str,
        series_name: &str,
        old_value: &str,
        suggested: Option<i32>,
        taken: &[i32],
    ) -> Result<PositionChoice, Box<dyn std::error::Error>> {
        println!(
            "\n'{}' in '{}' has the position \"{}\", which is no longer supported \
             — positions must be whole numbers.",
            book_title, series_name, old_value
        );
        if !taken.is_empty() {
            let mut sorted = taken.to_vec();
            sorted.sort_unstable();
            let list: Vec<String> = sorted.iter().map(|p| p.to_string()).collect();
            println!("Positions already used in this series: {}", list.join(", "));
        }

        let position = match suggested {
            Some(suggested) => Text::new("New position:")
                .with_default(&suggested.to_string())
                .prompt()
                .map_err(|e| format!("Failed to get position: {}", e))?,
            None => Text::new("New position (or Enter to leave it unnumbered):")
                .prompt()
                .map_err(|e| format!("Failed to get position: {}", e))?,
        };

        let position = match bookmon::series::parse_position_input(&position) {
            Some(position) => position,
            None => return Ok(PositionChoice::Clear),
        };

        if !taken.contains(&position) {
            return Ok(PositionChoice::Set(position));
        }

        let options = vec![
            "Insert here (move later books up one)",
            "Put it here anyway (two books share the position)",
            "Leave it unnumbered",
        ];
        let choice = Select::new(
            &format!("Position {} is taken. What should happen?", position),
            options,
        )
        .prompt()
        .map_err(|e| format!("Failed to get choice: {}", e))?;

        Ok(match choice {
            "Insert here (move later books up one)" => PositionChoice::Insert(position),
            "Put it here anyway (two books share the position)" => PositionChoice::Set(position),
            _ => PositionChoice::Clear,
        })
    }
```

- [ ] **Step 6: Add the method to the existing `TestPrompter`**

`tests/storage_test.rs` `TestPrompter` (:10-54) must satisfy the widened trait. It is never expected to be asked about positions:

```rust
    fn prompt_series_position(
        &self,
        _book_title: &str,
        _series_name: &str,
        _old_value: &str,
        _suggested: Option<i32>,
        _taken: &[i32],
    ) -> Result<PositionChoice, Box<dyn std::error::Error>> {
        panic!("TestPrompter should not be asked about series positions")
    }
```

- [ ] **Step 7: Run the full suite**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 8: Check clippy for new warnings**

Run: `cargo clippy 2>&1 | grep -c "^warning"`
Expected: no more than the baseline count.

- [ ] **Step 9: Commit**

```bash
cargo fmt
git add src/ tests/
git commit -m "feat: migrate legacy series positions interactively on load"
```

---

### Task 5: `edit-series` command

**Files:**
- Modify: `src/main.rs` (`Commands` enum at :102-163, the command match arm near :433-438, and a new flow function beside `rename_series_flow` at :632)
- Test: `tests/series_test.rs`

**Interfaces:**
- Consumes: `shift_positions_from`, `swap_positions` (Task 2); `get_books_in_series`, `write_storage`.
- Produces: `EditSeries` variant on `Commands`; `edit_series_flow(storage: &mut Storage, storage_file: &str) -> Result<(), Box<dyn std::error::Error>>` (private to `main.rs`).

The mutation logic is already tested in Task 2. This task adds the CLI shell plus one test that the two helpers compose into the "repair a collapsed series" case.

- [ ] **Step 1: Write the failing test**

Add to `tests/series_test.rs`:

```rust
#[test]
fn test_insert_repairs_a_series_collapsed_by_migration() {
    // After migration, the novella and book 3 both sit at 3.
    let mut storage = Storage::new();
    let series_id = get_or_create_series(&mut storage, "Mistborn");
    let b1 = add_book_to_series(&mut storage, &series_id, "Book 1", Some(1));
    let b2 = add_book_to_series(&mut storage, &series_id, "Book 2", Some(2));
    let novella = add_book_to_series(&mut storage, &series_id, "Novella", Some(3));
    let b3 = add_book_to_series(&mut storage, &series_id, "Book 3", Some(3));

    // Insert the novella at 3, pushing everything else at or after 3 up.
    shift_positions_from(&mut storage, &series_id, 3, &novella);

    assert_eq!(storage.books[&b1].position_in_series, Some(1));
    assert_eq!(storage.books[&b2].position_in_series, Some(2));
    assert_eq!(storage.books[&novella].position_in_series, Some(3));
    assert_eq!(storage.books[&b3].position_in_series, Some(4));

    let ordered: Vec<&str> = storage
        .get_books_in_series(&series_id)
        .iter()
        .map(|b| b.title.as_str())
        .collect();
    assert_eq!(ordered, vec!["Book 1", "Book 2", "Novella", "Book 3"]);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test series_test insert_repairs_a_series`
Expected: FAIL — `Book 3` is still at `Some(3)` if `shift_positions_from` was not applied, or a compile error if `add_book_to_series` was not added in Task 2.

- [ ] **Step 3: Add the `EditSeries` command variant**

In `src/main.rs`, after `RenameSeries` (:162):

```rust
    /// Edit book positions within a series
    EditSeries,
```

And in the command match, beside the `RenameSeries` arm (:436-438):

```rust
            Commands::EditSeries => {
                edit_series_flow(&mut storage, &settings.storage_file)?;
            }
```

- [ ] **Step 4: Implement `edit_series_flow`**

Add after `rename_series_flow` in `src/main.rs`, following its select-then-act shape:

```rust
/// Interactively moves a book to a different position within its series.
fn edit_series_flow(
    storage: &mut Storage,
    storage_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if storage.series.is_empty() {
        println!("No series to edit.");
        return Ok(());
    }

    let mut series_list: Vec<(&String, &storage::Series)> = storage.series.iter().collect();
    series_list.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));
    let names: Vec<&str> = series_list.iter().map(|(_, s)| s.name.as_str()).collect();

    let selection = match Select::new("Select series to edit:", names).prompt() {
        Ok(s) => s,
        Err(_) => {
            println!("Operation cancelled.");
            return Ok(());
        }
    };

    let idx = series_list
        .iter()
        .position(|(_, s)| s.name.as_str() == selection)
        .expect("selection from prompt must exist in series list");
    let series_id = series_list[idx].0.clone();

    // Snapshot id + label before mutating, so the borrow on storage ends here.
    let books: Vec<(String, String)> = storage
        .get_books_in_series(&series_id)
        .iter()
        .map(|b| {
            let position = b
                .position_in_series
                .map(|p| format!("#{}", p))
                .unwrap_or_else(|| "—".to_string());
            (b.id.clone(), format!("{} {}", position, b.title))
        })
        .collect();

    if books.is_empty() {
        println!("'{}' has no books.", selection);
        return Ok(());
    }

    let labels: Vec<&str> = books.iter().map(|(_, label)| label.as_str()).collect();
    let picked = match Select::new("Select book to move:", labels).prompt() {
        Ok(p) => p,
        Err(_) => {
            println!("Operation cancelled.");
            return Ok(());
        }
    };

    let book_id = books
        .iter()
        .find(|(_, label)| label.as_str() == picked)
        .map(|(id, _)| id.clone())
        .expect("selection from prompt must exist in book list");

    let position_input = match Text::new("New position (or Enter to leave it unnumbered):").prompt()
    {
        Ok(p) => p,
        Err(_) => {
            println!("Operation cancelled.");
            return Ok(());
        }
    };

    let position = match bookmon::series::parse_position_input(&position_input) {
        Some(position) => position,
        None => {
            if !position_input.trim().is_empty() {
                println!("Position must be a whole number of 0 or more.");
                return Ok(());
            }
            if let Some(book) = storage.books.get_mut(&book_id) {
                book.position_in_series = None;
            }
            storage::write_storage(storage_file, storage)?;
            println!("Removed the position from '{}'.", picked);
            return Ok(());
        }
    };

    // Find an occupant other than the book being moved.
    let occupant = storage
        .books
        .values()
        .find(|b| {
            b.series_id.as_deref() == Some(series_id.as_str())
                && b.position_in_series == Some(position)
                && b.id != book_id
        })
        .map(|b| (b.id.clone(), b.title.clone()));

    if let Some((occupant_id, occupant_title)) = occupant {
        let options = vec![
            "Insert here (move later books up one)",
            "Swap the two books",
            "Cancel",
        ];
        let choice = match Select::new(
            &format!("#{} is taken by '{}'. What should happen?", position, occupant_title),
            options,
        )
        .prompt()
        {
            Ok(c) => c,
            Err(_) => {
                println!("Operation cancelled.");
                return Ok(());
            }
        };

        match choice {
            "Insert here (move later books up one)" => {
                bookmon::series::shift_positions_from(storage, &series_id, position, &book_id);
                if let Some(book) = storage.books.get_mut(&book_id) {
                    book.position_in_series = Some(position);
                }
            }
            "Swap the two books" => {
                // swap_positions already exchanges the two — do NOT pre-assign
                // `position` to the moved book first, or both books end up on it.
                if let Err(e) =
                    bookmon::series::swap_positions(storage, &series_id, &book_id, &occupant_id)
                {
                    eprintln!("Failed to swap: {}", e);
                    return Ok(());
                }
            }
            _ => {
                println!("Operation cancelled.");
                return Ok(());
            }
        }
    } else if let Some(book) = storage.books.get_mut(&book_id) {
        book.position_in_series = Some(position);
    }

    storage::write_storage(storage_file, storage)?;
    println!("Moved '{}' to #{}.", picked, position);
    Ok(())
}
```

- [ ] **Step 5: Run the tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 6: Verify the command is wired up**

Run: `cargo run -- --help`
Expected: `edit-series  Edit book positions within a series` appears in the command list.

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add src/main.rs tests/series_test.rs
git commit -m "feat: add edit-series command for moving book positions"
```

---

### Task 6: Update the JSON schema and README

**Files:**
- Modify: `docs/storage-schema.json` (`position_in_series` at :155-159)
- Modify: `README.md` (series section, around :165)

- [ ] **Step 1: Update the schema**

Replace the `position_in_series` property:

```json
        "position_in_series": {
          "type": "integer",
          "minimum": 0,
          "description": "Optional non-negative position within the series; 0 marks a prequel. Omitted entirely when absent (never written as null). Books without a position sort after those with one. Legacy files may hold this as a string (\"3\") or a non-integer value (\"2.5\"); integral strings are read as numbers, and non-integer values are migrated interactively on load and then rewritten in this form.",
          "examples": [0, 1, 3]
        }
```

- [ ] **Step 2: Validate the schema is still well-formed JSON**

Run: `python3 -c "import json; json.load(open('docs/storage-schema.json')); print('ok')"`
Expected: `ok`

- [ ] **Step 3: Update the README**

The series section (:165) currently reads "Series information (name and position)". Extend the series documentation to state that positions are whole numbers (`0` for prequels), that a file using the old fractional positions is migrated interactively the first time `bookmon` runs, and that `edit-series` moves a book to a different position with insert-or-swap when the target is taken. Match the surrounding heading style and command-list formatting already used for `print-series`, `delete-series` and `rename-series`.

- [ ] **Step 4: Commit**

```bash
git add docs/storage-schema.json README.md
git commit -m "docs: document integer series positions and edit-series"
```

---

## Verification

After all tasks:

- [ ] `cargo test` — full suite green.
- [ ] `cargo clippy 2>&1 | grep -c "^warning"` — at or below the baseline captured at the start.
- [ ] `cargo fmt --check` — clean.
- [ ] Manual migration check: write a storage file containing `"position_in_series": "2.5"`, run `cargo run`, confirm it prompts with a suggested default of `3`, offers insert-on-collision, and that the file afterwards holds an integer and loads without prompting a second time.
