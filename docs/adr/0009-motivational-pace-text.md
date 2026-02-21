# 0009 - Motivational Pace Text for Yearly Goal

## Status

Accepted

## Context

The yearly goal feature (ADR 0008) shows a progress bar and remaining count, but gives no sense of whether the user is on track. Users see "19 remaining" but have no idea if that's easy or difficult given the time left in the year. Adding a "books per month" pace line with motivational tone helps users gauge their progress and stay motivated.

Key design questions:
1. Where to put the logic — in `main.rs` alongside the other goal display functions, or in a separate module?
2. How to format the rate — decimals, whole numbers, or fractions?
3. How to handle edge cases — past years, future years, December, goal already met?
4. How to make the function testable when it depends on the current date?

## Decision

### Separate `goal.rs` module with a pure function

The motivational text logic lives in `src/goal.rs` as a public function `motivational_pace_text(finished, target, year, now)`. The `now: DateTime<Utc>` parameter is injected rather than calling `Utc::now()` internally, making the function fully deterministic and testable without mocking.

This was placed in a new module rather than in `main.rs` because:
- The display functions in `main.rs` are not testable from integration tests (they use `print!`)
- A pure `String`-returning function in `lib` is easy to test thoroughly
- It follows the existing pattern of domain modules (`reading.rs`, `review.rs`, etc.)

### Whole-number rounding (ceiling)

The books-per-month rate is always rounded up using ceiling division. "About 2 books per month" is clearer than "about 1.8 books per month". Users preferred simplicity over precision.

### Tone varies by pace

Three tone categories based on comparing the needed pace to the original yearly pace:
- **Comfortable** (<=1 book/month): "smooth sailing"
- **On track** (needed pace <= original yearly pace): "right on track"
- **Behind** (needed pace > original yearly pace): "time to pick up the pace"

Plus special cases for goal reached/exceeded, December (last month), past years, and future years.

### Display location: `print-goal` and default command only

The motivational text appears in `print_goal_status()`, which is called by both `print-goal` and the default (no subcommand) dashboard. It does NOT appear in `print-statistics` to keep that output compact.

## Consequences

### Easier

- Users immediately understand their reading pace relative to their goal
- The pure function design makes it trivial to add new tone categories or adjust thresholds
- 15 integration tests cover all edge cases, making future changes safe
- The `now` parameter injection pattern can be reused for other time-dependent features

### More difficult

- Adding a new module (`goal.rs`) means one more file to maintain
- The tone thresholds (on-track vs behind) are hardcoded; there's no user configuration for sensitivity
- Motivational messages are English-only with no i18n support
