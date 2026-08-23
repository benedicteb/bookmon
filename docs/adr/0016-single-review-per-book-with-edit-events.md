# 0016 - Single Review Per Book, Recorded as Edit Events

## Status

Accepted. Supersedes the "Multiple reviews per book" and "Storage model"
decisions of ADR 0005; that ADR's editor workflow and review navigation
decisions still stand.

## Context

ADR 0005 gave each book any number of independent `Review` entities, reasoning
that a re-read deserves a follow-up review. In practice that produces unrelated
rows rather than a revised judgement: a typo cannot be corrected, and there is
no record of how an opinion moved between the first reading and the second.

We want one review per book that can be edited, where the current text is
derived by replaying that book's review events, so the review can be displayed
as a timeline of changes with a diff and a date for each.

## Decision

### 1. Review events live in the existing `readings` collection

`ReadingEvent` gains `CreateReview` and `EditReview`. Review activity is not a
parallel event stream.

A single collection is what makes a per-book timeline possible — *started ->
page 120 -> finished -> reviewed -> edited* is one sequence, and splitting it
across two collections would mean merging them back together at every display
site.

### 2. Events carry full text snapshots, not patches

`ReadingMetadata` gains `review_text: Option<String>`, holding the complete
review as of that event. Diffs are computed when rendering and never stored.

Storing patches would be more compact and would make the timeline free, but a
single malformed patch corrupts every later version irrecoverably, and it turns
the storage file into something that can no longer be read or repaired by hand —
against ADR 0001's central point. Reviews are kilobytes; snapshots cost nothing
worth the risk.

`review_text` is a distinct field rather than a reuse of `note`, because a
`note` is a remark *about* reading progress while a review is the artifact
itself.

### 3. Status-bearing events are classified explicitly

`ReadingEvent::affects_status()` divides the variants:

- Status-bearing: `Started`, `Finished`, `WantToRead`, `UnmarkedAsWantToRead`,
  `Bought`
- Non-status: `Update`, `CreateReview`, `EditReview`

`most_recent_reading_event` filters on it. Without this, writing a review would
un-finish a book, because that function returned the latest event unfiltered and
`is_book_finished` compares it to `Finished`.

**This fixes a pre-existing bug.** `Started -> Finished -> Update` already
leaves `is_book_finished` returning `false` today: `is_book_started` skips
`Update`, `most_recent_reading_event` does not, and the two disagree. Books
wrongly un-finished by a progress note will reappear as finished after this
change.

`Bought` stays status-bearing — `get_bought_books` is built on
`get_books_by_most_recent_event(Bought)` and would otherwise return nothing.

### 4. `Review` becomes a derived type

`Storage.reviews` is removed. `Review` loses its `id` and is computed by
`review_for_book(book_id)`, carrying `created_on`, `updated_on`, the current
`text`, and the full `revisions` list. Review identity is now the book.

### 5. Migration discards extra reviews, with a backup

`migrate_reviews` runs before deserialization, converting each book's oldest
review into a `CreateReview` event and discarding the rest, reporting each
discard. It writes `<storage_path>.pre-review-migration.bak` first.

It must precede `load_storage`: serde ignores unknown keys, so once `Storage`
loses its `reviews` field a stale file would load cleanly and lose every review
silently.

Folding the later reviews into the timeline as `EditReview` events was offered
and declined in favour of the simpler rule.

### 6. Editor input switches to scissors-style stripping

`get_text_from_editor` keeps the body verbatim and discards everything below a
git-style scissors line, instead of dropping every line beginning with `#`.

Re-opening a review for editing makes the old rule actively harmful: a review
with a markdown heading would lose it on every edit, and the diff would show a
deletion the user never made. The change applies to all callers, since progress
notes (ADR 0014) share the function and the bug.

### 7. `similar` is capped below 3.0

`similar = ">=2.7, <3"`. Version 3.x is edition 2024 with MSRV 1.85, which Rust
1.83 cannot parse the manifest of (ADR 0006). Version 2.7 is edition 2018, MSRV
1.60, with no non-optional dependencies. The diff call sits behind `src/diff.rs`
so the crate can be swapped for a hand-rolled LCS diff in one file.

### Rejected: renaming `Reading` to `BookEvent`

Once reviews live in the collection, the name is a misnomer. But the rename is
558 occurrences across `src` and `tests` plus a top-level JSON key migration,
and bundling a mechanical rename into a semantic change makes both harder to
review and harder to revert. It can follow as its own commit.

### Relationship to ADR 0014's rejection of a new variant

ADR 0014 declined to add a `ReadingEvent` variant for progress notes, on the
grounds that a note is a *property of* an `Update` rather than a distinct event.
That reasoning does not transfer. A note and its page number are one user action
at one moment; a review edit is its own action at its own time, and needs its
own timestamp precisely so the timeline can order it. That is what an event is
for.

## Subagent Input

None recorded — the decision was made directly with the user during design.

## Consequences

### Easier

- A review can be corrected or revised, and the change is visible with its date.
- The full history of an opinion is preserved and cannot be corrupted.
- Reading and review activity share one ordered stream, so a combined per-book
  timeline becomes a view over data that already exists.
- Two latent bugs are fixed: progress notes no longer un-finish a book, and
  editor input no longer eats markdown headings.

### Harder

- Storage grows with a full copy of the review text per edit. Negligible at
  review sizes, but it is not free.
- The `readings` collection and the `Reading` struct are now misnamed.
- Users with several reviews on one book lose all but the oldest. Mitigated by
  the backup file and by reporting each discard, not silently dropping them.
- Every exhaustive `match` on `ReadingEvent` must handle two more variants. The
  compiler enforces this, which is the point, but it is churn.
