# 0002 - Event Sourcing for Reading Status

## Status

Accepted

## Context

Books can be in various states: not started, started, finished, bought, want-to-read. A book's status can change over time (e.g., started → finished → restarted). We need to track the full history while also being able to determine the current status.

## Decision

Use event sourcing for reading status. Each status change is recorded as an immutable `Reading` event with a timestamp. The current status is derived by looking at the most recent event for each book. Events include: `Started`, `Finished`, `Update`, `Bought`, `WantToRead`, and `UnmarkedAsWantToRead`.

For determining "is book started", the algorithm walks events from newest to oldest, skipping non-status events (Update, Bought, WantToRead), and returns true on `Started` or false on `Finished`.

## Consequences

- **Easier:** Full reading history is preserved. Can compute statistics by time period. Can track re-reads. Undo is possible by examining event history.
- **Harder:** Computing current status requires sorting and scanning events (O(n log n) per book). Logic for deriving status from events must be consistent across all query methods.
