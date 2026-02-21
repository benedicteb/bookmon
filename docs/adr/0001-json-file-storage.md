# 0001 - JSON File Storage

## Status

Accepted

## Context

bookmon needs to persist book collections, reading events, authors, and categories. We need a storage solution that is simple, portable, human-readable, and doesn't require a database server.

## Decision

Use a single JSON file for all data persistence. The `Storage` struct is serialized/deserialized using `serde_json`. JSON keys are sorted deterministically via `sort_json_value` to produce stable diffs in version control.

## Consequences

- **Easier:** Simple to implement, debug, and backup. No database setup required. Users can inspect and manually edit data if needed. Git-friendly with deterministic key ordering.
- **Harder:** No concurrent access support. All data must fit in memory. No indexing or query optimization — all queries are O(n) scans. Schema migrations must be handled manually.
