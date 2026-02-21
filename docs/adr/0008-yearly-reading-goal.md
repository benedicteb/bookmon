# 0008 - Yearly Reading Goal

## Status

Accepted

## Context

Users want to set a target number of books to finish per year and track their progress against that goal. The existing `PrintStatistics` command shows year-by-year finished book counts but has no concept of a target to measure against.

Key design questions:
1. Where to store the goal data — in the config file or the storage JSON file?
2. What metric to track — books finished or pages read?
3. How to surface goal progress in the CLI?

## Decision

### Storage location: Storage JSON file

Goals are stored as a `HashMap<i32, u32>` (year → target book count) in the `Storage` struct, alongside books, readings, authors, categories, and reviews. This was chosen over the config file because:

- Goals are per-year data, not a single global setting
- They travel with the data file (portable across machines)
- They participate in the same deterministic JSON serialization (version-control friendly)
- The `#[serde(default)]` pattern (already used for `reviews`) provides backward compatibility

### Metric: Books finished only

Only the number of finished books is tracked, not pages read. The `total_pages` field is unreliable (many books have unknown/zero page counts), making page-based tracking impractical with current data quality.

### CLI integration

- `set-goal <target> [--year <year>]` — sets a goal (defaults to current year)
- `print-goal [--year <year>]` — shows goal progress with a progress bar
- Default `bookmon` command (no subcommand) — shows current year goal status above the currently-reading table, only when a goal is set
- `print-statistics` — shows per-year goal progress inline with book counts

## Consequences

### Easier

- Users can set and track yearly reading goals entirely within the CLI
- The default dashboard view immediately shows goal progress without extra commands
- Multiple years can have independent goals (useful for year-end planning)
- Old storage files work without migration (empty goals HashMap by default)

### More difficult

- The `Storage` struct grows by one field, slightly increasing JSON file size
- No support for monthly/quarterly sub-goals (would require a more complex data model)
- No automatic goal suggestions based on past reading pace
