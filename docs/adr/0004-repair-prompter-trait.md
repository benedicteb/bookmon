# 0004 - RepairPrompter Trait for Storage Repair

## Status

Accepted

## Context

The `handle_missing_fields` function in `storage.rs` was directly importing `inquire::Text` to prompt users for missing data (orphaned authors, categories, etc.). This coupled the data/storage layer to a specific UI library, making it impossible to test without user interaction.

## Decision

Introduce a `RepairPrompter` trait that abstracts user input during storage repair operations. The `storage.rs` module defines the trait; `main.rs` provides the `InquirePrompter` implementation for production use. Tests use a `TestPrompter` that returns predefined values.

Also split `load_storage` into two functions: `load_storage` (pure deserialization) and `load_and_repair_storage` (deserialization + repair with a prompter).

## Consequences

- **Easier:** Storage layer has no UI dependencies. `handle_missing_fields` is fully testable with mock prompters. Separation of concerns is clean.
- **Harder:** Callers must pass a prompter when using repair functionality. Slightly more ceremony for the common case.
