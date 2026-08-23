# Review Edit Events Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make each book hold exactly one review that can be edited, where the current text is derived by replaying `CreateReview`/`EditReview` events, so the review can be shown as a timeline of dated diffs.

**Architecture:** Review activity becomes two new variants of the existing `ReadingEvent` enum, stored in the existing `readings` collection so reading and review history share one ordered stream. Each event carries a full snapshot of the review text in `ReadingMetadata::review_text`; the current review is a fold over those snapshots and diffs are computed only at render time. The persisted `Storage.reviews` map is removed and migrated into events before deserialization.

**Tech Stack:** Rust 2021, `serde`/`serde_json`, `chrono`, `uuid`, `inquire`, `similar` (new), `tempfile`.

**Spec:** `docs/superpowers/specs/2026-08-23-review-event-sourcing-design.md`

## Global Constraints

- Toolchain floor is **Rust 1.83**. Any dependency using edition 2024 or MSRV >= 1.85 fails at the manifest-parse stage. See `docs/adr/0006-dependency-version-caps-for-rust-183.md`.
- New dependency must be pinned exactly as: `similar = ">=2.7, <3"` with the comment `# Capped for Rust 1.83 compat, see docs/adr/0006`.
- All timestamps are `chrono::DateTime<Utc>`. All IDs are UUID v4 as `String`.
- Storage JSON is written through `write_storage` / `write_json_value`, which sort keys via `sort_json_value`. Never hand-write the file.
- Optional metadata fields carry `#[serde(default, skip_serializing_if = "Option::is_none")]` so existing events are not rewritten with null keys.
- Output is plain text. Nothing in this project emits ANSI colour.
- Run `cargo fmt` and `cargo clippy` before every commit.
- Tests are integration tests under `tests/`, run with `cargo test`.

---

### Task 1: Classify status-bearing events

Fixes a pre-existing bug: `most_recent_reading_event` returns the latest event unfiltered, while `is_book_started` skips `Update`. The two disagree, so `Started -> Finished -> Update` currently reports a book as not finished. This must land before review events join the enum, or writing a review would un-finish a book.

**Files:**
- Modify: `src/storage.rs:247-258` (enum + doc comment), `src/storage.rs:647-653` (`most_recent_reading_event`)
- Test: `tests/storage_test.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `ReadingEvent::affects_status(self) -> bool`. Task 2 extends its match arms.

- [ ] **Step 1: Write the failing tests**

Append to `tests/storage_test.rs`:

```rust
#[test]
fn test_progress_update_after_finished_keeps_book_finished() {
    let mut storage = Storage::new();
    let book_id = Uuid::new_v4().to_string();

    let mut started = Reading::new(book_id.clone(), ReadingEvent::Started);
    started.created_on = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    storage.readings.insert(started.id.clone(), started);

    let mut finished = Reading::new(book_id.clone(), ReadingEvent::Finished);
    finished.created_on = Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap();
    storage.readings.insert(finished.id.clone(), finished);

    let mut update = Reading::with_metadata(book_id.clone(), ReadingEvent::Update, 120);
    update.created_on = Utc.with_ymd_and_hms(2026, 1, 3, 0, 0, 0).unwrap();
    storage.readings.insert(update.id.clone(), update);

    assert!(
        storage.is_book_finished(&book_id),
        "a progress update must not un-finish a book"
    );
}

#[test]
fn test_bought_remains_status_bearing() {
    let mut storage = Storage::new();
    let category = Category::new("Fiction".to_string(), None);
    let category_id = category.id.clone();
    storage.add_category(category);
    let author = Author::new("Someone".to_string());
    let author_id = author.id.clone();
    storage.add_author(author);

    let book = Book::new(
        "Bought Book".to_string(),
        "111".to_string(),
        category_id,
        author_id,
        100,
    );
    let book_id = book.id.clone();
    storage.add_book(book);

    let bought = Reading::new(book_id.clone(), ReadingEvent::Bought);
    storage.readings.insert(bought.id.clone(), bought);

    let bought_books = storage.get_bought_books();
    assert_eq!(bought_books.len(), 1);
    assert_eq!(bought_books[0].id, book_id);
}

#[test]
fn test_affects_status_classification() {
    assert!(ReadingEvent::Started.affects_status());
    assert!(ReadingEvent::Finished.affects_status());
    assert!(ReadingEvent::WantToRead.affects_status());
    assert!(ReadingEvent::UnmarkedAsWantToRead.affects_status());
    assert!(ReadingEvent::Bought.affects_status());
    assert!(!ReadingEvent::Update.affects_status());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test storage_test progress_update_after_finished affects_status_classification`
Expected: compile error `no method named 'affects_status'`, and once that is stubbed, `test_progress_update_after_finished_keeps_book_finished` FAILS.

- [ ] **Step 3: Add the classification**

In `src/storage.rs`, directly after the `ReadingEvent` enum:

```rust
impl ReadingEvent {
    /// Whether this event participates in determining a book's current status.
    ///
    /// Progress updates say how far the reader got, not whether the book is
    /// started or finished, so they must not displace the last real status
    /// event when the most recent event is looked up.
    pub fn affects_status(self) -> bool {
        match self {
            ReadingEvent::Started
            | ReadingEvent::Finished
            | ReadingEvent::WantToRead
            | ReadingEvent::UnmarkedAsWantToRead
            | ReadingEvent::Bought => true,
            ReadingEvent::Update => false,
        }
    }
}
```

Note the exhaustive match with no wildcard arm. That is deliberate: Task 2 adds two variants and the compiler must force a decision about them.

- [ ] **Step 4: Filter in `most_recent_reading_event`**

Replace the body at `src/storage.rs:647`:

```rust
    /// The most recent status-bearing event for a book.
    ///
    /// Non-status events (progress updates, review activity) are skipped, so
    /// they never displace the book's actual status.
    pub fn most_recent_reading_event(&self, book_id: &str) -> Option<ReadingEvent> {
        self.readings
            .values()
            .filter(|r| r.book_id == book_id)
            .filter(|r| r.event.affects_status())
            .max_by_key(|r| r.created_on)
            .map(|r| r.event)
    }
```

- [ ] **Step 5: Run the full suite**

Run: `cargo test`
Expected: PASS. If a pre-existing test asserted the buggy behaviour (a book being un-finished by an `Update`), that test encoded the bug — update it to the corrected expectation and note the change in the commit body.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy
git add src/storage.rs tests/storage_test.rs
git commit -m "fix: stop progress updates from un-finishing books

most_recent_reading_event returned the latest event unfiltered
while is_book_started skipped Update, so the two disagreed.
Classify events explicitly via ReadingEvent::affects_status.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: Add review event variants and review text metadata

**Files:**
- Modify: `src/storage.rs` (enum, `affects_status`, `is_book_started`, `ReadingMetadata`, `ReadingMetadata::is_empty`, `Reading::with_metadata`, new `Reading::with_review`)
- Test: `tests/storage_test.rs`

**Interfaces:**
- Consumes: `ReadingEvent::affects_status` (Task 1).
- Produces:
  - `ReadingEvent::CreateReview`, `ReadingEvent::EditReview`
  - `ReadingMetadata { current_page: Option<i32>, note: Option<String>, review_text: Option<String> }`
  - `Reading::with_review(book_id: String, event: ReadingEvent, text: String) -> Reading`

- [ ] **Step 1: Write the failing tests**

Append to `tests/storage_test.rs`:

```rust
#[test]
fn test_review_event_does_not_change_book_status() {
    let mut storage = Storage::new();
    let book_id = Uuid::new_v4().to_string();

    let mut finished = Reading::new(book_id.clone(), ReadingEvent::Finished);
    finished.created_on = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    storage.readings.insert(finished.id.clone(), finished);

    let mut review = Reading::with_review(
        book_id.clone(),
        ReadingEvent::CreateReview,
        "Excellent.".to_string(),
    );
    review.created_on = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
    storage.readings.insert(review.id.clone(), review);

    assert!(
        storage.is_book_finished(&book_id),
        "writing a review must not un-finish a book"
    );
    assert!(!storage.is_book_started(&book_id));
    assert!(!ReadingEvent::CreateReview.affects_status());
    assert!(!ReadingEvent::EditReview.affects_status());
}

#[test]
fn test_with_review_sets_text_and_event() {
    let book_id = Uuid::new_v4().to_string();
    let reading = Reading::with_review(
        book_id.clone(),
        ReadingEvent::EditReview,
        "Revised text.".to_string(),
    );

    assert_eq!(reading.book_id, book_id);
    assert_eq!(reading.event, ReadingEvent::EditReview);
    assert_eq!(
        reading.metadata.review_text,
        Some("Revised text.".to_string())
    );
    assert_eq!(reading.metadata.current_page, None);
    assert_eq!(reading.metadata.note, None);
    assert!(!reading.metadata.is_empty());
}

#[test]
fn test_review_text_absent_is_omitted_from_json() {
    let book_id = Uuid::new_v4().to_string();
    let reading = Reading::new(book_id, ReadingEvent::Started);
    let json = serde_json::to_string(&reading).unwrap();

    assert!(!json.contains("review_text"));
    assert!(!json.contains("metadata"));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test storage_test with_review`
Expected: compile error — `no variant named 'CreateReview'`, `no function named 'with_review'`.

- [ ] **Step 3: Extend the enum and its classification**

In `src/storage.rs`, add the variants to `ReadingEvent`:

```rust
pub enum ReadingEvent {
    Finished,
    Started,
    Update,
    Bought,
    WantToRead,
    UnmarkedAsWantToRead,
    /// The first review written for a book. At most one per book.
    CreateReview,
    /// A revision of an existing review. Carries the full revised text.
    EditReview,
}
```

Extend `affects_status`'s false arm:

```rust
            ReadingEvent::Update | ReadingEvent::CreateReview | ReadingEvent::EditReview => false,
```

- [ ] **Step 4: Extend the metadata struct**

```rust
pub struct ReadingMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_page: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The complete review text as of this event, for `CreateReview` and
    /// `EditReview`. A full snapshot, not a patch — see ADR 0016.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_text: Option<String>,
}
```

Update `is_empty`:

```rust
    pub fn is_empty(&self) -> bool {
        self.current_page.is_none() && self.note.is_none() && self.review_text.is_none()
    }
```

`Reading::with_metadata` and `with_progress_note` construct `ReadingMetadata` with named fields, so add `review_text: None` to both. `Reading::new` uses `ReadingMetadata::default()` and needs no change.

- [ ] **Step 5: Add the constructor**

Inside `impl Reading`, after `with_progress_note`:

```rust
    /// Creates a review event carrying the full review text as of this change.
    ///
    /// `event` must be `CreateReview` or `EditReview`.
    pub fn with_review(book_id: String, event: ReadingEvent, text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata {
                current_page: None,
                note: None,
                review_text: Some(text),
            },
        }
    }
```

- [ ] **Step 6: Fix every exhaustive match the compiler flags**

Run: `cargo build`

`is_book_started` (`src/storage.rs:722`) will fail to compile. Add the two variants to its skip list:

```rust
                ReadingEvent::Update
                | ReadingEvent::Bought
                | ReadingEvent::WantToRead
                | ReadingEvent::UnmarkedAsWantToRead
                | ReadingEvent::CreateReview
                | ReadingEvent::EditReview => continue,
```

Fix any other flagged match the same way: review events are never a reading status. Keep repeating `cargo build` until it is clean.

- [ ] **Step 7: Run the full suite**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cargo fmt && cargo clippy
git add src/storage.rs tests/storage_test.rs
git commit -m "feat: add CreateReview and EditReview events

Review activity joins the readings collection so reading and
review history form one ordered stream. Text rides on
ReadingMetadata.review_text as a full snapshot per event.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: Line diff module

Independent of everything else. Wraps the diff crate behind one function so it can be swapped in a single file if the version cap ever becomes untenable.

**Files:**
- Create: `src/diff.rs`
- Modify: `Cargo.toml`, `src/lib.rs`
- Test: `tests/diff_test.rs` (create)

**Interfaces:**
- Consumes: nothing.
- Produces: `bookmon::diff::DiffLine` (`Context(String)`, `Added(String)`, `Removed(String)`) and `bookmon::diff::line_diff(old: &str, new: &str) -> Vec<DiffLine>`. Task 7 renders these.

- [ ] **Step 1: Add the dependency**

In `Cargo.toml`, in `[dependencies]`:

```toml
# Capped for Rust 1.83 compat, see docs/adr/0006
similar = ">=2.7, <3"
```

The cap is mandatory. `similar` 3.x is edition 2024 with MSRV 1.85 and will not build on the project floor. 2.7.0 is edition 2018 with MSRV 1.60 and has no non-optional dependencies.

- [ ] **Step 2: Verify it resolves under the cap**

Run: `cargo build`
Expected: succeeds, and `grep -A2 'name = "similar"' Cargo.lock` shows a `2.x` version.

- [ ] **Step 3: Write the failing tests**

Create `tests/diff_test.rs`:

```rust
use bookmon::diff::{line_diff, DiffLine};

fn rendered(old: &str, new: &str) -> Vec<String> {
    line_diff(old, new)
        .into_iter()
        .map(|line| match line {
            DiffLine::Context(text) => format!("  {}", text),
            DiffLine::Added(text) => format!("+ {}", text),
            DiffLine::Removed(text) => format!("- {}", text),
        })
        .collect()
}

#[test]
fn test_identical_text_yields_only_context() {
    let diff = line_diff("Same line.\nSecond.", "Same line.\nSecond.");
    assert!(diff.iter().all(|l| matches!(l, DiffLine::Context(_))));
}

#[test]
fn test_added_line() {
    assert_eq!(
        rendered("First.", "First.\nSecond."),
        vec!["  First.", "+ Second."]
    );
}

#[test]
fn test_removed_line() {
    assert_eq!(
        rendered("First.\nSecond.", "First."),
        vec!["  First.", "- Second."]
    );
}

#[test]
fn test_changed_line_is_a_removal_and_an_addition() {
    let out = rendered("Orwell is cold.", "Orwell is deliberately cold.");
    assert!(out.contains(&"- Orwell is cold.".to_string()));
    assert!(out.contains(&"+ Orwell is deliberately cold.".to_string()));
}

#[test]
fn test_empty_to_text_is_all_additions() {
    let diff = line_diff("", "A new review.");
    assert!(diff.iter().any(|l| matches!(l, DiffLine::Added(_))));
    assert!(!diff.iter().any(|l| matches!(l, DiffLine::Removed(_))));
}

#[test]
fn test_text_to_empty_is_all_removals() {
    let diff = line_diff("An old review.", "");
    assert!(diff.iter().any(|l| matches!(l, DiffLine::Removed(_))));
    assert!(!diff.iter().any(|l| matches!(l, DiffLine::Added(_))));
}

#[test]
fn test_lines_starting_with_hash_survive() {
    let out = rendered("# Heading", "# Heading\nBody.");
    assert_eq!(out, vec!["  # Heading", "+ Body."]);
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo test --test diff_test`
Expected: compile error `unresolved import 'bookmon::diff'`.

- [ ] **Step 5: Write the module**

Create `src/diff.rs`:

```rust
use similar::{ChangeTag, TextDiff};

/// One line of a rendered diff between two versions of a text.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
}

/// Compares two texts line by line.
///
/// Every line of both inputs is represented in the result, in order — this is
/// a full diff, not a windowed one. Review texts are short enough that context
/// trimming would cost more clarity than it saves.
pub fn line_diff(old: &str, new: &str) -> Vec<DiffLine> {
    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .map(|change| {
            let text = change.value().trim_end_matches('\n').to_string();
            match change.tag() {
                ChangeTag::Equal => DiffLine::Context(text),
                ChangeTag::Insert => DiffLine::Added(text),
                ChangeTag::Delete => DiffLine::Removed(text),
            }
        })
        .collect()
}
```

Register it in `src/lib.rs` alongside the existing `pub mod` declarations, in alphabetical position:

```rust
pub mod diff;
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test --test diff_test`
Expected: PASS, all 7.

If `test_empty_to_text_is_all_additions` fails because `line_diff("", ...)` yields a spurious empty `Context` line, filter a single leading empty-string change when the corresponding input is empty. Do not paper over it in the test.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy
git add Cargo.toml Cargo.lock src/diff.rs src/lib.rs tests/diff_test.rs
git commit -m "feat: add line diff module

Wraps similar behind DiffLine/line_diff so the crate can be
swapped in one file. Capped below 3.0 for Rust 1.83.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 4: Scissors-style editor stripping

`strip_editor_text` drops every line beginning with `#`. Once a review can be re-opened for editing, that silently deletes markdown headings the user wrote, and the diff shows a deletion they never made. Switch to git's scissors convention: the body is kept verbatim, everything from the scissors line down is discarded.

**Files:**
- Modify: `src/editor.rs`, `src/review.rs:19-25` (template), `src/reading.rs:104-109` (template)
- Test: `tests/editor_test.rs` (rewrite), `tests/reading_test.rs:745`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `bookmon::editor::SCISSORS: &str` — the exact marker line.
  - `bookmon::editor::strip_editor_text(text: &str) -> Option<String>` — unchanged signature, new semantics.
  - `bookmon::editor::instruction_block(lines: &[&str]) -> String` — builds a scissors block with each line commented.

- [ ] **Step 1: Rewrite the editor tests**

Replace the whole contents of `tests/editor_test.rs`. The existing tests assert the old `#`-anywhere behaviour and are now wrong by design.

```rust
use bookmon::editor::{instruction_block, strip_editor_text, SCISSORS};

#[test]
fn test_strips_everything_below_the_scissors_line() {
    let input = format!("My review.\n\n{}\n# Instructions here.\n", SCISSORS);
    assert_eq!(strip_editor_text(&input), Some("My review.".to_string()));
}

#[test]
fn test_hash_lines_in_the_body_are_preserved() {
    let input = format!(
        "# Verdict\n\nOrwell is cold.\n\n{}\n# Instructions here.\n",
        SCISSORS
    );
    assert_eq!(
        strip_editor_text(&input),
        Some("# Verdict\n\nOrwell is cold.".to_string())
    );
}

#[test]
fn test_returns_none_when_body_is_empty() {
    let input = format!("\n\n{}\n# Instructions here.\n", SCISSORS);
    assert_eq!(strip_editor_text(&input), None);
}

#[test]
fn test_returns_none_for_whitespace_only_body() {
    let input = format!("  \n  \n{}\n# Instructions.\n", SCISSORS);
    assert_eq!(strip_editor_text(&input), None);
}

#[test]
fn test_text_without_a_scissors_line_is_kept_whole() {
    let input = "A review with no scissors line.\n# Including this.";
    assert_eq!(
        strip_editor_text(input),
        Some("A review with no scissors line.\n# Including this.".to_string())
    );
}

#[test]
fn test_trims_surrounding_but_not_internal_whitespace() {
    let input = format!("\n\nFirst.\n\nSecond.\n\n{}\n# x\n", SCISSORS);
    assert_eq!(
        strip_editor_text(&input),
        Some("First.\n\nSecond.".to_string())
    );
}

#[test]
fn test_instruction_block_comments_every_line_and_leads_with_scissors() {
    let block = instruction_block(&["First instruction.", "Second instruction."]);
    let lines: Vec<&str> = block.lines().collect();

    assert_eq!(lines[0], SCISSORS);
    assert_eq!(lines[1], "# First instruction.");
    assert_eq!(lines[2], "# Second instruction.");
    assert_eq!(strip_editor_text(&block), None);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test editor_test`
Expected: compile error — `SCISSORS` and `instruction_block` do not exist.

- [ ] **Step 3: Rewrite the stripping logic**

Replace `strip_editor_text` in `src/editor.rs` and add the two new items above it:

```rust
/// Marks the start of the instruction block. Everything from this line down is
/// discarded, which lets the body keep lines that begin with `#` — a review or
/// a note may legitimately use markdown headings.
pub const SCISSORS: &str = "# ------------------------ >8 ------------------------";

/// Builds an instruction block: the scissors line followed by each line
/// commented out. Placed at the end of an editor template.
pub fn instruction_block(lines: &[&str]) -> String {
    let mut block = String::from(SCISSORS);
    for line in lines {
        block.push_str("\n# ");
        block.push_str(line);
    }
    block.push('\n');
    block
}

/// Keeps everything above the scissors line, trimmed.
///
/// Returns None if nothing is left, which is how the user aborts: saving an
/// untouched template leaves an empty body.
pub fn strip_editor_text(text: &str) -> Option<String> {
    let body = match text.split_once(SCISSORS) {
        Some((above, _)) => above,
        None => text,
    };

    let trimmed = body.trim().to_string();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
```

- [ ] **Step 4: Update both templates**

In `src/review.rs`, replace the template in `get_review_text_from_editor`. It gains a `current` parameter so the same function serves both writing and editing — the signature change is consumed by Task 7.

```rust
/// Opens the user's default editor for writing or revising a review.
///
/// `current` pre-fills the buffer when revising. Returns None if the body is
/// empty (user aborted).
pub fn get_review_text_from_editor(
    book_title: &str,
    author_name: &str,
    current: Option<&str>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let verb = if current.is_some() { "Edit" } else { "Write" };
    let template = format!(
        "{}\n\n{}",
        current.unwrap_or(""),
        crate::editor::instruction_block(&[
            &format!(
                "{} your review of \"{}\" by {} above.",
                verb, book_title, author_name
            ),
            "Everything below this line is ignored.",
            "An empty review aborts. Unchanged text records no edit.",
        ])
    );
    crate::editor::get_text_from_editor(&template)
}
```

In `src/reading.rs`, replace `progress_note_template`:

```rust
pub fn progress_note_template(book_title: &str, author_name: &str) -> String {
    format!(
        "\n\n{}",
        crate::editor::instruction_block(&[
            &format!(
                "Write a note about your progress in \"{}\" by {} above.",
                book_title, author_name
            ),
            "Everything below this line is ignored.",
            "An empty note aborts the update.",
        ])
    )
}
```

- [ ] **Step 5: Fix the one existing caller**

`cargo build` will flag `get_review_text_from_editor` in `src/main.rs` (two call sites, around lines 1297 and 1403). Pass `None` at both for now; Task 7 replaces them properly.

- [ ] **Step 6: Run the full suite**

Run: `cargo test`
Expected: PASS. `tests/reading_test.rs:745` (`test_untouched_progress_note_template_strips_to_none`) still passes — the new template has an empty body above the scissors line.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy
git add src/editor.rs src/review.rs src/reading.rs src/main.rs tests/editor_test.rs
git commit -m "fix: keep markdown headings in editor input

Stripping every line starting with # deletes headings the user
wrote. Switch to git-style scissors so the body is verbatim.
Applies to reviews and progress notes alike.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 5: Migrate persisted reviews into events

Must run before deserialization. Serde ignores unknown keys, so once Task 6 removes `Storage.reviews`, a stale file would load cleanly and lose every review silently.

**Files:**
- Modify: `src/storage.rs` (new `migrate_reviews`, wire into `load_and_repair_storage:1238-1247`)
- Test: `tests/review_migration_test.rs` (create)

**Interfaces:**
- Consumes: `ReadingEvent::CreateReview`, `ReadingMetadata::review_text` (Task 2); existing `write_json_value` (`src/storage.rs:1229`).
- Produces: `pub fn migrate_reviews(storage_path: &str) -> Result<bool, Box<dyn std::error::Error>>` — `Ok(true)` when the file was rewritten, `Ok(false)` when there was nothing to do.

- [ ] **Step 1: Write the failing tests**

Create `tests/review_migration_test.rs`:

```rust
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
    assert!(value.get("reviews").is_none(), "reviews key must be removed");

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
    assert_eq!(readings["rev-old"]["metadata"]["review_text"], "First thoughts.");
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test review_migration_test`
Expected: compile error `unresolved import 'bookmon::storage::migrate_reviews'`.

- [ ] **Step 3: Implement the migration**

Add to `src/storage.rs`, near `migrate_positions`:

```rust
/// Converts legacy `reviews` entries into `CreateReview` reading events.
///
/// Must run before deserialization: serde ignores unknown keys, so once
/// `Storage` lost its `reviews` field a stale file would load cleanly and drop
/// every review without a word.
///
/// Each book keeps only its oldest review; later ones are reported and
/// discarded (ADR 0016). A backup is written first, since this loses data.
///
/// Returns `Ok(false)` when there was nothing to migrate, having touched
/// nothing.
pub fn migrate_reviews(storage_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(storage_path)?;
    let mut value: serde_json::Value = serde_json::from_str(&contents)?;

    // Absent, null, not an object, or empty: nothing to migrate, and nothing
    // written. An empty `reviews` key left behind is harmless — serde ignores
    // unknown keys — and leaving it makes a second run a true no-op.
    let reviews = match value.get("reviews").and_then(|r| r.as_object()) {
        Some(reviews) if !reviews.is_empty() => reviews.clone(),
        _ => return Ok(false),
    };

    fs::write(format!("{}.pre-review-migration.bak", storage_path), &contents)?;

    // Group by book, oldest first. Sorting by the raw RFC 3339 string is safe:
    // these are all UTC with a fixed shape, so lexical order is chronological.
    let mut by_book: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
        std::collections::BTreeMap::new();
    for review in reviews.values() {
        let book_id = review
            .get("book_id")
            .and_then(|b| b.as_str())
            .unwrap_or_default()
            .to_string();
        by_book.entry(book_id).or_default().push(review.clone());
    }

    let known_books: std::collections::HashSet<String> = value
        .get("books")
        .and_then(|b| b.as_object())
        .map(|books| books.keys().cloned().collect())
        .unwrap_or_default();

    let mut events = serde_json::Map::new();

    for (book_id, mut group) in by_book {
        group.sort_by(|a, b| {
            a.get("created_on")
                .and_then(|c| c.as_str())
                .unwrap_or_default()
                .cmp(b.get("created_on").and_then(|c| c.as_str()).unwrap_or_default())
        });

        let title = value
            .get("books")
            .and_then(|books| books.get(&book_id))
            .and_then(|book| book.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("an unknown book")
            .to_string();

        if !known_books.contains(&book_id) {
            println!(
                "Discarded {} review(s) referencing a book that no longer exists.",
                group.len()
            );
            continue;
        }

        let oldest = group.remove(0);
        if !group.is_empty() {
            println!(
                "Discarded {} later review(s) for \"{}\".",
                group.len(),
                title
            );
        }

        let id = oldest
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or_default()
            .to_string();
        let text = oldest.get("text").and_then(|t| t.as_str()).unwrap_or("");

        events.insert(
            id.clone(),
            serde_json::json!({
                "id": id,
                "created_on": oldest.get("created_on").cloned().unwrap_or(serde_json::Value::Null),
                "book_id": book_id,
                "event": "CreateReview",
                "metadata": { "review_text": text }
            }),
        );
    }

    let root = value.as_object_mut().ok_or("storage root is not an object")?;
    let readings = root
        .entry("readings")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
    if let Some(readings) = readings.as_object_mut() {
        readings.extend(events);
    }
    root.remove("reviews");

    write_json_value(storage_path, &value)?;
    Ok(true)
}
```

- [ ] **Step 4: Wire it into the load path**

In `load_and_repair_storage` (`src/storage.rs:1241`), after `migrate_positions`:

```rust
    // Must precede load_storage: an unmigrated position cannot be deserialized.
    migrate_positions(storage_path, prompter)?;
    // Must also precede it: serde ignores the unknown `reviews` key, so a
    // stale file would otherwise load cleanly and lose every review.
    migrate_reviews(storage_path)?;
    let mut storage = load_storage(storage_path)?;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --test review_migration_test`
Expected: PASS, all 6.

- [ ] **Step 6: Run the full suite**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
cargo fmt && cargo clippy
git add src/storage.rs tests/review_migration_test.rs
git commit -m "feat: migrate stored reviews into CreateReview events

Keeps each book's oldest review and reports what it discards.
Backs the file up first, and runs before deserialization so a
stale reviews key cannot be silently dropped.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 6: Derive Review from events

Swaps the persisted `Review` for a derived one. This is a single atomic change because removing `Storage.reviews` breaks every consumer at once; the compiler drives the work.

**Files:**
- Modify: `src/storage.rs` (`Review`, new `ReviewRevision`, `review_for_book`, `all_reviews`; remove `Storage.reviews`, `add_review`, `get_review`, `get_reviews_for_book`), `src/review.rs` (`store_review`), `src/main.rs` (call sites)
- Test: `tests/review_test.rs`

**Interfaces:**
- Consumes: `Reading::with_review`, `ReadingEvent::CreateReview`/`EditReview` (Task 2).
- Produces:
  - `ReviewRevision { created_on: DateTime<Utc>, event: ReadingEvent, text: String }`
  - `Review { book_id: String, created_on: DateTime<Utc>, updated_on: DateTime<Utc>, text: String, revisions: Vec<ReviewRevision> }`
  - `Storage::review_for_book(&self, book_id: &str) -> Option<Review>`
  - `Storage::all_reviews(&self) -> Vec<Review>`
  - `review::store_review(storage: &mut Storage, book_id: &str, text: String) -> Result<(), String>`

- [ ] **Step 1: Write the failing tests**

Rewrite `tests/review_test.rs`. Keep only the `create_storage_with_book` helper; every existing test in that file either constructs `Review::new`, passes a `Review` to `store_review`, or calls `show_review_detail` with a review id, and all three are gone. Do not try to salvage them — the assertions below cover the same ground against the new model.

Set the imports to:

```rust
use bookmon::review::{show_review_detail, show_reviews, store_review};
use bookmon::storage::{Author, Book, Category, ReadingEvent, Storage};
```

```rust
#[test]
fn test_no_events_means_no_review() {
    let (storage, book_id) = create_storage_with_book();
    assert!(storage.review_for_book(&book_id).is_none());
}

#[test]
fn test_create_only_yields_one_revision() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First take.".to_string()).unwrap();

    let review = storage.review_for_book(&book_id).unwrap();
    assert_eq!(review.text, "First take.");
    assert_eq!(review.book_id, book_id);
    assert_eq!(review.revisions.len(), 1);
    assert_eq!(review.revisions[0].event, ReadingEvent::CreateReview);
    assert_eq!(review.created_on, review.updated_on);
}

#[test]
fn test_edits_fold_to_the_newest_text() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Second.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Third.".to_string()).unwrap();

    let review = storage.review_for_book(&book_id).unwrap();
    assert_eq!(review.text, "Third.");
    assert_eq!(review.revisions.len(), 3);

    // Oldest first.
    let texts: Vec<&str> = review.revisions.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(texts, vec!["First.", "Second.", "Third."]);

    assert_eq!(review.revisions[0].event, ReadingEvent::CreateReview);
    assert_eq!(review.revisions[1].event, ReadingEvent::EditReview);
    assert_eq!(review.revisions[2].event, ReadingEvent::EditReview);
    assert!(review.updated_on >= review.created_on);
}

#[test]
fn test_only_one_create_review_per_book() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Second.".to_string()).unwrap();

    let creates = storage
        .readings
        .values()
        .filter(|r| r.book_id == book_id && r.event == ReadingEvent::CreateReview)
        .count();
    assert_eq!(creates, 1);
}

#[test]
fn test_unchanged_text_records_no_event() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "Same.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Same.".to_string()).unwrap();

    assert_eq!(storage.review_for_book(&book_id).unwrap().revisions.len(), 1);
}

#[test]
fn test_store_review_rejects_unknown_book() {
    let (mut storage, _book_id) = create_storage_with_book();
    let result = store_review(&mut storage, "no-such-book", "Text.".to_string());
    assert!(result.is_err());
}

#[test]
fn test_all_reviews_returns_one_per_reviewed_book() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "First.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Edited.".to_string()).unwrap();

    let all = storage.all_reviews();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].text, "Edited.");
    assert_eq!(all[0].revisions.len(), 2);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test review_test`
Expected: compile error — `store_review` takes the wrong argument types, `review_for_book` does not exist.

- [ ] **Step 3: Replace the Review type**

In `src/storage.rs`, replace the `Review` struct and its `impl` block:

```rust
/// One recorded state of a review, as of a single event.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewRevision {
    pub created_on: DateTime<Utc>,
    pub event: ReadingEvent,
    pub text: String,
}

/// A book's review, derived by replaying its review events.
///
/// Not persisted. A book has at most one, identified by the book itself —
/// there is no review id. Built by `Storage::review_for_book`.
#[derive(Debug, Clone)]
pub struct Review {
    pub book_id: String,
    /// When the review was first written.
    pub created_on: DateTime<Utc>,
    /// When it was last changed. Equal to `created_on` if never edited.
    pub updated_on: DateTime<Utc>,
    /// The current text: the newest snapshot.
    pub text: String,
    /// Every revision, oldest first.
    pub revisions: Vec<ReviewRevision>,
}

impl Review {
    /// How many times the review has been revised since it was written.
    pub fn edit_count(&self) -> usize {
        self.revisions.len().saturating_sub(1)
    }
}
```

`Review` no longer derives `Serialize`/`Deserialize` — it is never written to disk.

- [ ] **Step 4: Replace the storage accessors**

Remove the `reviews: HashMap<String, Review>` field from `Storage` (`src/storage.rs:498`) and the `add_review`, `get_review` and `get_reviews_for_book` methods (`src/storage.rs:604-623`). Add:

```rust
    /// Replays a book's review events into its current review.
    ///
    /// Returns None if the book has never been reviewed.
    pub fn review_for_book(&self, book_id: &str) -> Option<Review> {
        let mut events: Vec<&Reading> = self
            .readings
            .values()
            .filter(|r| r.book_id == book_id)
            .filter(|r| {
                matches!(
                    r.event,
                    ReadingEvent::CreateReview | ReadingEvent::EditReview
                )
            })
            .collect();

        if events.is_empty() {
            return None;
        }

        events.sort_by_key(|r| r.created_on);

        let revisions: Vec<ReviewRevision> = events
            .iter()
            .map(|r| ReviewRevision {
                created_on: r.created_on,
                event: r.event,
                text: r.metadata.review_text.clone().unwrap_or_default(),
            })
            .collect();

        let first = revisions.first()?;
        let last = revisions.last()?;

        Some(Review {
            book_id: book_id.to_string(),
            created_on: first.created_on,
            updated_on: last.created_on,
            text: last.text.clone(),
            revisions,
        })
    }

    /// Every book's review, newest first by creation date.
    ///
    /// Sorted on `created_on` rather than `updated_on` so the listing keeps
    /// its existing order and is sorted by the date column it displays.
    pub fn all_reviews(&self) -> Vec<Review> {
        let book_ids: std::collections::HashSet<&str> = self
            .readings
            .values()
            .filter(|r| {
                matches!(
                    r.event,
                    ReadingEvent::CreateReview | ReadingEvent::EditReview
                )
            })
            .map(|r| r.book_id.as_str())
            .collect();

        let mut reviews: Vec<Review> = book_ids
            .into_iter()
            .filter_map(|book_id| self.review_for_book(book_id))
            .collect();

        reviews.sort_by(|a, b| b.created_on.cmp(&a.created_on));
        reviews
    }
```

- [ ] **Step 5: Rewrite `store_review`**

In `src/review.rs`:

```rust
/// Records a review for a book, creating it or revising the existing one.
///
/// The first review becomes a `CreateReview` event; every later one becomes an
/// `EditReview`. Text identical to the current version records nothing, so the
/// timeline never shows an empty diff.
pub fn store_review(storage: &mut Storage, book_id: &str, text: String) -> Result<(), String> {
    if !storage.books.contains_key(book_id) {
        return Err(format!("Book with ID {} does not exist", book_id));
    }

    let event = match storage.review_for_book(book_id) {
        Some(existing) if existing.text == text => return Ok(()),
        Some(_) => ReadingEvent::EditReview,
        None => ReadingEvent::CreateReview,
    };

    let reading = Reading::with_review(book_id.to_string(), event, text);
    storage.readings.insert(reading.id.clone(), reading);
    Ok(())
}
```

Update the imports at the top of `src/review.rs` to `use crate::storage::{Reading, ReadingEvent, Review, Storage};`.

- [ ] **Step 6: Fix the remaining compile errors**

Run: `cargo build`

Expected breakages and their fixes:

- `src/review.rs` `show_reviews`: replace `storage.reviews.values().collect()` and the manual sort with `let reviews = storage.all_reviews();`. Each `review.book_id` still resolves the book the same way.
- `src/review.rs` `show_review_detail`: change the signature to `pub fn show_review_detail(storage: &Storage, book_id: &str) -> io::Result<()>` and open with `let review = storage.review_for_book(book_id).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Review not found"))?;`. Task 7 adds the history section.
- `src/main.rs:1299` and `src/main.rs:1405`: replace `storage::Review::new(...)` plus `store_review(&mut storage, review_obj)` with `review::store_review(&mut storage, &book_id, text)`.
- `src/main.rs:1426-1473` `review_interactive_mode`: build the list from `storage.all_reviews()` and key `display_to_id` on `book_id`, passing it to `show_review_detail`.

- [ ] **Step 7: Run the full suite**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cargo fmt && cargo clippy
git add src/storage.rs src/review.rs src/main.rs tests/review_test.rs
git commit -m "feat: derive a book's review from its events

One review per book, identified by the book. Storage.reviews is
removed; the current text is the newest snapshot in the event
stream, and a second write records an EditReview.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 7: Render the review timeline

**Files:**
- Modify: `src/review.rs` (`show_review_detail`, `show_reviews`), `src/main.rs` (menu wording, edit pre-fill)
- Test: `tests/review_test.rs`

**Interfaces:**
- Consumes: `Review`, `ReviewRevision`, `Review::edit_count` (Task 6); `line_diff`, `DiffLine` (Task 3); `get_review_text_from_editor(title, author, current)` (Task 4).
- Produces: `review::format_review_detail(storage: &Storage, book_id: &str) -> Option<String>` — the rendered detail view, returned rather than printed so it can be tested. `show_review_detail` prints it.

- [ ] **Step 1: Write the failing tests**

Append to `tests/review_test.rs`:

```rust
use bookmon::review::format_review_detail;

#[test]
fn test_detail_without_edits_has_no_last_edited_line() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "A cold book.".to_string()).unwrap();

    let out = format_review_detail(&storage, &book_id).unwrap();
    assert!(out.contains("Review of \"1984\" by George Orwell"));
    assert!(out.contains("A cold book."));
    assert!(!out.contains("Last edited"));
    assert!(!out.contains("History"));
}

#[test]
fn test_detail_with_edits_shows_diff_and_dates() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "Orwell is cold.".to_string()).unwrap();
    store_review(
        &mut storage,
        &book_id,
        "Orwell is deliberately cold.".to_string(),
    )
    .unwrap();

    let out = format_review_detail(&storage, &book_id).unwrap();

    // Current text is the newest version.
    assert!(out.contains("Orwell is deliberately cold."));
    assert!(out.contains("Last edited on"));
    assert!(out.contains("(1 edit)"));
    assert!(out.contains("History"));
    assert!(out.contains("- Orwell is cold."));
    assert!(out.contains("+ Orwell is deliberately cold."));
    assert!(out.contains("Written on"));
}

#[test]
fn test_detail_pluralises_edit_count() {
    let (mut storage, book_id) = create_storage_with_book();
    store_review(&mut storage, &book_id, "One.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Two.".to_string()).unwrap();
    store_review(&mut storage, &book_id, "Three.".to_string()).unwrap();

    let out = format_review_detail(&storage, &book_id).unwrap();
    assert!(out.contains("(2 edits)"));
}

#[test]
fn test_detail_for_unreviewed_book_is_none() {
    let (storage, book_id) = create_storage_with_book();
    assert!(format_review_detail(&storage, &book_id).is_none());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test review_test format_review_detail`
Expected: compile error `unresolved import 'bookmon::review::format_review_detail'`.

- [ ] **Step 3: Implement the renderer**

In `src/review.rs`:

```rust
use crate::diff::{line_diff, DiffLine};

/// Renders the full review detail view: current text, then the history.
///
/// Returns None if the book has no review. Returned rather than printed so the
/// layout can be tested without capturing stdout.
pub fn format_review_detail(storage: &Storage, book_id: &str) -> Option<String> {
    let review = storage.review_for_book(book_id)?;
    let book = storage.books.get(book_id);
    let title = book.map(|b| b.title.as_str()).unwrap_or("Unknown Book");
    let author_name = book
        .map(|b| storage.author_name_for_book(b))
        .unwrap_or("Unknown Author");

    let rule = "-".repeat(60);
    let mut out = String::new();

    out.push_str(&format!("\nReview of \"{}\" by {}\n", title, author_name));
    out.push_str(&format!(
        "Written on {}\n",
        review.created_on.format("%Y-%m-%d")
    ));

    let edits = review.edit_count();
    if edits > 0 {
        out.push_str(&format!(
            "Last edited on {} ({} edit{})\n",
            review.updated_on.format("%Y-%m-%d"),
            edits,
            if edits == 1 { "" } else { "s" }
        ));
    }

    out.push_str(&format!("{}\n{}\n", rule, review.text));

    if edits > 0 {
        out.push_str(&format!("\nHistory\n{}\n", rule));

        // Newest first, matching how reviews are listed elsewhere.
        for index in (1..review.revisions.len()).rev() {
            let previous = &review.revisions[index - 1];
            let current = &review.revisions[index];

            out.push_str(&format!(
                "Edited on {}\n",
                current.created_on.format("%Y-%m-%d")
            ));
            for line in line_diff(&previous.text, &current.text) {
                out.push_str(&match line {
                    DiffLine::Context(text) => format!("    {}\n", text),
                    DiffLine::Added(text) => format!("  + {}\n", text),
                    DiffLine::Removed(text) => format!("  - {}\n", text),
                });
            }
            out.push('\n');
        }

        let original = &review.revisions[0];
        out.push_str(&format!(
            "Written on {}\n",
            original.created_on.format("%Y-%m-%d")
        ));
        for line in original.text.lines() {
            out.push_str(&format!("    {}\n", line));
        }
    }

    out.push('\n');
    Some(out)
}
```

Replace the body of `show_review_detail` with:

```rust
pub fn show_review_detail(storage: &Storage, book_id: &str) -> io::Result<()> {
    let rendered = format_review_detail(storage, book_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Review not found"))?;
    print!("{}", rendered);
    Ok(())
}
```

- [ ] **Step 4: Add the Edits column to the listing**

In `show_reviews`, add a fifth header `"Edits".to_string()` after `"Date"`, and per row:

```rust
        let edits = review.edit_count();
        let edits_cell = if edits == 0 {
            String::new()
        } else {
            edits.to_string()
        };
```

Push it into the row between `date` and `preview`, and extend the alignment array to `[Left, Left, Right, Right, Left]`. The preview keeps using `truncate_text(&review.text, 60)`, which is now the current text.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --test review_test`
Expected: PASS.

- [ ] **Step 6: Wire up the edit flow in the CLI**

In `src/main.rs`:

- At `src/main.rs:1091`, choose the label from whether a review exists:

```rust
    let review_action = if storage.review_for_book(&selected_book.id).is_some() {
        "Edit review"
    } else {
        "Write review"
    };
    actions.push(review_action);
```

- At `src/main.rs:1289`, match on either label:

```rust
    if action_selection == "Write review" || action_selection == "Edit review" {
```

- At both editor call sites (around `src/main.rs:1297` and `src/main.rs:1403`), pass the current text so the buffer is pre-filled:

```rust
        let existing = storage.review_for_book(&book_id);
        let current = existing.as_ref().map(|r| r.text.as_str());
        match review::get_review_text_from_editor(&book_title, author_name, current) {
```

- In `review_book_flow`, the picker prompt and success message should say "Select a book to review:" and print `"Review saved successfully!"` when created and `"Review updated successfully!"` when a review already existed.

- [ ] **Step 7: Run the full suite and check the binary**

Run: `cargo test && cargo build`
Expected: PASS, clean build.

Run: `cargo run -- print-reviews` against a scratch storage file to confirm the table renders with the `Edits` column.

- [ ] **Step 8: Commit**

```bash
cargo fmt && cargo clippy
git add src/review.rs src/main.rs tests/review_test.rs
git commit -m "feat: show review history as dated diffs

The detail view prints the current text then each edit newest
first, with a line diff against the version before it. The book
menu offers Edit review once one exists.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

### Task 8: Update documentation

**Files:**
- Modify: `README.md`, `AGENTS.md`

**Interfaces:**
- Consumes: everything above.
- Produces: nothing.

- [ ] **Step 1: Update `AGENTS.md`**

In the "Key Concepts" section, the reading-events line currently lists six events. Replace it with:

```markdown
- **Reading events:** Books are tracked via `Reading` entries with events: `Started`, `Finished`, `Update`, `Bought`, `WantToRead`, `UnmarkedAsWantToRead`, `CreateReview`, `EditReview`. The most recent *status-bearing* event determines current status — `Update`, `CreateReview` and `EditReview` are non-status events, see `ReadingEvent::affects_status`.
- **Reviews:** A book has at most one review, derived by replaying its `CreateReview`/`EditReview` events. Each event stores a full text snapshot; diffs are computed at display time. See ADR 0016.
```

Add `diff.rs` to the source layout listing, after `config.rs`:

```
  diff.rs          # Line diff for review history
```

- [ ] **Step 2: Update `README.md`**

In the storage-file contents list, the `Reviews` bullet is still accurate. Add a line to whichever section describes the review commands, noting that writing a review for an already-reviewed book edits it and that `print-reviews -i` shows the edit history.

- [ ] **Step 3: Verify the docs match the code**

Run: `grep -n "CreateReview\|affects_status" AGENTS.md`
Expected: both appear.

- [ ] **Step 4: Commit**

```bash
git add AGENTS.md README.md
git commit -m "docs: describe review events and derived reviews

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>"
```

---

## Verification

Before considering the plan complete:

- [ ] `cargo test` passes with no ignored failures.
- [ ] `cargo clippy` is clean.
- [ ] `cargo fmt -- --check` passes.
- [ ] `grep -rn "storage.reviews\|add_review\|get_reviews_for_book" src/ tests/` returns nothing.
- [ ] `grep -A2 'name = "similar"' Cargo.lock` shows a 2.x version.
- [ ] Manual: point `bookmon` at a copy of a real storage file containing reviews, run any command, and confirm the migration reports what it discarded, wrote a `.pre-review-migration.bak`, and that `print-reviews` still lists the reviews.
