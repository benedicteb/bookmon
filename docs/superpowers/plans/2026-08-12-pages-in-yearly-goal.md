# Pages in the Yearly Goal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the yearly reading goal so it carries a pages target alongside the books target, with pages read counted from the reading event ledger.

**Architecture:** The `goals` map value changes from a bare `u32` to a `Goal { books, pages }` struct whose `Deserialize` also accepts the old bare number, so existing files load unchanged. A new pure module `src/pages.rs` walks a book's reading events chronologically and credits pages to the year of the event that produced them; `Storage` aggregates that across all books. Display formatting lives in `src/pages.rs` as pure functions so it is testable, with `main.rs` only printing the results.

**Tech Stack:** Rust 2021, `serde`/`serde_json` for persistence, `chrono` for timestamps, `clap` for the CLI.

## Global Constraints

- Storage files written before this change must keep loading. A goal stored as a bare number loads as `{ books: N, pages: 0 }`. No read command may rewrite the file.
- Run `cargo fmt` and `cargo clippy` before every commit. Both must be clean.
- Run `cargo test` before every commit. All tests must pass.
- Tests are integration tests in `tests/`, not `#[cfg(test)]` modules in `src/`. Follow the existing layout.
- Do not add dependencies. Everything needed is already in `Cargo.toml`.
- Use the Unicode em dash escape `\u{2014}` in format strings, matching the existing code in `src/main.rs` and `src/goal.rs`.
- `src/goal.rs` is not modified by this plan. The motivational pace text stays books-only.

**Spec:** `docs/superpowers/specs/2026-08-12-pages-in-yearly-goal-design.md`

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `src/storage.rs` | Modify | `Goal` struct + legacy-tolerant deserializer; `goals: HashMap<i32, Goal>`; goal accessors; page aggregation across books. |
| `src/pages.rs` | Create | Pure page ledger over one book's events, plus the two display formatters. No I/O, no `Storage`. |
| `src/lib.rs` | Modify | Register the `pages` module. |
| `src/main.rs` | Modify | `set-goal <books> <pages>` CLI; print the pages lines in `print-goal` and `print-statistics`. |
| `tests/pages_test.rs` | Create | Ledger rules and formatter output. |
| `tests/storage_test.rs` | Modify | Goal struct round-trip, legacy shapes, page aggregation. |
| `docs/adr/0015-pages-in-yearly-goal.md` | Create | Records the decision, superseding part of ADR 0008. |
| `README.md` | Modify | `set-goal` usage lines (129-136) reflect the two required arguments. |

---

### Task 1: Goal carries a books target and a pages target

Changes the stored shape and the `set-goal` command. Nothing displays pages yet — that arrives in Task 5.

**Files:**
- Modify: `src/storage.rs:329-332` (field), `src/storage.rs:656-669` (accessors), plus a new `Goal` type near the other structs
- Modify: `src/main.rs:141-148` (clap `SetGoal`), `src/main.rs:215-220` (handler), `src/main.rs:328` (statistics), `src/main.rs:455-486` (`print_goal_status`)
- Test: `tests/storage_test.rs:2119-2285` (existing goal tests) and new tests appended after them

**Interfaces:**
- Consumes: nothing from earlier tasks
- Produces:
  - `bookmon::storage::Goal` — `#[derive(Debug, Serialize, Clone, Copy, PartialEq)] pub struct Goal { pub books: u32, pub pages: u32 }`
  - `Storage::set_goal(&mut self, year: i32, books: u32, pages: u32)`
  - `Storage::get_goal(&self, year: i32) -> Option<Goal>`
  - `Storage::remove_goal(&mut self, year: i32) -> Option<Goal>`

- [ ] **Step 1: Write the failing tests for the new goal shape**

Replace the existing goal tests in `tests/storage_test.rs`. Find each test below by name and replace its body with this version. Add `Goal` to the `bookmon::storage` import list at the top of the file.

```rust
#[test]
fn test_set_and_get_goal() {
    let mut storage = Storage::new();

    // Set a goal for 2026
    storage.set_goal(2026, 24, 9000);

    // Verify the goal was set
    assert_eq!(
        storage.get_goal(2026),
        Some(Goal {
            books: 24,
            pages: 9000
        })
    );
}

#[test]
fn test_set_goal_overwrites_existing() {
    let mut storage = Storage::new();

    // Set a goal, then change it
    storage.set_goal(2026, 12, 4000);
    assert_eq!(
        storage.get_goal(2026),
        Some(Goal {
            books: 12,
            pages: 4000
        })
    );

    storage.set_goal(2026, 24, 9000);
    assert_eq!(
        storage.get_goal(2026),
        Some(Goal {
            books: 24,
            pages: 9000
        })
    );
}

#[test]
fn test_remove_goal() {
    let mut storage = Storage::new();

    storage.set_goal(2026, 24, 9000);
    assert_eq!(
        storage.get_goal(2026),
        Some(Goal {
            books: 24,
            pages: 9000
        })
    );

    let removed = storage.remove_goal(2026);
    assert_eq!(
        removed,
        Some(Goal {
            books: 24,
            pages: 9000
        })
    );
    assert_eq!(storage.get_goal(2026), None);
}

#[test]
fn test_multiple_year_goals() {
    let mut storage = Storage::new();

    storage.set_goal(2025, 10, 3000);
    storage.set_goal(2026, 24, 9000);
    storage.set_goal(2027, 30, 12000);

    assert_eq!(storage.get_goal(2025).unwrap().books, 10);
    assert_eq!(storage.get_goal(2026).unwrap().books, 24);
    assert_eq!(storage.get_goal(2027).unwrap().pages, 12000);

    storage.remove_goal(2026);
    assert_eq!(storage.get_goal(2025).unwrap().books, 10);
    assert_eq!(storage.get_goal(2026), None);
    assert_eq!(storage.get_goal(2027).unwrap().books, 30);
}

#[test]
fn test_set_goal_zero_stores_zero() {
    let mut storage = Storage::new();

    // Setting a goal of 0 is allowed and stores the value
    storage.set_goal(2026, 0, 0);
    assert_eq!(
        storage.get_goal(2026),
        Some(Goal { books: 0, pages: 0 }),
        "Goal of 0 should be stored as Some(..), not treated as None"
    );
}

#[test]
fn test_goals_round_trip() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap().to_string();

    // Create storage with goals
    let mut storage = Storage::new();
    storage.set_goal(2025, 10, 3000);
    storage.set_goal(2026, 24, 9000);

    // Also add some standard data to ensure goals coexist properly
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let book = Book::new(
        "Test Book".to_string(),
        "123".to_string(),
        category_id,
        author_id,
        200,
    );
    storage.add_book(book);

    // Write to file
    write_storage(&path, &storage).unwrap();

    // Load from file
    let loaded = bookmon::storage::load_storage(&path).unwrap();

    // Verify goals round-tripped correctly
    assert_eq!(
        loaded.get_goal(2025),
        Some(Goal {
            books: 10,
            pages: 3000
        })
    );
    assert_eq!(
        loaded.get_goal(2026),
        Some(Goal {
            books: 24,
            pages: 9000
        })
    );
    assert_eq!(loaded.get_goal(2027), None);

    // Verify other data is also intact
    assert_eq!(loaded.books.len(), 1);
    assert_eq!(loaded.authors.len(), 1);
    assert_eq!(loaded.categories.len(), 1);
}

#[test]
fn test_goals_in_sorted_json() {
    let mut storage = Storage::new();
    storage.set_goal(2026, 24, 9000);
    storage.set_goal(2025, 12, 4000);

    let json_string = storage.to_sorted_json_string().unwrap();
    let value: serde_json::Value = serde_json::from_str(&json_string).unwrap();

    // Verify the goals field appears in the JSON
    assert!(
        value.get("goals").is_some(),
        "Goals should be present in serialized JSON"
    );

    // Each goal serializes as an object with both targets
    let goals = value.get("goals").unwrap().as_object().unwrap();
    let g2025 = goals.get("2025").unwrap();
    assert_eq!(g2025.get("books").unwrap().as_u64(), Some(12));
    assert_eq!(g2025.get("pages").unwrap().as_u64(), Some(4000));
    let g2026 = goals.get("2026").unwrap();
    assert_eq!(g2026.get("books").unwrap().as_u64(), Some(24));
    assert_eq!(g2026.get("pages").unwrap().as_u64(), Some(9000));
}
```

`test_get_goal_returns_none_for_unset_year`, `test_goals_backward_compatibility`, and `test_goals_empty_in_new_storage_json` need no changes — leave them exactly as they are.

Then append these three new tests at the end of the goal test section:

```rust
#[test]
fn test_legacy_bare_number_goal_loads_with_zero_pages() {
    // Goals written before pages existed were stored as a bare number
    let legacy_json = r#"{
        "authors": {},
        "books": {},
        "categories": {},
        "readings": {},
        "reviews": {},
        "goals": { "2026": 30 }
    }"#;

    let storage: Storage = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(
        storage.get_goal(2026),
        Some(Goal {
            books: 30,
            pages: 0
        }),
        "A legacy bare-number goal should load as a books target with no pages target"
    );
}

#[test]
fn test_object_shape_goal_loads_both_targets() {
    let json = r#"{
        "authors": {},
        "books": {},
        "categories": {},
        "readings": {},
        "reviews": {},
        "goals": { "2026": { "books": 30, "pages": 9000 } }
    }"#;

    let storage: Storage = serde_json::from_str(json).unwrap();
    assert_eq!(
        storage.get_goal(2026),
        Some(Goal {
            books: 30,
            pages: 9000
        })
    );
}

#[test]
fn test_legacy_goal_is_written_back_in_object_shape() {
    let legacy_json = r#"{
        "authors": {},
        "books": {},
        "categories": {},
        "readings": {},
        "reviews": {},
        "goals": { "2026": 30 }
    }"#;

    let storage: Storage = serde_json::from_str(legacy_json).unwrap();
    let json_string = storage.to_sorted_json_string().unwrap();
    let value: serde_json::Value = serde_json::from_str(&json_string).unwrap();

    let goal = value.get("goals").unwrap().get("2026").unwrap();
    assert!(
        goal.is_object(),
        "A legacy goal must be saved back in the object shape, got: {}",
        goal
    );
    assert_eq!(goal.get("books").unwrap().as_u64(), Some(30));
    assert_eq!(goal.get("pages").unwrap().as_u64(), Some(0));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test storage_test`
Expected: compile failure — `cannot find type Goal in bookmon::storage`, and `set_goal` called with 3 arguments but takes 2.

- [ ] **Step 3: Add the `Goal` type to `src/storage.rs`**

Place this immediately above the `Storage` struct definition (just before the `/// Persisted as a single JSON file.` doc comment around line 320):

```rust
/// A yearly reading goal: how many books to finish and how many pages to read.
///
/// Serialized as `{"books": 30, "pages": 9000}`. Goals written before pages
/// existed were stored as a bare number; see `Goal`'s `Deserialize` impl.
#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct Goal {
    pub books: u32,
    pub pages: u32,
}

/// The two on-disk shapes a goal can have. Untagged, so serde tries the legacy
/// bare number first and falls back to the current object form.
#[derive(Deserialize)]
#[serde(untagged)]
enum GoalRepr {
    Legacy(u32),
    Full { books: u32, pages: u32 },
}

impl<'de> Deserialize<'de> for Goal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match GoalRepr::deserialize(deserializer)? {
            GoalRepr::Legacy(books) => Goal { books, pages: 0 },
            GoalRepr::Full { books, pages } => Goal { books, pages },
        })
    }
}
```

- [ ] **Step 4: Change the `goals` field and the accessors**

In the `Storage` struct, replace the `goals` field and its doc comment:

```rust
    /// Yearly reading goals: year -> books and pages targets.
    /// Uses `#[serde(default)]` for backward compatibility with existing JSON files.
    #[serde(default)]
    pub goals: HashMap<i32, Goal>,
```

Replace the three accessor methods:

```rust
    /// Sets a yearly reading goal (books to finish and pages to read).
    pub fn set_goal(&mut self, year: i32, books: u32, pages: u32) {
        self.goals.insert(year, Goal { books, pages });
    }

    /// Returns the reading goal for a given year, or None if no goal is set.
    pub fn get_goal(&self, year: i32) -> Option<Goal> {
        self.goals.get(&year).copied()
    }

    /// Removes the reading goal for a given year, returning the previous value if it existed.
    pub fn remove_goal(&mut self, year: i32) -> Option<Goal> {
        self.goals.remove(&year)
    }
```

- [ ] **Step 5: Update the `set-goal` command in `src/main.rs`**

Replace the `SetGoal` variant:

```rust
    /// Set a yearly reading goal (books to finish and pages to read)
    SetGoal {
        /// Number of books to finish
        books: u32,
        /// Number of pages to read
        pages: u32,
        /// Year to set the goal for (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
    },
```

Replace its handler:

```rust
            Commands::SetGoal { books, pages, year } => {
                let year = year.unwrap_or_else(|| chrono::Utc::now().year());
                storage.set_goal(year, *books, *pages);
                storage::write_storage(&settings.storage_file, &storage)?;
                println!(
                    "Reading goal for {}: {} books, {} pages",
                    year, books, pages
                );
            }
```

- [ ] **Step 6: Update the two read sites in `src/main.rs`**

In `Commands::PrintStatistics`, change the goal lookup to read the books target off the struct:

```rust
                            // Show goal progress if a goal is set for this year
                            if let Some(goal) = storage.get_goal(year) {
                                let target = goal.books;
                                let finished = books.len() as u32;
```

The rest of that block is unchanged. In `print_goal_status`, change the match arm binding and update the hint text:

```rust
        Some(goal) => {
            let target = goal.books;
            let finished = storage.get_books_finished_in_year(year).len() as u32;
```

```rust
        None => {
            println!(
                "No reading goal set for {}. Use `bookmon set-goal <books> <pages>` to set one.",
                year
            );
        }
```

`show_goal_status_if_set` needs no change — it only calls `.is_some()`.

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test`
Expected: PASS, all tests.

- [ ] **Step 8: Verify formatting and lints**

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no output from fmt, no warnings from clippy.

- [ ] **Step 9: Commit**

```bash
git add src/storage.rs src/main.rs tests/storage_test.rs
git commit -m "feat: give yearly goals a pages target alongside books"
```

---

### Task 2: The page ledger

A pure function that credits pages to years from one book's reading history. No consumers yet.

**Files:**
- Create: `src/pages.rs`
- Modify: `src/lib.rs` (add `pub mod pages;` between `pub mod lookup`'s siblings, keeping alphabetical order: after `pub mod goal;`)
- Test: `tests/pages_test.rs`

**Interfaces:**
- Consumes: `bookmon::storage::{Reading, ReadingEvent, ReadingMetadata}` (already exist)
- Produces: `bookmon::pages::pages_credited_by_year(readings: &[&Reading], total_pages: i32) -> std::collections::HashMap<i32, u32>` — `readings` must be one book's readings sorted ascending by `created_on`

- [ ] **Step 1: Write the failing tests**

Create `tests/pages_test.rs`:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test pages_test`
Expected: compile failure — `could not find pages in bookmon`.

- [ ] **Step 3: Create `src/pages.rs`**

```rust
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
```

- [ ] **Step 4: Register the module in `src/lib.rs`**

Add `pub mod pages;` after `pub mod goal;`, keeping the list alphabetical:

```rust
pub mod goal;
pub mod pages;
pub mod reading;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --test pages_test`
Expected: PASS, 10 tests.

- [ ] **Step 6: Verify formatting and lints**

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no output from fmt, no warnings from clippy.

- [ ] **Step 7: Commit**

```bash
git add src/pages.rs src/lib.rs tests/pages_test.rs
git commit -m "feat: add page ledger crediting pages read to years"
```

---

### Task 3: Aggregate pages read across all books

**Files:**
- Modify: `src/storage.rs` (add two methods to `impl Storage`, after `remove_goal`)
- Test: `tests/storage_test.rs` (append after the goal tests)

**Interfaces:**
- Consumes: `bookmon::pages::pages_credited_by_year(readings: &[&Reading], total_pages: i32) -> HashMap<i32, u32>` from Task 2
- Produces:
  - `Storage::pages_read_by_year(&self) -> HashMap<i32, u32>`
  - `Storage::pages_read_in_year(&self, year: i32) -> u32`

- [ ] **Step 1: Write the failing tests**

Append to `tests/storage_test.rs`. The file already imports `Book`, `Author`, `Category`, `Reading`, `ReadingEvent`, and `Storage`; add `ReadingMetadata` and `chrono::{TimeZone, Utc}` to the imports if they are not already there.

```rust
// Helper: adds a book with the given page count, returning its id.
fn add_book_with_pages(storage: &mut Storage, title: &str, total_pages: i32) -> String {
    let author = Author::new("Test Author".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);

    let book = Book::new(
        title.to_string(),
        "123".to_string(),
        category_id,
        author_id,
        total_pages,
    );
    let book_id = book.id.clone();
    storage.add_book(book);
    book_id
}

// Helper: adds a reading event on a specific date.
fn add_event_on(
    storage: &mut Storage,
    book_id: &str,
    event: ReadingEvent,
    page: Option<i32>,
    year: i32,
    month: u32,
    day: u32,
) {
    let reading = Reading {
        id: format!("{}-{}-{}-{}", book_id, year, month, day),
        created_on: Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap(),
        book_id: book_id.to_string(),
        event,
        metadata: ReadingMetadata {
            current_page: page,
            note: None,
        },
    };
    storage.add_reading(reading);
}

#[test]
fn test_pages_read_in_year_aggregates_across_books() {
    let mut storage = Storage::new();

    // Book one: finished in 2026 with no progress updates -> full 300 pages
    let first = add_book_with_pages(&mut storage, "First", 300);
    add_event_on(&mut storage, &first, ReadingEvent::Started, None, 2026, 1, 5);
    add_event_on(
        &mut storage,
        &first,
        ReadingEvent::Finished,
        None,
        2026,
        2,
        1,
    );

    // Book two: still in progress at page 120 in 2026
    let second = add_book_with_pages(&mut storage, "Second", 400);
    add_event_on(&mut storage, &second, ReadingEvent::Started, None, 2026, 3, 1);
    add_event_on(
        &mut storage,
        &second,
        ReadingEvent::Update,
        Some(120),
        2026,
        3,
        20,
    );

    // Book three: read entirely in a different year
    let third = add_book_with_pages(&mut storage, "Third", 200);
    add_event_on(&mut storage, &third, ReadingEvent::Started, None, 2025, 5, 1);
    add_event_on(
        &mut storage,
        &third,
        ReadingEvent::Finished,
        None,
        2025,
        6,
        1,
    );

    assert_eq!(storage.pages_read_in_year(2026), 420);
    assert_eq!(storage.pages_read_in_year(2025), 200);
}

#[test]
fn test_pages_read_in_year_returns_zero_for_year_without_readings() {
    let mut storage = Storage::new();
    let book = add_book_with_pages(&mut storage, "Only Book", 300);
    add_event_on(&mut storage, &book, ReadingEvent::Started, None, 2026, 1, 5);
    add_event_on(
        &mut storage,
        &book,
        ReadingEvent::Finished,
        None,
        2026,
        2,
        1,
    );

    assert_eq!(storage.pages_read_in_year(2024), 0);
}

#[test]
fn test_pages_read_by_year_sorts_events_regardless_of_map_order() {
    let mut storage = Storage::new();
    let book = add_book_with_pages(&mut storage, "Out Of Order", 300);

    // Inserted newest-first; the readings HashMap has no inherent ordering, so
    // the aggregation must sort by created_on before walking the events.
    add_event_on(
        &mut storage,
        &book,
        ReadingEvent::Update,
        Some(250),
        2026,
        3,
        1,
    );
    add_event_on(
        &mut storage,
        &book,
        ReadingEvent::Update,
        Some(100),
        2026,
        2,
        1,
    );
    add_event_on(&mut storage, &book, ReadingEvent::Started, None, 2026, 1, 1);

    let by_year = storage.pages_read_by_year();
    assert_eq!(by_year.get(&2026), Some(&250));
}

#[test]
fn test_pages_read_by_year_ignores_readings_for_missing_books() {
    let mut storage = Storage::new();

    // A reading whose book is not in storage cannot be credited any pages
    add_event_on(
        &mut storage,
        "no-such-book",
        ReadingEvent::Finished,
        None,
        2026,
        1,
        1,
    );

    assert!(storage.pages_read_by_year().is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test storage_test`
Expected: compile failure — `no method named pages_read_in_year found for struct Storage`.

- [ ] **Step 3: Add the aggregation methods**

In `src/storage.rs`, add to `impl Storage` immediately after `remove_goal`:

```rust
    /// Total pages credited to each year across every book.
    ///
    /// Groups readings by book, sorts each book's events chronologically, and
    /// folds in that book's per-year credits. Readings whose book is missing
    /// from storage are skipped, since their page count is unknowable.
    pub fn pages_read_by_year(&self) -> HashMap<i32, u32> {
        let mut by_book: HashMap<&str, Vec<&Reading>> = HashMap::new();
        for reading in self.readings.values() {
            by_book
                .entry(reading.book_id.as_str())
                .or_default()
                .push(reading);
        }

        let mut totals: HashMap<i32, u32> = HashMap::new();
        for (book_id, mut readings) in by_book {
            let total_pages = match self.books.get(book_id) {
                Some(book) => book.total_pages,
                None => continue,
            };
            readings.sort_by_key(|r| r.created_on);
            for (year, pages) in crate::pages::pages_credited_by_year(&readings, total_pages) {
                *totals.entry(year).or_insert(0) += pages;
            }
        }
        totals
    }

    /// Total pages read in a single year. Returns 0 for years with no reading.
    pub fn pages_read_in_year(&self, year: i32) -> u32 {
        self.pages_read_by_year().get(&year).copied().unwrap_or(0)
    }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test storage_test`
Expected: PASS.

- [ ] **Step 5: Verify formatting and lints**

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no output from fmt, no warnings from clippy.

- [ ] **Step 6: Commit**

```bash
git add src/storage.rs tests/storage_test.rs
git commit -m "feat: aggregate pages read per year across all books"
```

---

### Task 4: Display formatters for the pages lines

Pure formatting functions, so the output is covered by tests rather than by eyeballing the binary.

**Files:**
- Modify: `src/pages.rs` (append)
- Test: `tests/pages_test.rs` (append)

**Interfaces:**
- Consumes: nothing from earlier tasks
- Produces:
  - `bookmon::pages::format_goal_pages_line(pages_read: u32, target: u32) -> String`
  - `bookmon::pages::format_statistics_pages_line(pages_read: u32, target: u32) -> String`
  - In both, a `target` of 0 means "no pages target set"

- [ ] **Step 1: Write the failing tests**

Append to `tests/pages_test.rs`, and extend the import at the top of the file to:

```rust
use bookmon::pages::{format_goal_pages_line, format_statistics_pages_line, pages_credited_by_year};
```

```rust
#[test]
fn test_goal_pages_line_shows_progress_and_percentage() {
    assert_eq!(
        format_goal_pages_line(4210, 9000),
        "Pages: 4210/9000 (47%)"
    );
}

#[test]
fn test_goal_pages_line_rounds_percentage_to_whole_number() {
    assert_eq!(format_goal_pages_line(1, 3), "Pages: 1/3 (33%)");
    assert_eq!(format_goal_pages_line(2, 3), "Pages: 2/3 (67%)");
}

#[test]
fn test_goal_pages_line_reports_no_target_instead_of_a_bogus_percentage() {
    // A goal saved before pages existed migrates to a target of 0. Showing
    // "4210/0 (100%)" would be nonsense.
    assert_eq!(
        format_goal_pages_line(4210, 0),
        "Pages: no target set \u{2014} use set-goal <books> <pages>"
    );
}

#[test]
fn test_goal_pages_line_handles_exceeded_target() {
    assert_eq!(
        format_goal_pages_line(12000, 9000),
        "Pages: 12000/9000 (133%)"
    );
}

#[test]
fn test_statistics_pages_line_includes_goal_when_set() {
    assert_eq!(
        format_statistics_pages_line(4210, 9000),
        "      4210 pages (Goal: 9000 \u{2014} 47% complete)"
    );
}

#[test]
fn test_statistics_pages_line_omits_goal_clause_without_a_target() {
    assert_eq!(format_statistics_pages_line(4210, 0), "      4210 pages");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test pages_test`
Expected: compile failure — `no format_goal_pages_line in bookmon::pages`.

- [ ] **Step 3: Add the formatters to `src/pages.rs`**

Append to the module:

```rust
/// Shown in place of a percentage when a goal carries no pages target, which
/// happens for goals saved before pages were part of a goal.
const NO_PAGES_TARGET: &str = "Pages: no target set \u{2014} use set-goal <books> <pages>";

/// The pages line under the books block in `print-goal`.
/// A `target` of 0 means no pages target is set.
pub fn format_goal_pages_line(pages_read: u32, target: u32) -> String {
    if target == 0 {
        return NO_PAGES_TARGET.to_string();
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test pages_test`
Expected: PASS, 16 tests.

- [ ] **Step 5: Verify formatting and lints**

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no output from fmt, no warnings from clippy.

- [ ] **Step 6: Commit**

```bash
git add src/pages.rs tests/pages_test.rs
git commit -m "feat: add pages line formatters for goal and statistics output"
```

---

### Task 5: Show pages in `print-goal` and `print-statistics`, and record the ADR

**Files:**
- Modify: `src/main.rs:1-10` (imports), `src/main.rs` `Commands::PrintStatistics` arm, `src/main.rs` `print_goal_status`
- Create: `docs/adr/0015-pages-in-yearly-goal.md`

**Interfaces:**
- Consumes: `Storage::pages_read_by_year`, `Storage::pages_read_in_year` (Task 3); `bookmon::pages::{format_goal_pages_line, format_statistics_pages_line}` (Task 4); `Goal { books, pages }` (Task 1)
- Produces: nothing consumed by later tasks

- [ ] **Step 1: Import the `pages` module in `src/main.rs`**

The `use bookmon::{...}` list at the top currently reads `book, config, goal,` — add `pages` in alphabetical position:

```rust
    book, config, goal, pages,
```

- [ ] **Step 2: Print the pages line in `print_goal_status`**

In `print_goal_status`, after the motivational pace text block and before the final `println!();`:

```rust
            if let Some(motivation) =
                goal::motivational_pace_text(finished, target, year, chrono::Utc::now())
            {
                println!("{}", motivation);
            }
            println!(
                "{}",
                pages::format_goal_pages_line(storage.pages_read_in_year(year), goal.pages)
            );
            println!();
```

- [ ] **Step 3: Print the pages line per year in `print-statistics`**

In the `Commands::PrintStatistics` arm, compute the per-year page totals once before the year loop:

```rust
                } else if let Some(earliest_year) = storage.get_earliest_finished_year() {
                    let current_year = chrono::Utc::now().year();
                    let pages_by_year = storage.pages_read_by_year();
                    println!("\nReading Statistics by Year:");
                    println!("------------------------");
```

Then, inside the loop, add the pages line after the books line — that is, after the closing brace of the `if let Some(goal) = storage.get_goal(year) { ... } else { ... }` block and before the `for book in books` loop:

```rust
                            let pages_read = pages_by_year.get(&year).copied().unwrap_or(0);
                            let pages_target = storage.get_goal(year).map(|g| g.pages).unwrap_or(0);
                            println!(
                                "{}",
                                pages::format_statistics_pages_line(pages_read, pages_target)
                            );
                            for book in books {
```

- [ ] **Step 4: Run the whole test suite**

Run: `cargo test`
Expected: PASS, all tests.

- [ ] **Step 5: Verify the wiring by hand against a sandboxed data file**

The binary reads its config from the OS config directory, which is derived from `$HOME`. Point `$HOME` at a scratch directory so your real data and config are untouched.

```bash
export SANDBOX=$(mktemp -d)
mkdir -p "$SANDBOX/Library/Application Support/bookmon"
cat > "$SANDBOX/Library/Application Support/bookmon/config.yml" <<EOF
storage_file: $SANDBOX/books.json
EOF
cat > "$SANDBOX/books.json" <<'EOF'
{
  "authors": { "a1": { "id": "a1", "name": "Test Author" } },
  "categories": { "c1": { "id": "c1", "name": "Fiction", "description": null } },
  "books": {
    "b1": {
      "id": "b1",
      "title": "Long Book",
      "added_on": "2026-01-01T00:00:00Z",
      "isbn": "111",
      "category_id": "c1",
      "author_id": "a1",
      "total_pages": 500
    },
    "b2": {
      "id": "b2",
      "title": "In Progress",
      "added_on": "2026-01-01T00:00:00Z",
      "isbn": "222",
      "category_id": "c1",
      "author_id": "a1",
      "total_pages": 400
    }
  },
  "readings": {
    "r1": { "id": "r1", "created_on": "2026-01-05T12:00:00Z", "book_id": "b1", "event": "Started" },
    "r2": { "id": "r2", "created_on": "2026-02-05T12:00:00Z", "book_id": "b1", "event": "Finished" },
    "r3": { "id": "r3", "created_on": "2026-03-01T12:00:00Z", "book_id": "b2", "event": "Started" },
    "r4": { "id": "r4", "created_on": "2026-03-20T12:00:00Z", "book_id": "b2", "event": "Update", "metadata": { "current_page": 120 } }
  },
  "reviews": {},
  "goals": { "2026": 30 },
  "series": {}
}
EOF
```

Run each of these with `HOME=$SANDBOX` and confirm the output:

```bash
HOME=$SANDBOX cargo run -- print-goal
```
Expected: the books block as before, then `Pages: no target set — use set-goal <books> <pages>` (the goal in the file is the legacy bare-number shape).

```bash
HOME=$SANDBOX cargo run -- set-goal 30 9000
```
Expected: `Reading goal for 2026: 30 books, 9000 pages`

```bash
HOME=$SANDBOX cargo run -- print-goal
```
Expected: the books block, then `Pages: 620/9000 (7%)` — 500 pages for the finished book plus 120 logged on the in-progress one.

```bash
HOME=$SANDBOX cargo run -- print-statistics
```
Expected: `2026: 1 books (Goal: 30 — 3% complete, 29 remaining)` followed by an indented `      620 pages (Goal: 9000 — 7% complete)`.

```bash
HOME=$SANDBOX cargo run -- set-goal 30
```
Expected: a clap usage error naming the missing `<PAGES>` argument.

Finally confirm the goal was written in the new shape and clean up:

```bash
grep -A 3 '"goals"' "$SANDBOX/books.json"
rm -rf "$SANDBOX"
```
Expected: an object with `"books": 30` and `"pages": 9000`.

- [ ] **Step 6: Write the ADR**

Create `docs/adr/0015-pages-in-yearly-goal.md`:

```markdown
# 0015 - Pages in the Yearly Reading Goal

## Status

Accepted. Supersedes the "Metric: Books finished only" decision in ADR 0008.

## Context

ADR 0008 tracked only books finished, rejecting pages because `total_pages` is
often unknown or zero. Users want the goal to reflect volume as well as count: a
year of doorstoppers reads differently from a year of novellas.

Two questions had to be settled: what a goal contains, and how pages read in a
year are counted.

## Decision

### A goal carries both targets

`Storage.goals` maps a year to `Goal { books, pages }`. Both targets are required
when setting a goal, so `set-goal` takes two positional arguments. This is a
breaking CLI change: `bookmon set-goal 30` now fails with a usage error.

`Goal`'s `Deserialize` accepts both the legacy bare number and the current object
form, so existing files load without migration. A legacy goal becomes
`{ books: N, pages: 0 }` and is written back in the object shape on the next
save. A pages target of 0 renders as "no target set" rather than a bogus 100%.

### Pages are counted from the event ledger

Pages read in a year are computed by walking each book's reading events in order
and crediting the gain from each event to the year that event occurred in, rather
than by summing `total_pages` of the books finished that year. This counts
partial progress and splits a book read across New Year between the two years.

`Finished` credits the remaining pages up to `total_pages`, so users who never log
progress updates still get a meaningful count. `Started` resets the running
position, so a re-read earns its pages again. Progress updates are clamped to
`total_pages`, and a downward correction credits nothing without allowing the
same pages to be counted twice later.

Books with an unknown page count and no logged progress still contribute nothing.
That data-quality hole is unchanged from ADR 0008; the existing `total_pages`
repair prompt is the remedy.

## Consequences

### Easier

- The goal reflects reading volume, not just book count
- Progress on long, unfinished books is visible during the year
- Legacy storage files keep working with no migration step

### More difficult

- `bookmon set-goal 30` no longer works; both targets must be given
- Page counts are only as good as the logged `total_pages` values
- Pages read is derived from the full event history on every call, so it is
  recomputed rather than stored
```

- [ ] **Step 7: Update the README**

`README.md:129-136` documents the old single-argument form. Replace those lines with:

```markdown
- `set-goal <books> <pages>` - Set a yearly reading goal (books to finish and pages to read)
- `print-goal` - Show progress toward your reading goal

​```bash
bookmon set-goal 24 8000
bookmon set-goal 30 9000 --year 2025
bookmon print-goal
bookmon print-goal --year 2025
​```
```

(The zero-width spaces before the inner code fences are an artifact of nesting them in this plan — write plain ``` fences in the README.)

- [ ] **Step 8: Verify formatting and lints one last time**

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: no fmt output, no clippy warnings, all tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/main.rs README.md docs/adr/0015-pages-in-yearly-goal.md
git commit -m "feat: show pages progress in goal and statistics output"
```

---

## Not in this plan

- Pages in the motivational pace text (`src/goal.rs` is untouched)
- Prompting for missing `total_pages` as part of goal tracking
- Monthly or quarterly sub-goals
