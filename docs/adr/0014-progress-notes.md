# 0014 - Progress Notes

## Status

Accepted

## Context

Reading progress was recorded as `ReadingEvent::Update` events carrying only a current page number. A page number says how far the reader got, not what they made of it — what clicked, what dragged, what is worth stealing for work.

## Decision

1. **The note lives on the metadata, not on the event enum.** `ReadingMetadata` gains `note: Option<String>`. `ReadingEvent` is unchanged.
2. **New constructor** `Reading::with_progress_note(book_id, current_page, note)` sits beside the existing `with_metadata`, which keeps its signature. `Reading::new` now builds `ReadingMetadata::default()` rather than naming fields, so it does not need editing when the struct grows.
3. **Interactive entry only.** A new "Update progress with notes" action in the book menu prompts for a page, then opens `$EDITOR`. The page-only "Update progress" action is untouched.
4. **An empty note aborts the whole update.** Nothing is written to storage, matching how `review-book` handles an empty review. The template is entirely comment lines, so saving an untouched buffer aborts.
5. **Editor mechanics extracted** to `src/editor.rs` and shared by reviews and progress notes.

### Rejected: a new `ReadingEvent` variant

A note is a property of a progress update, not a different kind of event. A separate variant would mean either two events for one user action, or a second variant overlapping `Update` permanently. Every place that reasons about event types — `most_recent_reading_event`, `is_book_started`, the statistics and goal queries — would have to learn about it and would treat it identically to `Update`.

### Rejected: a non-interactive `update-progress` subcommand

It would require addressing a book by name from the command line. The codebase has no such pattern: substring filtering exists only for *series names* on the print commands (`--series`), while choosing a *book to act on* always goes through the `inquire` picker. Introducing book-name matching for one command was not worth the inconsistency.

## Subagent Input

None recorded — the decision was made directly with the user during design.

## Consequences

### Easier

- Progress updates carry context, not just a number
- Editor mechanics live in one place, reusable by any future free-text feature

### Harder

- Nothing displays notes yet. They are written to the storage JSON and read from there. This was a deliberate scope choice; a viewer can follow.
- Two similarly-named menu actions ("Update progress" and "Update progress with notes") sit next to each other, which is slightly more to read.
