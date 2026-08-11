# Progress notes on reading updates

**Date:** 2026-08-11
**Status:** Approved, not yet implemented

## Problem

`bookmon` records reading progress as `ReadingEvent::Update` events carrying a
current page number. A page number alone says how far you got, not what you
made of it. There is no way to record a couple of sentences alongside an
update — what clicked, what dragged, what you want to steal for work.

## Scope

Add optional free-text notes to progress updates, entered through the
interactive mode.

Explicitly out of scope: any command that displays notes back. Notes are
written to the storage JSON and read from there. A viewer can be added later
if it turns out to be wanted.

Also out of scope: a non-interactive `update-progress` subcommand. It was
considered and dropped. It would require addressing a book by name from the
command line, and the codebase has no such pattern — substring filtering
exists only for *series names* on the print commands (`--series`), while
choosing a *book to act on* always goes through the `inquire` picker.
Introducing book-name matching for one command is not worth the inconsistency.

## Design

### Data model

`ReadingEvent` is unchanged. The note lives on the existing metadata struct
in `src/storage.rs`:

```rust
pub struct ReadingMetadata {
    #[serde(default)]
    pub current_page: Option<i32>,
    #[serde(default)]
    pub note: Option<String>,
}
```

A new constructor sits beside the existing `with_metadata`, which keeps its
current signature (one production caller, three test callers):

```rust
pub fn with_progress_note(book_id: String, current_page: i32, note: String) -> Self
```

It sets `event` to `ReadingEvent::Update`, `current_page` to the given page,
and `note` to `Some(note)`.

**Why extend the metadata rather than add an enum variant:** a note is a
property of a progress update, not a different kind of event. A separate
variant would mean either two events for one action, or a second variant
overlapping `Update` forever. Every place that reasons about event types —
`most_recent_reading_event`, `is_book_started`, the statistics and goal
queries — would need to learn about it and would treat it identically to
`Update`.

**Backwards compatibility:** `#[serde(default)]` means storage files written
before this change deserialize with `note: None`. There is no migration.
`sort_json_value` is unaffected.

### Interactive flow

In the book action menu (`src/main.rs`, around line 775), when a book is
started and not finished, the action list gains a second entry:

```
Update progress              (existing — page only)
Update progress with notes   (new)
```

The new action prompts for the current page exactly as the existing one
does, then opens `$EDITOR` pre-populated with a comment template. Both
actions write a `ReadingEvent::Update`; only the new one sets `note`.

**Abort behaviour:** if the editor text is empty after comment-stripping, the
whole event is discarded and nothing is written to storage. The user is told
the update was aborted. This matches `review-book`, which prints
"Review aborted (empty text)" and saves nothing. Falling back to a silent
page-only update would be surprising — the user chose the *with notes*
action.

The existing page-only "Update progress" action is untouched.

### Shared editor helper

`strip_editor_text` and `get_review_text_from_editor` in `src/review.rs` are
review-specific only in their template string. The editor-launching logic —
resolving `$EDITOR`/`$VISUAL` with a `vi` fallback, splitting the command so
values like `code --wait` work, writing a temp file, checking exit status,
stripping comments — is general.

Extract into a new `src/editor.rs`:

- `strip_editor_text(text: &str) -> Option<String>` (moved verbatim)
- `get_text_from_editor(template: &str) -> Result<Option<String>, Box<dyn Error>>`

`review.rs` keeps `get_review_text_from_editor(book_title, author_name)` as a
thin wrapper that builds its template and delegates, so its two callers in
`main.rs` are unchanged. `reading.rs` gains
`get_progress_note_from_editor(book_title, author_name)` doing the same with
a progress-note template.

This is the only refactoring in this change, and it is confined to code the
feature touches.

### Module boundaries

- `editor.rs` — knows how to get free text from the user's editor. Depends on
  nothing in the domain.
- `review.rs` / `reading.rs` — own their templates and their storage
  validation. Depend on `editor.rs`.
- `storage.rs` — owns the data model and persistence. Unaware of editors.
- `main.rs` — wires the interactive menu to the above.

## Testing

- `tests/storage_test.rs` — a `Reading` with a note round-trips through
  serialize/deserialize; JSON written before this change (no `note` key)
  deserializes to `note: None`; `with_progress_note` sets event, page and
  note correctly.
- `tests/editor_test.rs` — the `strip_editor_text` cases currently in
  `tests/review_test.rs` (lines 224 onward) move here, unchanged.
- `tests/review_test.rs` — keeps its `store_review` / display tests; its
  import of `strip_editor_text` moves to the new module path.

`get_text_from_editor` itself is not unit-tested, matching the existing
treatment of `get_review_text_from_editor` — it spawns a subprocess. The
comment-stripping logic it delegates to is tested directly.

## Notes for implementation

Per `AGENTS.md`, this project records decisions in `docs/adr/`. The data-model
choice above (extend `ReadingMetadata` rather than add a `ReadingEvent`
variant) is worth an ADR — the next number is 0014.
