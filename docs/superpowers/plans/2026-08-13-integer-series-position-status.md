# Integer Series Position — Execution Status

Durable record of the work: what the plan is, how far it got, and every decision
taken along the way. Written because the working ledger lives in `.superpowers/`,
which is git-ignored scratch and will not survive.

**Spec:** [`docs/superpowers/specs/2026-08-13-integer-series-position-design.md`](../specs/2026-08-13-integer-series-position-design.md) (committed `ce1f308`)
**Plan:** [`docs/superpowers/plans/2026-08-13-integer-series-position.md`](2026-08-13-integer-series-position.md) (committed `c649d40`) — the six tasks, with the full code and test bodies for each. Not duplicated here; it is already in git at that path.

**Branch:** `main`, committed in place. The user was offered an isolated worktree, a
feature branch, or main, and explicitly chose main.

**Baseline captured before starting:** `cargo test` all green;
`cargo clippy 2>&1 | grep -c "^warning"` = **44**. The clippy gate for every task is
"no more than 44", never zero — this repo has a large pre-existing warning count.

---

## Status: all 6 tasks written; Task 5's fix is unverified — no Rust toolchain

| # | Task | Commits | Status |
|---|---|---|---|
| — | Ignore `.superpowers/` scratch dir | `72da764` | done |
| 1 | Flip `position_in_series` to `Option<i32>` | `f0fd017` | complete, review clean |
| 2 | `shift_positions_from` / `swap_positions` helpers | `39c8de4` | complete, review clean |
| 3 | Scan raw JSON for invalid positions | `ea91c43` | complete, review clean |
| 4 | Interactive migration on load | `4a02c58`, `b8a1b1d`, `3db6c84` | complete after 2 fix rounds |
| 5 | `edit-series` command | `33d1c35` + **uncommitted fix in the working tree** | fix round 1 applied, **not compiled, not tested, not committed** |
| 6 | JSON schema + README | `01fa5c0` | complete (JSON well-formedness verified with `node`) |

### Blocker: no Rust toolchain in the current environment

The session that resumed this work has no `cargo`, `rustc` or `rustup` on `PATH`, and no
`~/.cargo` anywhere on the box — `python3` is absent too (the schema was validated with
`node` instead). So `cargo fmt`, `cargo test` and `cargo clippy` could not be run for
Task 5's fix. The edit is mechanical and hand-checked, but it is unverified by the plan's
own gates, which is why it was left uncommitted rather than committed on faith.

**Resume point:** in an environment with cargo, run `cargo fmt`, `cargo test`, and
`cargo clippy 2>&1 | grep -c "^warning"` (must be ≤ 44) against the working-tree change to
`src/main.rs`, commit it as `fix: report book titles in edit-series confirmations`, then
re-review Task 5, then run the final whole-branch review and the Verification section's
manual migration check.

### Task 5's finding — fix applied in the working tree, unverified

`src/main.rs:833, 843, 877, 942` (pre-fix line numbers) — the confirmation messages
printed the internal list label instead of the book's title. `picked` was bound to the
`Select` result over `labels`, which are built as `format!("{} {}", position_prefix,
title)`, so a successful edit printed `Moved '#3 Novella' to #4.` and clearing printed
`Removed the position from '— Some Book'.` This was on every success path (assign, insert,
swap). It came verbatim from the plan's own Step 4 sample code, so it is a plan defect,
not an implementer deviation.

Applied fix: `books` is now a `Vec<(String, String, String)>` of `(id, title, label)`; the
`Select` still shows `label`, and the destructuring `let (book_id, book_title) = ...` feeds
the plain title to both `println!`s. The plan's Task 5 Step 4 sample code still carries the
defect — anyone re-running that step from the plan will reintroduce it.

---

## Rulings taken on the user's behalf

Each is a decision made without asking, with what it costs if wrong.

1. **Pre-flight — plan ellipsis stands.** Task 1 Step 7's `...existing setup...` inside a
   code block is a placeholder by the letter of the rubric, but it unambiguously means
   "keep this test's fixture, change positions and name". *Cost if wrong:* the implementer
   rebuilds a fixture; caught by the test passing.

2. **Pre-flight — missing import lines are the implementer's to add.** Task 4's test code
   omitted `use std::cell::RefCell` and the `PositionChoice` import. *Cost if wrong:* one
   compile round inside the implementer's own loop.

3. **Task 3 — accepted a deviation from the plan's literal code.** The plan's
   series-resolution `and_then` chain triggered a new `clippy::manual_option_zip` warning
   (45 vs the 44 baseline); the implementer rewrote it as `Option::zip`. The Global
   Constraint outranks the plan's literal code, and the reviewer traced all three branches
   as behaviourally identical. *Cost if wrong:* a semantic difference in series resolution
   that three existing branch tests would have had to miss.

4. **Task 4 — fixed a silent data-loss path the plan mandated.** The plan's
   `InquirePrompter::prompt_series_position` mapped *any* unparseable answer to
   `PositionChoice::Clear` with nothing printed, so typing `2.5` at a prompt that had just
   said your old value was `2.5` silently discarded it — on the one path where the original
   value is unrecoverable. The spec's binding constraint is "never silently round or discard
   a user's position value". Now: unparseable input prints why and re-prompts, bounded by
   3 attempts. *Cost if wrong:* a more insistent prompt.

5. **Task 4 — promoted two Minors into the fix round instead of deferring them.** Both were
   one-test additions pinning behaviour in the code just written: no test anywhere called
   `load_and_repair_storage`, so the migrate-before-load ordering — the highest-risk seam in
   the plan — was guarded only by a comment; and `Clear` removing the JSON key rather than
   nulling it was asserted only indirectly. *Cost if wrong:* ~20 lines of test.

6. **Task 4 — opened a second fix round on a regression the re-review had accepted.** To make
   "Enter leaves it unnumbered" literally true, round 1 swapped `Text::with_default` for
   `with_placeholder`, which meant the suggested position could no longer be accepted at all
   — pressing Enter at a prompt reading "suggested 3" cleared the position instead. The user
   had explicitly asked for a suggested default, and a default you cannot accept with Enter
   is not one; it recreated the same footgun round 1 was raised to remove. Now Enter accepts
   the suggestion and clearing requires typing `none`. *Cost if wrong:* users type `none`
   instead of Enter to unnumber a book.

7. **Task 5 — accepted a second clippy-driven deviation.** The plan's literal `sort_by` in
   the new flow would have pushed clippy to 45; the implementer used `sort_by_key`. Same
   reasoning as ruling 3, and the reviewer independently confirmed 44. *Cost if wrong:* a
   sort-order change the reviewer would have had to miss.

---

## Deferred minor findings

None of these block merge; the final whole-branch review should triage them.

**Task 1**
- `-0` parses to `Some(0)` because IEEE `-0.0 < 0.0` is false (`src/storage.rs:368`). Harmless — 0 is a legal position — but incidental rather than intended, and untested.
- The `value > i32::MAX as f64` guard (`src/storage.rs:371-373`) has no covering test.

**Task 2**
- No test covers shifting or swapping a book already at position `0`, though `0` is the legal prequel position.
- `swap_positions(a, a)` is a harmless no-op, intentional but untested.

**Task 3**
- `scan_invalid_positions` sorts by `(series_name, suggested, book_id)` but no test asserts that ordering, even though it determines the order the user is prompted in.

**Task 4**
- Non-atomic `fs::write` in `write_json_value` (`src/storage.rs:1234`). Parity with the pre-existing `write_storage`, but migration is the one irreversible transform in the app — a temp-file-plus-rename, or a one-time `.bak` of the pre-migration file, would be cheap insurance.
- Two invalid books in one series that ceiling to the same slot are ordered by `book_id`, so the suggested defaults can invert their original reading order. The user sees `taken` and chooses, so nothing is lost, but the suggestion steers toward the inversion.
- `position + 1` overflow parity with `series::shift_positions_from` for a book at `i32::MAX`.
- Dead defensive branch at `src/storage.rs:1176-1179`: a failed book lookup returns *after* the shift was applied, persisting a shift with no placement. Unreachable today, but the wrong shape for a function whose partial application corrupts positions.

**Task 5**
- Series and book selection match on formatted display strings rather than stable ids. Mirrors the existing `rename_series_flow` / `delete_series_flow` pattern, but `edit-series` is specifically the tool for repairing collapsed series, which is the case most likely to produce colliding labels.
- "Insert here" can leave a gap at the moved book's old position. No contiguity requirement exists in the spec, so this is a UX quirk rather than a bug.
- `src/main.rs` is now 1469 lines and the repeated `*_series_flow` boilerplate is accumulating. A `src/cli/series.rs` extraction would help; out of scope here.

---

## Notes worth keeping

- Task 1's implementer found a factual error in the plan: it cited commit `efca510` for the
  original `i32` → `String` migration; the real commit is `efca310`. Corrected in the code.
- Task 4's implementer found a real defect in Task 3's shared test fixture
  (`storage_json_with_positions` was missing `Series.created_on`), which Task 3 never hit
  because it never deserialized. The reviewer confirmed the fix is additive.
- Mid-run the user restricted subagents to the project directory — no reading or searching
  the home directory, `~/.cargo`, or dependency sources. Carried into all later dispatches.
- Task 6 deviated from the plan's pointer, not its intent: the plan located the series
  documentation at `README.md:165`, but that line is inside the **ISBN lookup** section
  ("Series information (name and position)"), describing lookup-provider data — which
  Task 1 deliberately left as a `String`. The real target is the `#### Series Management`
  section at `:145`. Edited there; `:165` left alone on purpose.
- The user's stored note about this repo's clippy baseline said ~39; that figure came from
  `cargo clippy --all-targets -- -D warnings`. Plain `cargo clippy | grep -c "^warning"` is
  44. Different commands, not a moved baseline — pick one and use it consistently.
