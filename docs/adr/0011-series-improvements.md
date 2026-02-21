# 0011 - Series Improvements

## Status

Accepted

## Context

After the initial series implementation (ADR 0010), a review by domain and UX experts identified eight areas for improvement across data modeling, display, and interaction design. This ADR documents the full batch of changes made in response.

Key issues identified:

1. **`position_in_series: Option<i32>` was too restrictive** — Real-world series use fractional positions (2.5 for novellas), zero-numbered prequels, and potentially non-numeric entries. Goodreads, BookBrainz, and ISFDB all use string or decimal positions.

2. **`print-series` was a "dead listing"** — It showed book titles but no reading status, making it impossible to see progress through a series at a glance.

3. **Series lacked metadata** — No way to record whether a series is ongoing, completed, or abandoned, or how many books it contains.

4. **UX inconsistencies** — Confusing prompt wording for position input, leaked UUIDs in error messages, inconsistent confirmation message style, no marker for the current series in the "Change series" flow.

5. **Wasted screen space** — The Series column appeared in all book tables even when no books had series assignments.

6. **No duplicate position warning** — Users could accidentally assign two books to the same position without feedback.

7. **No series filtering** — Commands like `print-finished` had no way to filter results by series.

## Decision

### Data model changes

- **`position_in_series` changed from `Option<i32>` to `Option<String>`**. A custom serde deserializer (`deserialize_position`) accepts both JSON numbers (old i32 format) and strings (new format) for backward compatibility. Sorting uses f64 parsing with lexicographic fallback for non-numeric positions.

- **`SeriesStatus` enum added** with variants `Ongoing`, `Completed`, `Abandoned`. Stored as `Option<SeriesStatus>` on `Series` with `serde(default)` + `skip_serializing_if`.

- **`total_books: Option<u32>` added to `Series`** for tracking the known total number of books. Also uses `serde(default)` + `skip_serializing_if`.

### Display improvements

- **Enriched `print-series`** with reading status indicators: `✓` for finished, `▸` for reading, blank for unread. Shows progress counts ("3/7 read" when `total_books` is set, or "2 read" otherwise). Uses Unicode box-drawing separators.

- **Conditional Series column** in all book tables (`show_started_books`, `show_finished_books`, `print_book_list_table`). The column only appears when at least one displayed book has a series assignment.

### UX improvements

- **Improved position prompt**: "Book number in series (e.g. 3), or Enter for none" — clearer wording.
- **Standardized confirmation messages**: Consistent past-tense format echoing the relevant name.
- **"(current)" marker** in the Change series selection.
- **"(empty)" label** for series with no books in the Delete series flow.
- **No leaked UUIDs** in error messages — user-friendly text only.
- **Duplicate position warning** during add-book and interactive series assignment.

### CLI filtering

- **`--series` / `-s` flag** added to `print-finished`, `print-backlog`, and `print-want-to-read`. Uses case-insensitive substring matching.
- **Helpful empty-result messages** when the filter matches no books: shows the filter term, and when no series match at all, lists known series as suggestions.
- **Not added to `print-statistics`** (groups by year, not series), `print-series` (already organized by series), or `print-reviews`/`print-goal` (not relevant).

### Decided against

- **Renaming `print-series` to `list-series`** — cancelled to maintain consistency with the `print-*` command family. All display commands use the `print-` prefix.
- **Adding `author_id`, `description`, or `category_id` to Series** — series can span multiple authors and genres. These fields would create false constraints.
- **Multi-series per book** — acceptable limitation for now. The migration path (join table) is documented as a future option.

## Subagent Input

- **@book-domain-expert:** Advised that `i32` was too restrictive for series positions — real-world catalogs use strings/decimals. Recommended against adding `author_id`, `description`, or `category_id` to Series (series can have multiple authors, span genres). Confirmed that `status` and `total_books` were worth adding. Validated that one-series-per-book is acceptable for a personal collection tool.

- **@ux-designer:** Identified that `print-series` was a "dead listing" without reading status. Found inconsistent confirmation messages, confusing position prompt wording, leaked UUIDs in error messages, and the always-visible Series column wasting space. Recommended keeping `print-series` name for consistency with `print-*` family. Recommended case-insensitive substring matching for `--series` filter with helpful empty-result messages. Advised `-s` as the short flag.

## Consequences

- **Easier**: Tracking novellas and prequels with fractional/zero positions. Seeing series reading progress at a glance. Filtering large collections by series. Clean table output when series aren't in use.
- **Harder**: Position field is now a string, so validation is more permissive (any non-negative number accepted). Numeric sorting requires parsing.
- **Migration**: Fully backward compatible — old JSON files with integer positions are automatically converted by the custom deserializer. New fields use `serde(default)`.
