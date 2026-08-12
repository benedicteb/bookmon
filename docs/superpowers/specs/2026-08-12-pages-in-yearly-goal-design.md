# Pages in the Yearly Reading Goal — Design

## Problem

A yearly goal currently tracks only the number of books finished. ADR 0008 rejected
page-based tracking because `total_pages` is often unknown or zero. Users want the goal to
cover total pages read as well, so a year of long books reads differently from a year of
short ones.

## Decisions

A goal for a year carries **both** a books target and a pages target. Neither is optional.

Pages read in a year are counted from the reading event ledger — actual logged progress —
not by summing `total_pages` of the books finished that year. Marking a book `Finished`
credits its remaining pages, so users who never log progress updates still get a
meaningful page count.

## Data model

`Storage.goals` changes from `HashMap<i32, u32>` to `HashMap<i32, Goal>`:

```rust
pub struct Goal {
    pub books: u32,
    pub pages: u32,
}
```

`Goal` implements `Deserialize` through an untagged helper enum:

- a bare number (the old shape) loads as `{ books: N, pages: 0 }`
- an object loads as-is

Serialization always writes the object form, so a legacy file is upgraded the next time it
is saved. Nothing in the display path writes, so no command rewrites a healthy file just to
migrate it. Note the pre-existing `load_and_repair_storage` pass runs on every invocation
and does write when it repairs something — so a file needing repair is migrated as a side
effect of that write, including via the one silent branch (an orphaned `series_id`). The
migration is lossless either way. `to_sorted_json_string` needs no changes.

`Storage` API:

- `set_goal(year: i32, books: u32, pages: u32)`
- `get_goal(year: i32) -> Option<Goal>`
- `remove_goal(year: i32) -> Option<Goal>` (unchanged behaviour, new return type)

### Legacy goals and a zero pages target

A migrated legacy goal has `pages: 0`. `goal_percentage` returns 100% when the target is 0,
which would render as `Pages: 4210/0 (100%)`. The display therefore treats a pages target of
0 as *no pages target set*, while still showing the pages read so far:

```
Pages: 4210 read — no target set, use set-goal <books> <pages>
```

Storage still migrates to `pages: 0`; this guard applies only at render time. The same guard
applies in `print-statistics`: a year whose goal has a zero pages target shows its pages line
without a goal clause.

## Page ledger — `src/pages.rs`

A new pure module, sibling to `goal.rs`. The core function takes one book's readings sorted
ascending by `created_on`, plus that book's `total_pages`, and returns the pages credited
per year:

```rust
pub fn pages_credited_by_year(readings: &[&Reading], total_pages: i32) -> HashMap<i32, u32>
```

The walk keeps a running `last_page`, starting at 0. Every credit is attributed to the year
of the event that produced it.

| Event | Rule |
|---|---|
| `Started` | `last_page = 0` — a re-read earns its pages again |
| `Update(p)` | clamp `p` to `total_pages` when that is greater than 0; credit `max(0, p - last_page)`; then `last_page = max(last_page, p)` |
| `Finished` | credit `max(0, total_pages - last_page)` when `total_pages > 0`, otherwise credit nothing; then `last_page = 0` |
| `Bought`, `WantToRead`, `UnmarkedAsWantToRead` | ignored; `last_page` untouched |

Two consequences are deliberate:

- `last_page = max(last_page, p)` means a downward correction (a typo fix from 200 to 150)
  credits nothing and does not let pages 150–200 be counted a second time later.
- Clamping to `total_pages` means an over-reported update cannot inflate a year beyond the
  book's real length.

A book with `total_pages = 0` and no progress updates contributes nothing, silently. That is
the data-quality hole ADR 0008 identified; the existing `prompt_total_pages` repair path
already exists to fill it and is out of scope here.

`Storage` aggregates on top:

- `pages_read_by_year(&self) -> HashMap<i32, u32>` — groups all readings by book, sorts each
  book's readings ascending by `created_on` (the ordering `pages_credited_by_year` requires),
  and folds each book's map in
- `pages_read_in_year(&self, year: i32) -> u32`

## CLI

`set-goal` takes two required positionals. `--year` is unchanged and still defaults to the
current year.

```
$ bookmon set-goal 30 9000
Reading goal for 2026: 30 books, 9000 pages
```

This is a breaking change: `bookmon set-goal 30` now fails with a clap usage error.

## Display

`print-goal` keeps its current books block and appends one pages line:

```
Reading goal 2026: 12/30 books (40%)
████████░░░░░░░░░░░░ 18 remaining
That's about 4 books per month — time to pick up the pace!
Pages: 4210/9000 (47%)
```

The motivational pace text stays books-only, so `src/goal.rs` is not modified. The default
no-subcommand dashboard calls `print_goal_status`, so it inherits the pages line.

`print-statistics` gains a second line per year, leaving the existing books line's shape
intact:

```
2026: 12 books (Goal: 30 — 40% complete, 18 remaining)
      4210 pages (Goal: 9000 — 47% complete)
```

For a year with no goal set, both lines appear without their goal clauses. This changes
output for goal-less years, which is accepted: pages read is a statistic worth showing
whether or not a goal exists.

## Testing

`tests/pages_test.rs` (new) covers the ledger as a pure function:

- finish with no progress updates credits the full `total_pages`
- updates followed by a finish credit each segment once
- a re-read (`Started` after `Finished`) resets and credits again
- a downward correction credits nothing and does not double-count later
- an over-reported update is clamped to `total_pages`
- `total_pages = 0` with updates credits the update values; without updates credits nothing
- a December update plus a January finish splits credit across two years
- `Bought`, `WantToRead`, and `UnmarkedAsWantToRead` events are ignored

`tests/storage_test.rs` extends with:

- `Goal` deserializes from both the bare-number and object shapes
- a legacy goal round-trips out as the object form
- `pages_read_in_year` aggregates across multiple books
- `test_set_and_get_goal` updated to the new signature

`tests/goal_test.rs` needs no changes.

## Documentation

Add ADR `0015-pages-in-yearly-goal.md`, superseding ADR 0008's "books finished only"
decision and recording why the event ledger was chosen over summing `total_pages` of the
books finished in a year.

## Out of scope

- Pages in the motivational pace text
- Prompting to fill in missing `total_pages` as part of goal tracking
- Monthly or quarterly sub-goals
