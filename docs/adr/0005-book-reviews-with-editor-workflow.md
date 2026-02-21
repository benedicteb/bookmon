# 0005 - Book Reviews with Editor Workflow

## Status

Accepted

## Context

Users want to write free-form reviews of books they've read. Reviews need to be
persisted in the same JSON storage file as other data (books, readings, etc.).
Writing a review requires entering potentially long, multi-line text, which is
awkward with inline terminal prompts.

Key design questions:
1. How to capture multi-line review text in a CLI?
2. Should a book have one review or multiple?
3. How to store reviews alongside existing data?
4. How to browse/view reviews after writing them?

## Decision

### Editor workflow (git-commit style)

Reviews are written using the user's default editor (`$EDITOR`, falling back to
`$VISUAL`, then `vi`). A temporary file is created with comment instructions
(lines starting with `#`), the editor is opened, and after the user saves and
quits, comment lines are stripped. If the result is empty, the review is aborted.

This mirrors `git commit` behavior and is familiar to CLI users. It avoids the
limitations of single-line prompts for long-form text.

### Multiple reviews per book

A book can have multiple reviews. Each `Review` is a separate entity with its
own UUID, timestamp, and text. This allows users to write follow-up reviews
after re-reading, or capture evolving thoughts over time.

### Storage model

A `Review` struct with `id`, `created_on`, `book_id`, and `text` is stored in a
new `reviews: HashMap<String, Review>` field on `Storage`. The field uses
`#[serde(default)]` for backward compatibility with existing JSON files that
lack a `reviews` key.

### tempfile crate promotion

The `tempfile` crate was promoted from dev-dependency to regular dependency.
It provides `NamedTempFile` with automatic cleanup on drop, which is safer than
manual temp file management for the editor workflow.

### Review navigation

- `print-reviews`: table with Title, Author, Date, Preview (truncated to 60 chars)
- `print-reviews -i`: interactive select-and-view loop (select -> show full text -> return to list)
- "Write review" action available from the interactive book actions menu

## Consequences

**Easier:**
- Writing long-form reviews using familiar editor tooling
- Accumulating multiple reviews per book over time
- Backward-compatible storage — old JSON files load without migration

**Harder:**
- Editor workflow is not testable in automated integration tests (requires TTY)
- The `tempfile` crate is now a runtime dependency (small cost)
- No rating system — reviews are text-only (can be added later as a field)
