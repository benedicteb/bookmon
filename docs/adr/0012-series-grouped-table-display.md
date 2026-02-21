# 0012 - Series-Grouped Table Display

## Status

Accepted

## Context

Book tables (currently reading, finished, backlog/want-to-read) showed series membership as a flat "Series" column with values like "Harry Potter #3". This repeated the series name on every row within a series and didn't visually associate related books. Users with several series books saw scattered entries without a clear sense of which books belong together.

## Decision

Replace the flat Series column with visual grouping:

1. **Group header rows** — Series names appear as spanning header rows decorated with `──` (e.g., `── The Expanse ──`), centered across the full table width.
2. **No separators within groups** — Books in the same series have no row separator between them, creating visual cohesion.
3. **Position prefix on titles** — Titles within a group get a `#N` prefix (e.g., `#1 Leviathan Wakes`) replacing the Series column.
4. **Series column removed** — The separate "Series" column is eliminated when grouping is active, saving horizontal space.
5. **Flat fallback** — When no books have series, the table renders identically to the previous flat layout.
6. **Sort order** — Groups and standalone books interleave by author name. Within the same author, series groups precede standalone books. Within a group, books sort by position (using existing `compare_positions` logic).

### Table API extension

A new `TableRow` enum and `format_structured_table` function were added to `table.rs`:
- `TableRow::Header(Vec<String>)` — Column headers
- `TableRow::Data(Vec<String>)` — Regular data row
- `TableRow::GroupHeader(String, usize)` — Spanning series name row with count of grouped `Data` rows that follow

The `usize` count in `GroupHeader` explicitly bounds how many subsequent `Data` rows belong to the group. This prevents standalone `Data` rows after a group from being incorrectly treated as part of the preceding group (a bug found in the original `GroupHeader(String)` design).

The existing `format_table`/`print_table` API is preserved for backward compatibility.

### Reviews table excluded

The reviews table (`review.rs`) was deliberately excluded from grouping. Reviews are sorted by date (temporal ordering) which conflicts with series grouping. Both domain expert and UX designer recommended keeping reviews flat.

## Subagent Input

- **@book-domain-expert:** Recommended interleaving series groups with standalone books by author name (not segregating series first). Advised using author of lowest-positioned book as the "sort author" for the group. Confirmed position prefixes should use `#N` format (not "Vol." or "Book N"). Noted that fractional positions ("2.5") and zero-numbered prequels ("0") are standard practice and should sort correctly (existing `compare_positions` handles this). Recommended replacing the Series column with group headers to reduce redundancy.

- **@ux-designer:** Recommended `── Label ──` decoration on group headers to visually distinguish them from data rows. Advised against separator suppression for standalone books — the presence/absence of separators between rows is sufficient to signal grouping. Suggested keeping the flat table as default when no series exist (no unnecessary complexity). Flagged center-alignment of text columns as a future improvement opportunity. Recommended against adding a `--flat` flag preemptively — make grouping automatic and only add a toggle if users request it.

## Consequences

### Easier

- Scanning series progress across table views — books are visually clustered
- Understanding which books belong to a series without reading a column
- Narrower tables — removing the Series column saves ~20 characters of width

### Harder

- The `build_started_books_table` return type changed from `Vec<Vec<String>>` to `Vec<TableRow>`, breaking any code that indexed into the flat structure
- The table engine now has two APIs (`format_table` for flat, `format_structured_table` for grouped) — callers need to choose the right one
- Standalone books between two series groups could be visually ambiguous — they rely on the separator pattern to signal "not in a group"
