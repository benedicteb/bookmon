# 0010 - Book Series

## Status

Accepted

## Context

Users often read books that belong to a series (e.g. "Harry Potter", "Lord of the Rings", "A Song of Ice and Fire"). Without a series concept, there's no way to group related books together or track reading order within a series. Additionally, the OpenLibrary API provides series information in Edition data that we weren't previously capturing.

## Decision

Add a `Series` entity to the data model with the following design choices:

1. **One series per book** — A book has an optional `series_id` FK and optional `position_in_series: i32`. This keeps the model simple while covering the vast majority of real-world use cases.

2. **Optional assignment** — Books can exist without a series (`series_id: None`). During `add-book`, the user is prompted but can skip.

3. **Same storage pattern** — `Series` is stored as a `HashMap<String, Series>` in `Storage`, consistent with authors, categories, and reviews. Uses `#[serde(default)]` for backward compatibility.

4. **ISBN lookup integration** — The OpenLibrary provider now fetches Edition data via `/isbn/{isbn}.json` to extract the `series` field (e.g. `"Harry Potter #1"`). A `parse_series_string()` function splits this into name + position using regex.

5. **Case-insensitive matching** — `get_or_create_series()` matches existing series by name case-insensitively to avoid duplicates like "Harry Potter" vs "harry potter".

6. **Book fields use `skip_serializing_if`** — The `series_id` and `position_in_series` fields on `Book` use `#[serde(skip_serializing_if = "Option::is_none")]` so that standalone books don't clutter the JSON with null fields.

## Consequences

- **Easier**: Grouping books by series, displaying reading order, auto-detecting series from ISBN lookup.
- **Harder**: If a book belongs to multiple series (rare), users must pick one. This can be revisited later with a join table if needed.
- **Migration**: Existing JSON files load without changes due to `serde(default)`. No data migration needed.
- **Extra API call**: OpenLibrary lookups now make one additional HTTP request (to `/isbn/{isbn}.json`). This is best-effort — errors are silently ignored.
