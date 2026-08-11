# Progress Notes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the user attach a few sentences of free text to a reading progress update, entered through the interactive mode.

**Architecture:** `ReadingEvent` is unchanged; the note becomes an optional `note` field on the existing `ReadingMetadata` struct, so a progress update stays one event. The `$EDITOR`-launching logic already used for reviews is extracted into a new `src/editor.rs` module and shared. A new "Update progress with notes" action in the interactive book menu prompts for a page, then opens the editor.

**Tech Stack:** Rust 2021, `serde`/`serde_json` for persistence, `chrono` for timestamps, `uuid` for IDs, `inquire` for interactive prompts, `tempfile` for the editor temp file.

## Global Constraints

- Storage files written before this change must keep loading. Every new serialized field carries `#[serde(default)]`. There is no migration step.
- Run `cargo fmt` and `cargo clippy` before every commit. Both must be clean.
- Run `cargo test` before every commit. All tests must pass.
- Tests are integration tests in `tests/`, not `#[cfg(test)]` modules in `src/`. Follow the existing layout.
- IDs are UUID v4 strings; timestamps are `chrono::DateTime<Utc>`.
- Do not add dependencies. Everything needed is already in `Cargo.toml`.

**Spec:** `docs/superpowers/specs/2026-08-11-progress-notes-design.md`

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `src/editor.rs` | Create | Get free text from the user's `$EDITOR`. No domain knowledge. |
| `src/lib.rs` | Modify | Register the `editor` module. |
| `src/review.rs` | Modify | Keeps its review template; delegates editor mechanics to `editor.rs`. |
| `src/storage.rs` | Modify | `ReadingMetadata.note` + `Reading::with_progress_note`. |
| `src/reading.rs` | Modify | Progress-note template + editor call. |
| `src/main.rs` | Modify | New interactive menu action and its handler. |
| `tests/editor_test.rs` | Create | `strip_editor_text` cases, moved from `review_test.rs`. |
| `tests/review_test.rs` | Modify | Drops the moved tests and their import. |
| `tests/storage_test.rs` | Modify | Note round-trip, legacy JSON, constructor. |
| `tests/reading_test.rs` | Modify | Progress-note template. |
| `docs/adr/0014-progress-notes.md` | Create | Records the data-model decision. |

---

### Task 1: Extract the editor helper into its own module

Pure refactor — no behaviour changes. `strip_editor_text` and the editor-launching body of `get_review_text_from_editor` in `src/review.rs` are review-specific only in their template string. Everything else (resolving `$EDITOR`/`$VISUAL` with a `vi` fallback, splitting the command so `code --wait` works, temp file, exit status check, comment stripping) is general and is needed by progress notes too.

**Files:**
- Create: `src/editor.rs`
- Create: `tests/editor_test.rs`
- Modify: `src/lib.rs:1-10`
- Modify: `src/review.rs:1-77`
- Modify: `tests/review_test.rs:1` and `tests/review_test.rs:224-267`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  - `bookmon::editor::strip_editor_text(text: &str) -> Option<String>`
  - `bookmon::editor::get_text_from_editor(template: &str) -> Result<Option<String>, Box<dyn std::error::Error>>`
  - `bookmon::review::get_review_text_from_editor(book_title: &str, author_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>>` (unchanged signature, now a wrapper)

- [ ] **Step 1: Write the failing test**

Create `tests/editor_test.rs`. These are the six `strip_editor_text` tests currently at `tests/review_test.rs:224-267`, moved verbatim except for the import path:

```rust
use bookmon::editor::strip_editor_text;

#[test]
fn test_strip_editor_text_removes_comment_lines() {
    let input = "This is my review.\n# This is a comment.\nSecond line.";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("This is my review.\nSecond line.".to_string()));
}

#[test]
fn test_strip_editor_text_returns_none_for_empty() {
    let input = "# Only comments.\n# Nothing else.\n";
    assert_eq!(strip_editor_text(input), None);
}

#[test]
fn test_strip_editor_text_returns_none_for_whitespace_only() {
    let input = "  \n  \n# comment\n  ";
    assert_eq!(strip_editor_text(input), None);
}

#[test]
fn test_strip_editor_text_trims_surrounding_whitespace() {
    let input = "\n\nMy review.\n\n# comment\n\n";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("My review.".to_string()));
}

#[test]
fn test_strip_editor_text_preserves_internal_whitespace() {
    let input = "First paragraph.\n\nSecond paragraph.\n# comment";
    let result = strip_editor_text(input);
    assert_eq!(
        result,
        Some("First paragraph.\n\nSecond paragraph.".to_string())
    );
}

#[test]
fn test_strip_editor_text_handles_template_format() {
    let input = "A great book about dystopia.\n# Write your review of \"1984\" by George Orwell above.\n# Lines starting with # will be stripped.\n# An empty review (after stripping comments) will abort.\n";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("A great book about dystopia.".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test editor_test`
Expected: FAIL to compile — `error[E0432]: unresolved import bookmon::editor` (the module does not exist yet).

- [ ] **Step 3: Create the editor module**

Create `src/editor.rs`:

```rust
use std::io::Write;
use tempfile::NamedTempFile;

/// Strips comment lines (starting with #) and trims whitespace from editor text.
/// Returns None if the resulting text is empty (indicating the user aborted).
pub fn strip_editor_text(text: &str) -> Option<String> {
    let stripped: String = text
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string();

    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

/// Opens the user's default editor on a temp file pre-populated with `template`.
///
/// The editor is determined by checking $EDITOR, then $VISUAL, falling back to "vi".
/// Returns the edited text with comment lines stripped, or None if the result is
/// empty (the user aborted).
pub fn get_text_from_editor(
    template: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut tmp = NamedTempFile::new()?;
    write!(tmp, "{}", template)?;
    tmp.flush()?;

    let path = tmp.path().to_path_buf();

    // Split editor command to support values like "code --wait" or "subl -w"
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (editor_bin, editor_args) = parts
        .split_first()
        .ok_or("$EDITOR is empty after splitting")?;

    let status = std::process::Command::new(editor_bin)
        .args(editor_args)
        .arg(&path)
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(format!("Editor '{}' exited with non-zero status", editor).into());
    }

    let content = std::fs::read_to_string(&path)?;
    Ok(strip_editor_text(&content))
}
```

- [ ] **Step 4: Register the module**

In `src/lib.rs`, add `pub mod editor;` to the module list, keeping alphabetical order:

```rust
pub mod author;
pub mod book;
pub mod category;
pub mod config;
pub mod editor;
pub mod goal;
pub mod reading;
pub mod review;
pub mod series;
pub mod storage;
pub mod table;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test editor_test`
Expected: PASS — 6 tests.

- [ ] **Step 6: Make review.rs delegate**

In `src/review.rs`, delete `strip_editor_text` (lines 14-30) and replace the body of `get_review_text_from_editor` (lines 32-77). The signature does not change, so both callers in `main.rs` (lines 960 and 1066) are untouched. The result is:

```rust
/// Opens the user's default editor for writing a review of the given book.
///
/// Returns None if the review is empty after comment-stripping (user aborted).
pub fn get_review_text_from_editor(
    book_title: &str,
    author_name: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let template = format!(
        "\n# Write your review of \"{}\" by {} above.\n# Lines starting with # will be stripped.\n# An empty review (after stripping comments) will abort.\n",
        book_title, author_name
    );
    crate::editor::get_text_from_editor(&template)
}
```

The `use std::io` at the top of `review.rs` is still needed by `show_reviews` and `show_review_detail`. Remove nothing else.

- [ ] **Step 7: Clean up the old tests**

In `tests/review_test.rs`, delete lines 224-267 (the `// --- strip_editor_text tests ---` comment through the end of `test_strip_editor_text_handles_template_format`) — they now live in `tests/editor_test.rs`.

Change the import on line 1 from:

```rust
use bookmon::review::{show_review_detail, show_reviews, store_review, strip_editor_text};
```

to:

```rust
use bookmon::review::{show_review_detail, show_reviews, store_review};
```

- [ ] **Step 8: Verify the whole suite still passes**

Run: `cargo test`
Expected: PASS, with the same total test count as before the task (the six tests moved files, none were added or lost).

Run: `cargo fmt -- --check && cargo clippy -- -D warnings`
Expected: no output, exit 0.

- [ ] **Step 9: Commit**

```bash
git add src/editor.rs src/lib.rs src/review.rs tests/editor_test.rs tests/review_test.rs
git commit -m "refactor: extract editor text helpers into src/editor.rs"
```

---

### Task 2: Add the note field to reading metadata

**Files:**
- Modify: `src/storage.rs:122-127` (the `ReadingMetadata` struct) and `src/storage.rs:234-258` (the `impl Reading` block)
- Test: `tests/storage_test.rs`

**Interfaces:**
- Consumes: nothing from Task 1.
- Produces:
  - `bookmon::storage::ReadingMetadata { current_page: Option<i32>, note: Option<String> }`
  - `bookmon::storage::Reading::with_progress_note(book_id: String, current_page: i32, note: String) -> Reading` — sets `event` to `ReadingEvent::Update`.

- [ ] **Step 1: Write the failing tests**

Append to `tests/storage_test.rs`. The imports on lines 1-4 already bring in `Reading`, `ReadingEvent`, `ReadingMetadata` and `Storage`, so no import change is needed.

```rust
// --- progress note tests ---

#[test]
fn test_with_progress_note_sets_event_page_and_note() {
    let reading = Reading::with_progress_note(
        "book-id".to_string(),
        143,
        "Chapter on orthogonality finally clicked.".to_string(),
    );

    assert_eq!(reading.book_id, "book-id");
    assert_eq!(reading.event, ReadingEvent::Update);
    assert_eq!(reading.metadata.current_page, Some(143));
    assert_eq!(
        reading.metadata.note,
        Some("Chapter on orthogonality finally clicked.".to_string())
    );
}

#[test]
fn test_progress_note_round_trips_through_json() {
    let reading = Reading::with_progress_note(
        "book-id".to_string(),
        143,
        "First line.\n\nSecond paragraph.".to_string(),
    );

    let json = serde_json::to_string(&reading).expect("Failed to serialize reading");
    let deserialized: Reading =
        serde_json::from_str(&json).expect("Failed to deserialize reading");

    assert_eq!(deserialized.metadata.current_page, Some(143));
    assert_eq!(
        deserialized.metadata.note,
        Some("First line.\n\nSecond paragraph.".to_string())
    );
}

#[test]
fn test_reading_without_note_key_deserializes_to_none() {
    // A reading event as written by versions before progress notes existed.
    let json = r#"{
        "id": "11111111-1111-1111-1111-111111111111",
        "created_on": "2026-08-11T09:12:44Z",
        "book_id": "22222222-2222-2222-2222-222222222222",
        "event": "Update",
        "metadata": { "current_page": 50 }
    }"#;

    let reading: Reading = serde_json::from_str(json).expect("legacy JSON must deserialize");

    assert_eq!(reading.metadata.current_page, Some(50));
    assert_eq!(reading.metadata.note, None);
}

#[test]
fn test_default_metadata_has_no_note() {
    let metadata = ReadingMetadata::default();
    assert_eq!(metadata.current_page, None);
    assert_eq!(metadata.note, None);
}

#[test]
fn test_with_metadata_leaves_note_unset() {
    // The existing page-only constructor must keep working and set no note.
    let reading = Reading::with_metadata("book-id".to_string(), ReadingEvent::Update, 50);
    assert_eq!(reading.metadata.current_page, Some(50));
    assert_eq!(reading.metadata.note, None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test storage_test`
Expected: FAIL to compile — `no function or associated item named with_progress_note found` and `no field note on type ReadingMetadata`.

- [ ] **Step 3: Add the field**

In `src/storage.rs`, replace the `ReadingMetadata` struct (lines 122-127) with:

```rust
/// Optional metadata attached to a reading event.
///
/// `current_page` records progress for `Update` events. `note` holds the user's
/// free-text remarks about that progress, written in their editor.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ReadingMetadata {
    #[serde(default)]
    pub current_page: Option<i32>,
    #[serde(default)]
    pub note: Option<String>,
}
```

- [ ] **Step 4: Fix the existing constructors and add the new one**

Still in `src/storage.rs`, the `impl Reading` block (lines 234-258) now has two constructors that must name the new field. Replace the block with:

```rust
impl Reading {
    /// Creates a new reading event with a generated UUID and current timestamp.
    pub fn new(book_id: String, event: ReadingEvent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata::default(),
        }
    }

    /// Creates a new reading event with page progress metadata.
    pub fn with_metadata(book_id: String, event: ReadingEvent, current_page: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event,
            metadata: ReadingMetadata {
                current_page: Some(current_page),
                note: None,
            },
        }
    }

    /// Creates an `Update` reading event carrying both a page number and a
    /// free-text note about the reader's progress.
    pub fn with_progress_note(book_id: String, current_page: i32, note: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_on: Utc::now(),
            book_id,
            event: ReadingEvent::Update,
            metadata: ReadingMetadata {
                current_page: Some(current_page),
                note: Some(note),
            },
        }
    }
}
```

Note that `new` now uses `ReadingMetadata::default()` instead of spelling out `ReadingMetadata { current_page: None }`, so it will not need editing again if the struct grows.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test storage_test`
Expected: PASS, including the five new tests.

- [ ] **Step 6: Verify nothing else broke**

Run: `cargo test`
Expected: PASS.

Run: `cargo fmt -- --check && cargo clippy -- -D warnings`
Expected: no output, exit 0.

- [ ] **Step 7: Commit**

```bash
git add src/storage.rs tests/storage_test.rs
git commit -m "feat: add optional note field to reading metadata"
```

---

### Task 3: Add the "Update progress with notes" interactive action

**Files:**
- Modify: `src/reading.rs` (add two functions near the top, after the `store_reading` function at line 95-103)
- Modify: `src/main.rs:775-778` (the action list) and `src/main.rs:951` (insert a handler before the "Write review" block)
- Test: `tests/reading_test.rs`

**Interfaces:**
- Consumes:
  - `bookmon::editor::get_text_from_editor(template: &str) -> Result<Option<String>, Box<dyn std::error::Error>>` (Task 1)
  - `bookmon::storage::Reading::with_progress_note(book_id: String, current_page: i32, note: String) -> Reading` (Task 2)
- Produces:
  - `bookmon::reading::progress_note_template(book_title: &str, author_name: &str) -> String`
  - `bookmon::reading::get_progress_note_from_editor(book_title: &str, author_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>>`

- [ ] **Step 1: Write the failing tests**

Append to `tests/reading_test.rs`. Add `use bookmon::editor::strip_editor_text;` and `use bookmon::reading::progress_note_template;` to the imports at the top of the file, keeping the existing imports intact.

The last test is the important one: it proves that opening the editor and saving without typing anything aborts, rather than storing an event whose note is the template's own comment lines.

```rust
// --- progress note template tests ---

#[test]
fn test_progress_note_template_mentions_book_and_author() {
    let template = progress_note_template("The Pragmatic Programmer", "Hunt & Thomas");
    assert!(template.contains("The Pragmatic Programmer"));
    assert!(template.contains("Hunt & Thomas"));
}

#[test]
fn test_progress_note_template_lines_are_all_comments() {
    // Every non-empty line must be a comment, so an untouched template strips to nothing.
    let template = progress_note_template("Some Book", "Some Author");
    for line in template.lines() {
        assert!(
            line.trim().is_empty() || line.starts_with('#'),
            "template line is not a comment: {:?}",
            line
        );
    }
}

#[test]
fn test_untouched_progress_note_template_strips_to_none() {
    let template = progress_note_template("Some Book", "Some Author");
    assert_eq!(strip_editor_text(&template), None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test reading_test progress_note`
Expected: FAIL to compile — `unresolved import bookmon::reading::progress_note_template`.

- [ ] **Step 3: Add the template and editor call**

In `src/reading.rs`, add these two functions immediately after `store_reading` (which ends at line 103):

```rust
/// Builds the editor template shown when writing a progress note.
///
/// Every line is a comment, so an untouched template strips to nothing and
/// aborts the update.
pub fn progress_note_template(book_title: &str, author_name: &str) -> String {
    format!(
        "\n# Write a note about your progress in \"{}\" by {} above.\n# Lines starting with # will be stripped.\n# An empty note (after stripping comments) will abort the update.\n",
        book_title, author_name
    )
}

/// Opens the user's default editor for writing a progress note.
///
/// Returns None if the note is empty after comment-stripping (user aborted).
pub fn get_progress_note_from_editor(
    book_title: &str,
    author_name: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    crate::editor::get_text_from_editor(&progress_note_template(book_title, author_name))
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test reading_test`
Expected: PASS, including the three new tests.

- [ ] **Step 5: Add the menu entry**

In `src/main.rs`, the block at lines 775-778 currently reads:

```rust
    if is_started && !is_finished {
        actions.push("Update progress");
        actions.push("Mark as finished");
    }
```

Change it to:

```rust
    if is_started && !is_finished {
        actions.push("Update progress");
        actions.push("Update progress with notes");
        actions.push("Mark as finished");
    }
```

- [ ] **Step 6: Add the handler**

Still in `src/main.rs`, insert this block immediately before the `// Handle "Write review" action separately from reading events` comment at line 951. It must come before the `let event = match action_selection` at line 981, because it returns early — the note may abort, in which case no event is written at all.

```rust
    // Handle "Update progress with notes" separately: the note may abort the update
    if action_selection == "Update progress with notes" {
        let author_name = storage.author_name_for_book(selected_book);
        let author_name = if author_name.is_empty() {
            "Unknown Author"
        } else {
            author_name
        };

        let current_page = Text::new("Enter current page:")
            .prompt()
            .map_err(|e| format!("Failed to get current page: {}", e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("Invalid page number: {}", e))?;

        match reading::get_progress_note_from_editor(&selected_book.title, author_name) {
            Ok(Some(note)) => {
                let reading_event = storage::Reading::with_progress_note(
                    selected_book.id.clone(),
                    current_page,
                    note,
                );
                let mut storage = storage.clone();
                match reading::store_reading(&mut storage, reading_event) {
                    Ok(_) => {
                        storage::write_storage(storage_file, &storage)?;
                        println!("Progress update saved successfully!");
                    }
                    Err(e) => eprintln!("Failed to add reading event: {}", e),
                }
            }
            Ok(None) => {
                println!("Progress update aborted (empty note).");
            }
            Err(e) => eprintln!("Failed to get progress note: {}", e),
        }

        return Ok(());
    }
```

The `_ => unreachable!()` arm in the event match at line 988 stays correct, because this branch returns before reaching it.

- [ ] **Step 7: Verify the build and the whole suite**

Run: `cargo build`
Expected: compiles with no warnings.

Run: `cargo test`
Expected: PASS.

Run: `cargo fmt -- --check && cargo clippy -- -D warnings`
Expected: no output, exit 0.

- [ ] **Step 8: Verify the flow by hand**

The interactive menu is not covered by automated tests — `tests/interactive_test.rs` exercises the pure display-string helpers, not the `inquire` prompts. Check the real flow against a scratch storage file so your own book data is not touched:

```bash
cargo run -- change-storage-path /tmp/bookmon-manual-test.json
cargo run -- add-book          # add a book, mark it as started
cargo run -- -i                # select it, choose "Update progress with notes"
```

Confirm all four:
1. "Update progress with notes" appears in the action list for a started, unfinished book.
2. Entering a page then writing a note saves it — `jq '.readings[] | .metadata' /tmp/bookmon-manual-test.json` shows both `current_page` and `note`.
3. Saving the editor without typing anything prints "Progress update aborted (empty note)." and adds no reading event.
4. The plain "Update progress" action still works and writes `note: null`.

Restore your real storage path afterwards:

```bash
cargo run -- change-storage-path <your real path>
```

- [ ] **Step 9: Commit**

```bash
git add src/reading.rs src/main.rs tests/reading_test.rs
git commit -m "feat: add 'Update progress with notes' interactive action"
```

---

### Task 4: Record the decision as an ADR

`AGENTS.md` requires an ADR in `docs/adr/` for data-model decisions. The last one is `0013-table-column-alignment.md`, so this is 0014.

**Files:**
- Create: `docs/adr/0014-progress-notes.md`

**Interfaces:**
- Consumes: nothing. Documentation only.
- Produces: nothing.

- [ ] **Step 1: Write the ADR**

Create `docs/adr/0014-progress-notes.md`, following the format of the existing ADRs (Status / Context / Decision / Subagent Input / Consequences):

```markdown
# 0014 - Progress Notes

## Status

Accepted

## Context

Reading progress was recorded as `ReadingEvent::Update` events carrying only a
current page number. A page number says how far the reader got, not what they
made of it — what clicked, what dragged, what is worth stealing for work.

## Decision

1. **Note lives on the metadata, not on the event enum.** `ReadingMetadata`
   gains `note: Option<String>`. `ReadingEvent` is unchanged.
2. **New constructor** `Reading::with_progress_note(book_id, current_page, note)`
   sits beside the existing `with_metadata`, which keeps its signature.
3. **Interactive entry only.** A new "Update progress with notes" action in the
   book menu prompts for a page, then opens `$EDITOR`. The page-only
   "Update progress" action is untouched.
4. **Empty note aborts the whole update.** Nothing is written to storage,
   matching how `review-book` handles an empty review.
5. **Editor mechanics extracted** to `src/editor.rs` and shared by reviews and
   progress notes.

### Rejected: a new `ReadingEvent` variant

A note is a property of a progress update, not a different kind of event. A
separate variant would mean either two events for one user action, or a second
variant overlapping `Update` permanently. Every place that reasons about event
types — `most_recent_reading_event`, `is_book_started`, the statistics and goal
queries — would have to learn about it and would treat it identically to
`Update`.

### Rejected: a non-interactive `update-progress` subcommand

It would require addressing a book by name from the command line. The codebase
has no such pattern: substring filtering exists only for *series names* on the
print commands (`--series`), while choosing a *book to act on* always goes
through the `inquire` picker. Introducing book-name matching for one command
was not worth the inconsistency.

## Subagent Input

None recorded — the decision was made directly with the user during design.

## Consequences

### Easier

- Progress updates carry context, not just a number
- Editor mechanics are in one place, reusable by any future free-text feature

### Harder

- Nothing displays notes yet. They are written to the storage JSON and read
  from there. This was a deliberate scope choice; a viewer can follow.
- Two similarly-named menu actions ("Update progress" and "Update progress
  with notes") sit next to each other, which is slightly more to read.
```

- [ ] **Step 2: Verify the file renders and nothing else changed**

Run: `git status --short`
Expected: exactly one untracked file, `docs/adr/0014-progress-notes.md`.

- [ ] **Step 3: Commit**

```bash
git add docs/adr/0014-progress-notes.md
git commit -m "docs: add ADR 0014 for progress notes"
```

---

## Done when

- `cargo test`, `cargo fmt -- --check` and `cargo clippy -- -D warnings` are all clean.
- A started book's action menu offers "Update progress with notes"; it saves page and note together, and aborts on an empty note.
- A storage file written before this change still loads, with `note` reading as `None`.
