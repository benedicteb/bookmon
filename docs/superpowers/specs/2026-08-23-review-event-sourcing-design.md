# Single review per book, with edit events

**Date:** 2026-08-23
**Status:** Approved, not yet implemented

## Problem

A book can currently hold any number of independent `Review` entities (ADR
0005). Each is a separate row with its own UUID and text, so re-reading a book
and writing a second review produces two unrelated reviews rather than a
revised one. There is no way to correct a typo, no record of what changed, and
no way to see how a judgement of a book moved over time.

What we want instead: one review per book, editable, where the current text is
the result of replaying that book's review events. Every edit is preserved as
its own event, so the review can be shown as a timeline — each change with its
date and a diff against the version before it.

## Scope

- Reviews become one-per-book. A second write edits the existing review.
- Review activity is recorded as events in the existing `readings` collection.
- The current review text is derived by folding those events, never stored as
  a standalone mutable field.
- The review detail view gains a history section with per-edit diffs.
- Existing storage files are migrated; extra reviews per book are discarded.

Out of scope: ratings, per-review visibility/privacy, editing an individual
past revision, reverting to an earlier revision, and any diff granularity finer
than whole lines. A separate `review-history` command was considered and
dropped — the history belongs in the detail view the user is already looking at.

Also out of scope, deliberately: renaming `Reading`/`readings` to
`BookEvent`/`events`. Once review events live in that collection the name is a
misnomer, but the rename is 558 occurrences across `src` and `tests` plus a
top-level JSON key migration. Bundling a mechanical rename into a semantic
change makes both harder to review and harder to back out. It can follow as its
own no-behaviour-change commit.

## Design

### Data model

Review events join the existing event enum in `src/storage.rs`, rather than
forming a parallel stream. This is what makes a single per-book timeline
possible: *started -> page 120 -> finished -> reviewed -> edited* reads as one
sequence.

```rust
pub enum ReadingEvent {
    Finished,
    Started,
    Update,
    Bought,
    WantToRead,
    UnmarkedAsWantToRead,
    CreateReview,   // new
    EditReview,     // new
}
```

The text rides on the existing metadata object, which is already where optional
per-event payload lives:

```rust
pub struct ReadingMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_page: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_text: Option<String>,   // new
}
```

`review_text` is a **full snapshot** of the review as of that event, not a
patch. Reviews are a few kilobytes at most, so snapshots cost nothing, and they
buy a history that cannot be corrupted by a bad patch and a storage file that
can still be repaired by hand (ADR 0001). Diffs are computed at display time and
never persisted.

A separate field rather than reusing `note`: a `note` is a remark *about*
reading progress, a review is the artifact itself. Keeping them apart means the
timeline renderer never has to infer which one it is holding.

`ReadingMetadata::is_empty()` gains the third field, so events that carry no
review text serialize exactly as they do today.

New `Reading` constructor, beside the existing `with_progress_note`:

```rust
impl Reading {
    pub fn with_review(book_id: String, event: ReadingEvent, text: String) -> Self
}
```

### Status classification

`most_recent_reading_event` (`src/storage.rs:647`) currently returns a book's
latest event with no filtering, and `is_book_finished` compares that to
`Finished`. Adding review events to the same collection would therefore make
writing a review un-finish the book, dropping it out of `print-finished`,
`print-statistics`, the series display (`src/series.rs:64`) and the interactive
actions menu (`src/main.rs:1050`).

So the variants get classified explicitly:

- **Status-bearing:** `Started`, `Finished`, `WantToRead`,
  `UnmarkedAsWantToRead`, `Bought`, `Abandoned`
- **Non-status:** `Update`, `CreateReview`, `EditReview`

`Abandoned` (ADR 0016) is status-bearing for the same reason `Bought` is:
`get_abandoned_books` is built on `get_books_by_most_recent_event(Abandoned)`
and would return nothing otherwise.

```rust
impl ReadingEvent {
    /// Whether this event participates in determining a book's current status.
    /// Progress updates and review activity say nothing about whether a book
    /// is started, finished, or wanted.
    pub fn affects_status(self) -> bool { /* exhaustive match */ }
}
```

`most_recent_reading_event` filters on `affects_status()`. `is_book_started`'s
existing skip-list (`src/storage.rs:726`) gains the two review variants; its
match is exhaustive, so the compiler enforces the classification.

`Bought` and `Abandoned` must stay status-bearing: `get_bought_books` and
`get_abandoned_books` are built on `get_books_by_most_recent_event` and would
return nothing otherwise.

**This fixes a pre-existing bug.** Today, `Started -> Finished -> Update` leaves
`is_book_finished` returning `false`, because `is_book_started` skips `Update`
but `most_recent_reading_event` does not. The two functions disagree, and the
doc comment on `ReadingEvent` describes the behaviour `is_book_started` has.
After this change a progress note no longer un-finishes a book. Books in
existing storage files that were wrongly un-finished this way will reappear as
finished.

### Deriving the current review

`Review` stays, but becomes a derived view rather than a persisted entity. It
loses its `id`; review identity is now the book.

```rust
pub struct ReviewRevision {
    pub created_on: DateTime<Utc>,
    pub event: ReadingEvent,   // CreateReview or EditReview
    pub text: String,
}

pub struct Review {
    pub book_id: String,
    pub created_on: DateTime<Utc>,   // from the CreateReview event
    pub updated_on: DateTime<Utc>,   // from the newest event
    pub text: String,                // the newest snapshot
    pub revisions: Vec<ReviewRevision>,  // oldest first
}
```

`Storage::review_for_book(&self, book_id: &str) -> Option<Review>` collects the
book's `CreateReview`/`EditReview` events, sorts by `created_on` ascending, and
folds them. Returns `None` when there are none.

`Storage::reviews` (the `HashMap` field), `add_review`, `get_review` and
`get_reviews_for_book` are removed. `Storage::all_reviews() -> Vec<Review>` (one
per reviewed book, newest `created_on` first) replaces the last of these for the
listing view. Sorting stays on `created_on` rather than `updated_on` so the
listing keeps its current order and is sorted by the date column it actually
shows.

### Migration

`migrate_reviews(storage_path) -> Result<bool, ...>` in `src/storage.rs`, called
from `load_and_repair_storage` immediately before `load_storage`, mirroring
`migrate_positions`.

It must run before deserialization. Serde ignores unknown keys by default, so
once `Storage` loses its `reviews` field a stale file would load cleanly and
silently drop every review. (`migrate_positions` runs first for a different
reason — an unmigrated position cannot be deserialized at all.)

1. Read the raw JSON. If there is no `reviews` key, or it is empty, return
   `Ok(false)` without writing anything — no spurious backup.
2. Write `<storage_path>.pre-review-migration.bak` before modifying anything,
   because this step discards data.
3. Group reviews by `book_id`; sort each group by `created_on` ascending.
4. The oldest becomes a `readings` entry: the review's existing UUID as the
   reading `id`, its `created_on`, `event: "CreateReview"`, and
   `metadata.review_text` set to its text. Reusing the UUID keeps the id stable
   and cannot collide. If the file has no `readings` key at all, it is created.
5. Report what was dropped, one line per book:
   `Discarded 2 later review(s) for "1984"`. Reviews referencing a book that no
   longer exists are dropped and reported the same way.
6. Remove the `reviews` key and write through `write_json_value`, so the
   migrated file is byte-identical in format to a normally saved one.

No prompter is needed — the discard policy is fixed, not asked about.

Running it twice is a no-op: after the first pass there is no `reviews` key.

### Write and edit flow

```rust
pub fn store_review(storage: &mut Storage, book_id: &str, text: String) -> Result<(), String>
```

Validates that the book exists, then consults `review_for_book`: `None` appends
a `CreateReview` event, otherwise an `EditReview`. If the text is byte-identical
to the current version, nothing is recorded — the timeline never shows an empty
diff.

The one-review-per-book rule is therefore "at most one `CreateReview` per book",
enforced here rather than by the type.

The interactive book menu entry reads "Write review" or "Edit review" depending
on whether one exists, and the `review-book` command behaves the same way. When
editing, the editor buffer is pre-filled with the current text.

`show_review_detail` takes a `book_id` instead of a `review_id`, and
`review_interactive_mode` selects by book.

### Editor stripping

`strip_editor_text` (`src/editor.rs:6`) drops every line beginning with `#`. A
review containing a markdown heading survives that today only because a review
is never re-opened. Once reviews can be edited, every edit would silently delete
its own headings, and the diff would show a deletion the user never made.

`get_text_from_editor` switches to git's scissors convention: the body is kept
verbatim and everything from the scissors line down is discarded.

```
<current review text>

# ------------------------ >8 ------------------------
# Edit your review of "1984" by George Orwell above.
# Everything below this line is ignored.
# An empty review aborts. Unchanged text records no edit.
```

An empty body still aborts, so saving an untouched create-template still
cancels. This applies to **all** callers, not just reviews: progress notes
(ADR 0014) go through the same function and have the identical bug.

### Timeline rendering

`show_review_detail(storage, book_id)` prints:

```
Review of "1984" by George Orwell
Written on 2026-03-04
Last edited on 2026-08-23 (3 edits)
------------------------------------------------------------
<current text>

History
------------------------------------------------------------
Edited on 2026-08-23
  - Orwell's prose is cold.
  + Orwell's prose is deliberately cold, and that is the point.

Edited on 2026-05-19
  ...

Written on 2026-03-04
  <original text>
```

Newest first, matching how the rest of the app orders reviews. The bottom entry
is the `CreateReview` with its full original text, so the timeline stands alone.
The `Last edited` line is omitted entirely when there are no edits.

Plain `+`/`-` prefixes, no ANSI colour — nothing else in the app emits colour.

`show_reviews` keeps its Title / Author / Date / Preview columns, where Date is
the creation date and Preview is the *current* text, and gains an `Edits`
column, blank when a review has never been edited.

### Diffing

A new `src/diff.rs` wraps the diff library behind a single function:

```rust
pub enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
}

pub fn line_diff(old: &str, new: &str) -> Vec<DiffLine>
```

Rendering maps these to a two-space indent with a ` `, `+`, or `-` prefix.

`Cargo.toml` gains, capped in the style ADR 0006 established:

```toml
# Capped for Rust 1.83 compat, see docs/adr/0006
similar = ">=2.7, <3"
```

The cap is required: `similar` 3.x is edition 2024 with MSRV 1.85, which Rust
1.83 cannot even parse the manifest of. Version 2.7.0 is edition 2018 with MSRV
1.60, and all four of its dependencies are optional, so with default features it
pulls in nothing.

The module boundary exists so that if the cap ever becomes untenable, replacing
`similar` with a hand-rolled LCS line diff touches one file.

## Testing

Integration tests in `tests/`, following the existing `tempfile` pattern.

**Status classification** (`tests/storage_test.rs`) — the regressions that
matter most:

- Finish a book, write a review: `is_book_finished` is still true and the book
  is still in `get_finished_books`.
- `Started -> Finished -> Update`: now finished (the pre-existing bug).
- `get_bought_books` still returns books whose latest status event is `Bought`.
- A review event does not make an unstarted book look started.

**Fold** (`tests/review_test.rs`):

- No events: `review_for_book` returns `None`.
- `CreateReview` only: text matches, one revision, `created_on == updated_on`.
- Create + two edits: newest text wins, three revisions oldest-first,
  `created_on` from the create and `updated_on` from the last edit.
- `store_review` twice: the second is an `EditReview`, and exactly one
  `CreateReview` exists for the book.
- `store_review` with unchanged text: no new event.
- `store_review` for an unknown book: `Err`.

**Migration** (new `tests/review_migration_test.rs`):

- One review per book: becomes one `CreateReview`, `reviews` key gone, `.bak`
  written, review id preserved.
- Multiple per book: oldest kept, later ones dropped and counted.
- Review pointing at a missing book: dropped and reported.
- No `reviews` key: returns `Ok(false)`, file unchanged, no `.bak`.
- Empty `reviews` object: same.
- Run twice: second run is a no-op.

**Pure functions** — unit tests:

- `line_diff`: added lines, removed lines, changed lines, empty to text, text to
  empty, identical input yields no changes.
- Scissors stripping: content below the scissors line removed, `#` lines in the
  body preserved, empty body returns `None`, no scissors line present still
  works.

The TTY half of the editor remains untested, as ADR 0005 already accepted.

## Follow-ups

- Rename `Reading`/`readings` to `BookEvent`/`events` as a standalone
  no-behaviour-change commit.
- A unified per-book timeline view showing reading and review events
  interleaved. The data model after this change supports it; the view does not
  exist yet.
