# 0016 - Abandoned Reading Event

## Status

Accepted

## Context

Once a book was marked as started, the only way out of "currently reading" was to mark it finished. A reader who gives up on a book partway through had no honest option: the book either sat in the currently-reading list forever or got a `Finished` event it did not earn, which would also count it toward the yearly goal.

## Decision

1. **A new `ReadingEvent::Abandoned` variant.** It ends a read-through without finishing. `is_book_started` treats it like `Finished` (returns false); `is_book_finished` is unaffected, so an abandoned book is neither reading nor finished. A later `Started` begins a fresh attempt — event sourcing (ADR 0002) gives re-reads for free.
2. **Abandoned is its own status, not backlog.** `print-backlog` lists books with no `Started`/`Finished` event; an abandoned book has been started, so it stays out. This is deliberate: backlog implies intent to read, and abandoning is an explicit signal the other way.
3. **A `print-abandoned` command** (with `--series` and `-i` like the other list commands) and `Storage::get_abandoned_books`. Without it, abandoned books would be unreachable interactively: the default interactive list shows only currently-reading and want-to-read books, and the backlog excludes them. `print-abandoned -i` is also how a reader restarts one.
4. **Interactive entry only.** A "Mark as abandoned" action sits directly after "Mark as finished" in the book menu, offered only while the book is being read. No confirmation prompt — consistent with "Mark as finished", and nothing is lost since the event is appended, never overwriting history.
5. **Pages ledger.** In `pages_credited_by_year`, `Abandoned` keeps the pages already logged through `Update` events but, unlike `Finished`, credits nothing for the unread remainder.
6. **No metadata.** The event carries no "page stopped at" or reason. Progress is already in `Update` events; a reason would be presentational. Can be added later if asked for.

### Rejected: `StoppedReading`, `DidNotFinish`, `Paused`, `Dropped`

`Abandoned` matches the past-participle shape of `Started`/`Finished`/`Bought` and is the conventional shelf name in reading trackers. `Paused` implies intent to resume and would be a different feature.

### Rejected: listing abandoned books in the backlog

Would silently resurface a book the reader deliberately walked away from.

## Subagent Input

- **@book-domain-expert:** Recommended `Abandoned` over "DNF"-style names; advised a distinct status excluded from both currently-reading and backlog, and no metadata under YAGNI.
- **@ux-designer:** Recommended the label "Mark as abandoned" to match the existing "Mark as X" pattern, placed right after "Mark as finished", with no confirmation prompt. Also suggested a specific success message ("Book marked as abandoned."); the generic "Reading event added successfully!" was kept for consistency with every other event — a specific-message pass for all events would be a separate change.

## Consequences

### Easier

- Giving up on a book is a first-class, honest action; the yearly goal and statistics are not skewed by fake finishes.
- Second attempts are recorded naturally as a new `Started`.

### Harder

- One more status to keep consistent across queries: any new status-derivation logic must decide how `Abandoned` behaves.
- `sort_books` ranks abandoned books with "not started" books; no separate sort bucket was added.
