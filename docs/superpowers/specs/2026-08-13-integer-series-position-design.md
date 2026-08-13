# Integer Series Position — Design

## Problem

`Book.position_in_series` is an `Option<String>`. Commit `efca310` moved it from `i32` to
`String` so fractional numbering (`"2.5"` for a novella between books 2 and 3), zero-numbered
prequels (`"0"`) and non-numeric labels were all expressible.

The string form has two costs. Ordering is recovered at comparison time — `compare_positions`
parses each side as `f64` on every comparison and falls back to lexicographic ordering for
values it cannot parse. And the type permits values the application itself refuses to create:
`parse_position_input` rejects anything that is not a non-negative number, so a non-numeric
label can only enter the file by hand-editing, yet every reader must still handle it.

The JSON schema is looser still. `"type": ["string", "number"]` carries no `pattern` and no
`minimum`, so `"banana"` validates, and the description advertises non-numeric labels as a
feature that the code does not implement.

We are giving up fractional positions in exchange for a type that means what it says. That is
a deliberate trade: a novella at `2.5` has no integer slot, and repairing one requires
renumbering the books after it. The design therefore treats renumbering as a first-class
operation rather than an afterthought.

## Decisions

`Book.position_in_series` becomes `Option<i32>`, constrained to non-negative values. `0` stays
legal — it is the prequel convention already in use.

Files containing non-integer positions are migrated **interactively, once**, at load time. The
user is prompted per affected book and chooses the new position; nothing is silently rounded
or discarded.

A new interactive `edit-series` command lets positions be moved after the fact, so a series
whose ordering was disturbed by the migration can be repaired.

## Data model

```rust
/// Optional position within the series (e.g. 1, 2, 0 for prequels).
#[serde(default)]
#[serde(skip_serializing_if = "Option::is_none")]
#[serde(deserialize_with = "deserialize_position")]
pub position_in_series: Option<i32>,
```

`deserialize_position` accepts the on-disk shapes that are losslessly integral and rejects
everything else:

| On-disk value | Result |
|---|---|
| absent, `null`, `""` | `None` |
| JSON integer `3` | `Some(3)` |
| string `"3"` | `Some(3)` |
| JSON `3.0`, string `"3.0"` | `Some(3)` |
| string `"2.5"`, JSON `2.5` | error |
| string `"Book Three"` | error |
| any negative | error |

The error is what makes the migration safe: it guarantees no unmigrated value can reach the
rest of the program as a silently altered integer. Its message names the book id and the
offending value, and says to run `bookmon` to migrate.

`compare_positions` reduces to an `Option<i32>` ordering with `None` last. The `f64` parse and
the lexicographic branch both disappear — a non-numeric position is now unrepresentable, so
the fallback has nothing left to catch.

## Migration

Once the field is `i32`, a file holding `"2.5"` cannot be deserialized at all, so the prompt
cannot happen after load. The migration runs **before** deserialization, on raw JSON.

`load_storage` is unchanged in spirit: it deserializes strictly and fails on unmigrated data.
It stays prompter-free, so tests and any non-interactive caller keep their current signature.

`load_and_repair_storage` gains a pass that runs first:

1. Read the file to a `serde_json::Value`.
2. Walk `books.*.position_in_series`, collecting every entry that is not a **non-negative
   integer**. This is deliberately the same predicate the deserializer accepts, not merely
   "is it fractional" — a legacy negative such as `-1` is integral but invalid, and if the
   migration ignored it the deserializer would reject the file on every subsequent run with
   no way for the user to fix it from inside the app.
3. Skip any book whose `series_id` is absent or dangles — `handle_missing_fields` clears the
   positions of orphaned books anyway, so prompting for them would ask the user to supply a
   value that is discarded moments later. Their position is cleared in the `Value` instead.
4. Prompt for each remaining entry, grouped by series and ordered by current position, so
   `2.5` is seen in the context of the books it sits between.
5. Patch the `Value` and write the file after each answer.
6. Deserialize the patched `Value` into `Storage`, then run `handle_missing_fields` as today.

Writing after each answer matches how `handle_missing_fields` already saves incrementally: an
interrupted migration keeps the answers already given, and re-running resumes with the rest.

### Prompter

`RepairPrompter` gains one method, keeping the migration testable with a fake prompter in the
same way the existing repair tests work:

```rust
fn prompt_series_position(
    &self,
    book_title: &str,
    series_name: &str,
    old_value: &str,
    suggested: Option<i32>,
    taken: &[i32],
) -> Result<PositionChoice, Box<dyn std::error::Error>>;

pub enum PositionChoice {
    /// Place the book here, shifting occupants at or after this position up by one.
    Insert(i32),
    /// Place the book here, leaving other books alone. May leave two books sharing a
    /// position — permitted, matching today's behaviour where `is_position_occupied`
    /// only warns.
    Set(i32),
    /// Drop the position; the book stays in the series and sorts last.
    Clear,
}
```

`taken` is the set of positions already occupied in that series, shown to the user so the
choice is informed.

### Suggested default

For a numeric value `v`, the suggestion is `ceil(v)` clamped to a minimum of `0` — `2.5 → 3`,
`0.5 → 1`, `-1 → 0`. Ceiling rather than truncation because `2.5` sorts *after* book 2: slot 3,
with everything from 3 up shifted by one, preserves the original reading order exactly.

When the suggestion is already occupied — which it usually is, since a `2.5` normally implies
a book 3 — the prompt offers `Insert` as the default action. `Set` is available for the case
where the user knows the occupant is wrong, and `Clear` for a novella they would rather leave
unnumbered.

A non-numeric value (`"Book Three"`) has no reliable anchor, so no suggestion is offered; the
user gets `taken` and types a number or clears it.

## `edit-series`

A new `EditSeries` variant on `Commands`, interactive-only, matching `DeleteSeries` and
`RenameSeries`. No flags — the other series commands take none, and nothing here needs
scripting.

Flow: select a series → see its books in order with their positions → select a book → enter a
new position. When the target position is occupied, offer:

- **Swap** — exchange the two books' positions.
- **Insert** — place the book here and shift the occupant and everything after it up by one.
- **Cancel** — leave it alone.

Insert is the operation that repairs a series broken by the migration; swap covers the
ordinary "these two are backwards" case.

The mutation logic lives in `series.rs` as pure functions over `Storage`, so it is testable
without driving the CLI:

- `shift_positions_from(storage, series_id, from: i32, except: &str)` — increments the
  position of every book in the series whose position is `>= from`. `except` is the book id
  being placed, skipped so it is not shifted out of the slot it was just given.
- `swap_positions(storage, series_id, book_a: &str, book_b: &str)` — takes two book ids.

The migration's `PositionChoice::Insert` uses the same `shift_positions_from`, so migration
and `edit-series` cannot drift apart in how they resolve a collision.

## Call sites

Signature changes ripple to:

- `series.rs` — `parse_position_input(&str) -> Option<i32>`, `format_position_prefix(Option<i32>)`,
  `format_series_label(&Series, Option<i32>)`, `is_position_occupied(.., position: i32)`.
- `storage.rs` — `compare_positions(Option<i32>, Option<i32>)`, `get_books_in_series`.
- `reading.rs` — three `format_position_prefix` call sites in the table builders.
- `book.rs` — the series-assignment prompt in `select_series`, including its `.with_default(pos)`
  which now formats an `i32`.
- `main.rs` — the interactive series-assignment branch, plus the new `EditSeries` arm.

## JSON schema

```json
"position_in_series": {
  "type": "integer",
  "minimum": 0,
  "description": "Optional non-negative position within the series; 0 marks a prequel. Omitted entirely when absent (never written as null). Books without a position sort after those with one. Legacy files may hold this as a string (\"3\") or a non-integer value (\"2.5\"); integral strings are read as numbers, and non-integer values are migrated interactively on load and then rewritten in this form.",
  "examples": [0, 1, 3]
}
```

The schema describes what bookmon writes. An unmigrated legacy file does not validate against
it, which is correct — that file is in the old format until the migration rewrites it.

`README.md`'s series section is updated to say positions are whole numbers and to document
`edit-series`.

## Testing

TDD, failing tests first, following the existing test layout.

`tests/series_test.rs`
- `parse_position_input` accepts `"3"`, `"0"`; rejects `"2.5"`, `"-1"`, `""`, `"abc"`.
- `format_position_prefix` / `format_series_label` on the integer type.
- `shift_positions_from` shifts only positions `>= from`, skips the excepted book, leaves other
  series untouched.
- `swap_positions` exchanges two positions and is a no-op for a book not in the series.
- `test_get_books_in_series_with_fractional_positions` is rewritten as an integer-ordering test
  covering `0` (prequel) and confirming `2` sorts before `10`.

`tests/storage_test.rs`
- The deserializer matrix above, accept and reject rows both asserted.
- `compare_positions` with `None` sorting last.
- Migration driven by a fake prompter: `Insert` shifts later books; `Set` overwrites; `Clear`
  drops the position; a book with a dangling `series_id` is cleared without a prompt; the file
  on disk is rewritten with integers; a second load prompts nothing.
- `test_book_series_fields_backward_compatibility_with_integer_position` now exercises the
  legacy JSON-number path directly.

Clippy is not clean in this repo (~39 pre-existing warnings), so the gate is **no new
warnings**, not zero.

## Out of scope

- Reintroducing fractional or non-numeric positions in any form.
- Non-interactive flags for `edit-series`.
- Automatic renumbering of a whole series (`edit-series` moves one book at a time).
