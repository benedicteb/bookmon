# 0013 - Table Column Alignment

## Status

Accepted

## Context

All table columns were center-aligned, which made the output hard to scan. Titles and author names of different lengths started at different horizontal positions, forcing the user's eye to zigzag across each column. Additionally, series group headers were centered, creating a visual disconnect from the indented series book titles below them.

## Decision

Introduce per-column alignment support and apply it consistently:

1. **`Alignment` enum** — `Left` (default), `Center`, `Right` added to `table.rs`.
2. **`format_structured_table` and `format_table`** accept an `&[Alignment]` parameter. If shorter than the column count, missing columns default to `Left`.
3. **Alignment convention:**
   - Text columns (Title, Author, Category, Preview) → `Left`
   - Numeric/date columns (Finished on, Started on, Added on, Days, Progress, Pages) → `Right`
   - Boolean flag columns (Bought, Want to read) → `Center`
4. **Header rows** use the same alignment as their data columns, so column text forms a consistent visual line from header to data.
5. **Group headers** changed from centered to left-aligned with a 2-space indent (plus 1-space gutter = 3 leading spaces), matching the indent of series book titles.
6. **Series book titles** get a 2-space indent (`"  #1 Title"`) so they visually nest under the group header.

## Subagent Input

- **@ux-designer:** Strongly recommended left-aligned text columns and right-aligned numeric/date columns — follows the universal table typography convention (F-pattern scanning). Recommended the 2-space indent for series books and left-aligning group headers to match. Suggested center alignment for single-character boolean flags. Recommended header alignment match data alignment.

## Consequences

### Easier

- Scanning long book lists — titles and authors start at a predictable left position
- Visual hierarchy — series books are clearly nested under their group header
- Date/number columns form a clean right edge, easy to compare

### Harder

- All callers of `format_table` and `format_structured_table` must pass an alignment slice (empty `&[]` preserves left-aligned default)
- The API is slightly more complex — two parameters instead of one
