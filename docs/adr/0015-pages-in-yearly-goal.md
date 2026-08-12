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
