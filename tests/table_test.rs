use bookmon::table::format_table;

#[test]
fn test_table_ascii_rows_are_aligned() {
    let rows = vec![
        vec!["Name".to_string(), "City".to_string()],
        vec!["Alice".to_string(), "Oslo".to_string()],
        vec!["Bob".to_string(), "Bergen".to_string()],
    ];
    let output = format_table(&rows);
    // All lines between '+' markers should have the same total width
    let lines: Vec<&str> = output.lines().collect();
    let expected_width = lines[0].len();
    for line in &lines {
        assert_eq!(
            line.len(),
            expected_width,
            "Line has wrong width: {:?}",
            line
        );
    }
}

#[test]
fn test_table_with_norwegian_characters_rows_are_aligned() {
    let rows = vec![
        vec!["Tittel".to_string(), "Forfatter".to_string()],
        vec!["Bøker og sånt".to_string(), "Ås".to_string()],
        vec!["Første bok".to_string(), "Ørjan Håland".to_string()],
    ];
    let output = format_table(&rows);

    // All lines should have the same display width
    let lines: Vec<&str> = output.lines().collect();

    // Separator lines use only ASCII, so their byte length equals display width.
    // Data lines may have multi-byte chars so we must check display width.
    let expected_display_width = unicode_width::UnicodeWidthStr::width(lines[0]);
    for (i, line) in lines.iter().enumerate() {
        let display_width = unicode_width::UnicodeWidthStr::width(*line);
        assert_eq!(
            display_width, expected_display_width,
            "Line {} has display width {} but expected {}: {:?}",
            i, display_width, expected_display_width, line
        );
    }
}

#[test]
fn test_table_with_mixed_ascii_and_unicode() {
    let rows = vec![
        vec![
            "Title".to_string(),
            "Author".to_string(),
            "Category".to_string(),
        ],
        vec![
            "Ringenes herre".to_string(),
            "Tolkien".to_string(),
            "Fantasy".to_string(),
        ],
        vec![
            "Sløseri".to_string(),
            "Erna Ødegård".to_string(),
            "Økonomi".to_string(),
        ],
    ];
    let output = format_table(&rows);

    let lines: Vec<&str> = output.lines().collect();
    let expected_display_width = unicode_width::UnicodeWidthStr::width(lines[0]);
    for (i, line) in lines.iter().enumerate() {
        let display_width = unicode_width::UnicodeWidthStr::width(*line);
        assert_eq!(
            display_width, expected_display_width,
            "Line {} has display width {} but expected {}: {:?}",
            i, display_width, expected_display_width, line
        );
    }
}

#[test]
fn test_table_empty_input() {
    let rows: Vec<Vec<String>> = vec![];
    let output = format_table(&rows);
    assert_eq!(output, "");
}

#[test]
fn test_table_header_only() {
    let rows = vec![vec!["Name".to_string(), "Age".to_string()]];
    let output = format_table(&rows);
    assert!(!output.is_empty());
    // Should have header separator, header row, header separator (3 lines)
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_table_structure_and_content() {
    let rows = vec![
        vec!["Name".to_string(), "Age".to_string()],
        vec!["Alice".to_string(), "30".to_string()],
    ];
    let output = format_table(&rows);
    let lines: Vec<&str> = output.lines().collect();

    // 5 lines: header separator, header, header separator, data row, data separator
    assert_eq!(lines.len(), 5);

    // Header separators use '='
    assert!(lines[0].starts_with('+'));
    assert!(lines[0].contains('='));
    assert!(!lines[0].contains('-'));
    assert!(lines[2].starts_with('+'));
    assert!(lines[2].contains('='));

    // Data separator uses '-'
    assert!(lines[4].starts_with('+'));
    assert!(lines[4].contains('-'));
    assert!(!lines[4].contains('='));

    // Header and data rows use '|'
    assert!(lines[1].starts_with('|'));
    assert!(lines[1].ends_with('|'));
    assert!(lines[1].contains("Name"));
    assert!(lines[1].contains("Age"));

    assert!(lines[3].starts_with('|'));
    assert!(lines[3].ends_with('|'));
    assert!(lines[3].contains("Alice"));
    assert!(lines[3].contains("30"));
}

#[test]
fn test_table_with_emoji() {
    let rows = vec![
        vec!["Icon".to_string(), "Name".to_string()],
        vec!["\u{1f4da}".to_string(), "Books".to_string()],
    ];
    let output = format_table(&rows);

    let lines: Vec<&str> = output.lines().collect();
    let expected_display_width = unicode_width::UnicodeWidthStr::width(lines[0]);
    for (i, line) in lines.iter().enumerate() {
        let display_width = unicode_width::UnicodeWidthStr::width(*line);
        assert_eq!(
            display_width, expected_display_width,
            "Line {} has display width {} but expected {}: {:?}",
            i, display_width, expected_display_width, line
        );
    }
}
