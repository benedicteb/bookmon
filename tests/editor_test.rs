use bookmon::editor::strip_editor_text;

#[test]
fn test_strip_editor_text_removes_comment_lines() {
    let input = "This is my review.\n# This is a comment.\nSecond line.";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("This is my review.\nSecond line.".to_string()));
}

#[test]
fn test_strip_editor_text_returns_none_for_empty() {
    let input = "# Only comments.\n# Nothing else.\n";
    assert_eq!(strip_editor_text(input), None);
}

#[test]
fn test_strip_editor_text_returns_none_for_whitespace_only() {
    let input = "  \n  \n# comment\n  ";
    assert_eq!(strip_editor_text(input), None);
}

#[test]
fn test_strip_editor_text_trims_surrounding_whitespace() {
    let input = "\n\nMy review.\n\n# comment\n\n";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("My review.".to_string()));
}

#[test]
fn test_strip_editor_text_preserves_internal_whitespace() {
    let input = "First paragraph.\n\nSecond paragraph.\n# comment";
    let result = strip_editor_text(input);
    assert_eq!(
        result,
        Some("First paragraph.\n\nSecond paragraph.".to_string())
    );
}

#[test]
fn test_strip_editor_text_handles_template_format() {
    let input = "A great book about dystopia.\n# Write your review of \"1984\" by George Orwell above.\n# Lines starting with # will be stripped.\n# An empty review (after stripping comments) will abort.\n";
    let result = strip_editor_text(input);
    assert_eq!(result, Some("A great book about dystopia.".to_string()));
}
