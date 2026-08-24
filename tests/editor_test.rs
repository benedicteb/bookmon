use bookmon::editor::{instruction_block, strip_editor_text, SCISSORS};

#[test]
fn test_strips_everything_below_the_scissors_line() {
    let input = format!("My review.\n\n{}\n# Instructions here.\n", SCISSORS);
    assert_eq!(strip_editor_text(&input), Some("My review.".to_string()));
}

#[test]
fn test_hash_lines_in_the_body_are_preserved() {
    let input = format!(
        "# Verdict\n\nOrwell is cold.\n\n{}\n# Instructions here.\n",
        SCISSORS
    );
    assert_eq!(
        strip_editor_text(&input),
        Some("# Verdict\n\nOrwell is cold.".to_string())
    );
}

#[test]
fn test_returns_none_when_body_is_empty() {
    let input = format!("\n\n{}\n# Instructions here.\n", SCISSORS);
    assert_eq!(strip_editor_text(&input), None);
}

#[test]
fn test_returns_none_for_whitespace_only_body() {
    let input = format!("  \n  \n{}\n# Instructions.\n", SCISSORS);
    assert_eq!(strip_editor_text(&input), None);
}

#[test]
fn test_text_without_a_scissors_line_is_kept_whole() {
    let input = "A review with no scissors line.\n# Including this.";
    assert_eq!(
        strip_editor_text(input),
        Some("A review with no scissors line.\n# Including this.".to_string())
    );
}

#[test]
fn test_trims_surrounding_but_not_internal_whitespace() {
    let input = format!("\n\nFirst.\n\nSecond.\n\n{}\n# x\n", SCISSORS);
    assert_eq!(
        strip_editor_text(&input),
        Some("First.\n\nSecond.".to_string())
    );
}

#[test]
fn test_body_containing_a_scissors_line_is_kept_up_to_the_last_one() {
    // A review that happens to quote the scissors marker in its own text
    // must not be truncated there: only the LAST scissors line (the real
    // separator, appended by the template) discards anything.
    let input = format!(
        "My review.\n\n{}\nMore of my review after a scissors-looking line.\n\n{}\n# Instructions.\n",
        SCISSORS, SCISSORS
    );
    assert_eq!(
        strip_editor_text(&input),
        Some(format!(
            "My review.\n\n{}\nMore of my review after a scissors-looking line.",
            SCISSORS
        ))
    );
}

#[test]
fn test_instruction_block_comments_every_line_and_leads_with_scissors() {
    let block = instruction_block(&["First instruction.", "Second instruction."]);
    let lines: Vec<&str> = block.lines().collect();

    assert_eq!(lines[0], SCISSORS);
    assert_eq!(lines[1], "# First instruction.");
    assert_eq!(lines[2], "# Second instruction.");
    assert_eq!(strip_editor_text(&block), None);
}
