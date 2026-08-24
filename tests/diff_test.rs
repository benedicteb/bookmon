use bookmon::diff::{line_diff, DiffLine};

fn rendered(old: &str, new: &str) -> Vec<String> {
    line_diff(old, new)
        .into_iter()
        .map(|line| match line {
            DiffLine::Context(text) => format!("  {}", text),
            DiffLine::Added(text) => format!("+ {}", text),
            DiffLine::Removed(text) => format!("- {}", text),
        })
        .collect()
}

#[test]
fn test_identical_text_yields_only_context() {
    let diff = line_diff("Same line.\nSecond.", "Same line.\nSecond.");
    assert!(diff.iter().all(|l| matches!(l, DiffLine::Context(_))));
}

#[test]
fn test_added_line() {
    assert_eq!(
        rendered("First.", "First.\nSecond."),
        vec!["  First.", "+ Second."]
    );
}

#[test]
fn test_removed_line() {
    assert_eq!(
        rendered("First.\nSecond.", "First."),
        vec!["  First.", "- Second."]
    );
}

#[test]
fn test_changed_line_is_a_removal_and_an_addition() {
    let out = rendered("Orwell is cold.", "Orwell is deliberately cold.");
    assert!(out.contains(&"- Orwell is cold.".to_string()));
    assert!(out.contains(&"+ Orwell is deliberately cold.".to_string()));
}

#[test]
fn test_empty_to_text_is_all_additions() {
    let diff = line_diff("", "A new review.");
    assert!(diff.iter().any(|l| matches!(l, DiffLine::Added(_))));
    assert!(!diff.iter().any(|l| matches!(l, DiffLine::Removed(_))));
}

#[test]
fn test_text_to_empty_is_all_removals() {
    let diff = line_diff("An old review.", "");
    assert!(diff.iter().any(|l| matches!(l, DiffLine::Removed(_))));
    assert!(!diff.iter().any(|l| matches!(l, DiffLine::Added(_))));
}

#[test]
fn test_lines_starting_with_hash_survive() {
    let out = rendered("# Heading", "# Heading\nBody.");
    assert_eq!(out, vec!["  # Heading", "+ Body."]);
}
