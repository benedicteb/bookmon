# 0007 - Replace pretty-table with unicode-aware table module

## Status

Accepted

## Context

The `pretty-table` crate (v0.1.3) calculates column widths using `str::len()`, which returns the byte count of a UTF-8 string rather than its display width. Norwegian characters like æ, ø, and å are each 2 bytes in UTF-8 but occupy only 1 column width on the terminal. This caused table rows to be misaligned whenever book titles, author names, or category names contained these characters.

The `pretty-table` crate is a small, low-download-count library (6K total downloads) with no option to customize width calculation, and the bug is fundamental to its design.

## Decision

Replace `pretty-table` with a custom `table` module (`src/table.rs`) that uses the `unicode-width` crate (473M+ downloads, the standard Rust crate for this purpose) for display width calculation.

The new module provides:
- `format_table(&[Vec<String>]) -> String` for generating table strings
- `print_table(&[Vec<String>])` for printing tables to stdout
- Correct alignment for ASCII, multi-byte Latin characters (æøå), and emoji
- The same visual style as the original (box-drawing with `+`, `|`, `-`, `=`)

## Consequences

- Table alignment is now correct for all Unicode characters, including Norwegian æøå.
- We control the table formatting code, making future customization straightforward.
- One fewer external dependency (`pretty-table` removed), replaced by the well-maintained `unicode-width`.
- The table formatting is fully tested with ASCII, Norwegian, and emoji test cases.
