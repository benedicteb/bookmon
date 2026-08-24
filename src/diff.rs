use std::borrow::Cow;

use similar::{ChangeTag, TextDiff};

/// One line of a rendered diff between two versions of a text.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
}

/// Compares two texts line by line.
///
/// Every line of both inputs is represented in the result, in order — this is
/// a full diff, not a windowed one. Review texts are short enough that context
/// trimming would cost more clarity than it saves.
///
/// Trailing newlines are normalized away before comparison (see
/// [`ensure_trailing_newline`]), so a difference consisting solely of a
/// trailing newline is invisible in the returned lines. This is safe for the
/// current callers because review text is trimmed by `strip_editor_text`
/// before it is ever stored; a future caller feeding untrimmed text should be
/// aware that trailing-newline-only changes will not show up here.
pub fn line_diff(old: &str, new: &str) -> Vec<DiffLine> {
    let old = ensure_trailing_newline(old);
    let new = ensure_trailing_newline(new);
    TextDiff::from_lines(old.as_ref(), new.as_ref())
        .iter_all_changes()
        .map(|change| {
            let text = change.value().trim_end_matches('\n').to_string();
            match change.tag() {
                ChangeTag::Equal => DiffLine::Context(text),
                ChangeTag::Insert => DiffLine::Added(text),
                ChangeTag::Delete => DiffLine::Removed(text),
            }
        })
        .collect()
}

/// Appends a trailing newline unless `text` is empty or already ends in one.
///
/// `TextDiff::from_lines` splits on line boundaries and treats a line's
/// trailing newline as part of its content. Without this normalization, the
/// final line of an input that lacks a trailing newline (the common case for
/// review text) never compares equal to the same line elsewhere with a
/// newline attached, producing a spurious delete/insert pair instead of a
/// single context line. Normalizing first keeps the comparison based on line
/// *content*, and it also sidesteps a spurious empty change that
/// `from_lines("", ...)` would otherwise produce for an empty side.
fn ensure_trailing_newline(text: &str) -> Cow<'_, str> {
    if text.is_empty() || text.ends_with('\n') {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(format!("{text}\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_to_empty_yields_no_lines() {
        assert_eq!(line_diff("", ""), Vec::new());
    }
}
