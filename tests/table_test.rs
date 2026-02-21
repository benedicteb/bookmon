use bookmon::table::{format_structured_table, format_table, Alignment, TableRow};

#[test]
fn test_table_ascii_rows_are_aligned() {
    let rows = vec![
        vec!["Name".to_string(), "City".to_string()],
        vec!["Alice".to_string(), "Oslo".to_string()],
        vec!["Bob".to_string(), "Bergen".to_string()],
    ];
    let output = format_table(&rows, &[]);
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
    let output = format_table(&rows, &[]);

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
    let output = format_table(&rows, &[]);

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
    let output = format_table(&rows, &[]);
    assert_eq!(output, "");
}

#[test]
fn test_table_header_only() {
    let rows = vec![vec!["Name".to_string(), "Age".to_string()]];
    let output = format_table(&rows, &[]);
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
    let output = format_table(&rows, &[]);
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
    let output = format_table(&rows, &[]);

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

// ── Structured table tests ──────────────────────────────────────────

#[test]
fn test_structured_table_empty_input() {
    let rows: Vec<TableRow> = vec![];
    let output = format_structured_table(&rows, &[]);
    assert_eq!(output, "");
}

#[test]
fn test_structured_table_data_only_no_groups() {
    // Without any GroupHeader, behaves like a regular table
    let rows = vec![
        TableRow::Header(vec!["Name".to_string(), "Age".to_string()]),
        TableRow::Data(vec!["Alice".to_string(), "30".to_string()]),
        TableRow::Data(vec!["Bob".to_string(), "25".to_string()]),
    ];
    let output = format_structured_table(&rows, &[]);
    let lines: Vec<&str> = output.lines().collect();

    // Structure: =header= header =header= data -sep- data -sep-
    // = 7 lines
    assert_eq!(lines.len(), 7);

    // Each data row should have a separator after it
    assert!(lines[4].starts_with('+') && lines[4].contains('-'));
    assert!(lines[6].starts_with('+') && lines[6].contains('-'));
}

#[test]
fn test_structured_table_with_group_header() {
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        TableRow::GroupHeader("The Expanse".to_string(), 2),
        TableRow::Data(vec![
            "#1 Leviathan Wakes".to_string(),
            "James S.A. Corey".to_string(),
        ]),
        TableRow::Data(vec![
            "#2 Caliban's War".to_string(),
            "James S.A. Corey".to_string(),
        ]),
    ];
    let output = format_structured_table(&rows, &[]);
    let lines: Vec<&str> = output.lines().collect();

    // Structure:
    // 0: +=====+=====+        (header thick sep)
    // 1: | Title | Author |   (header row)
    // 2: +=====+=====+        (header thick sep)
    // 3: | The Expanse    |   (group header spanning)
    // 4: | #1 Lev.. | James.. | (data — no separator after because next is Data in same group)
    // 5: | #2 Cal.. | James.. | (data — separator after because end of group)
    // 6: +-----+-----+        (thin sep)
    assert_eq!(lines.len(), 7, "Output:\n{}", output);

    // Group header row should contain the series name and span full width
    assert!(
        lines[3].contains("The Expanse"),
        "Group header should contain series name"
    );
    assert!(lines[3].starts_with('|') && lines[3].ends_with('|'));

    // No separator between the two grouped data rows (lines 4 and 5 are both data)
    assert!(lines[4].starts_with('|'), "First data row in group");
    assert!(
        lines[5].starts_with('|'),
        "Second data row in group — no separator before it"
    );

    // Separator after the last data row in the group
    assert!(lines[6].starts_with('+') && lines[6].contains('-'));
}

#[test]
fn test_structured_table_mixed_groups_and_standalone() {
    // Standalone rows come before series groups (sorted by caller).
    // All Data rows between GroupHeaders belong to the preceding group.
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        // Standalone book (before any group header)
        TableRow::Data(vec!["Piranesi".to_string(), "Susanna Clarke".to_string()]),
        TableRow::GroupHeader("Harry Potter".to_string(), 2),
        TableRow::Data(vec![
            "#1 Philosopher's Stone".to_string(),
            "J.K. Rowling".to_string(),
        ]),
        TableRow::Data(vec![
            "#2 Chamber of Secrets".to_string(),
            "J.K. Rowling".to_string(),
        ]),
        TableRow::GroupHeader("Discworld".to_string(), 1),
        TableRow::Data(vec![
            "#1 The Colour of Magic".to_string(),
            "Terry Pratchett".to_string(),
        ]),
    ];
    let output = format_structured_table(&rows, &[]);
    let lines: Vec<&str> = output.lines().collect();

    // Structure:
    // 0: +=====+=====+
    // 1: | Title | Author |
    // 2: +=====+=====+
    // 3: | Piranesi | Susanna.. |  (standalone data, sep after)
    // 4: +-----+-----+
    // 5: |    Harry Potter    |  (group header)
    // 6: | #1 Phil.. | J.K. .. |  (data, no sep — next is data in same group)
    // 7: | #2 Cham.. | J.K. .. |  (data, sep after — next is GroupHeader)
    // 8: +-----+-----+
    // 9: |    Discworld    |  (group header)
    // 10: | #1 Colour.. | Terry.. |  (data, sep after — end of rows)
    // 11: +-----+-----+
    assert_eq!(lines.len(), 12, "Output:\n{}", output);

    // Verify standalone row and its separator
    assert!(lines[3].contains("Piranesi"));
    assert!(lines[4].starts_with('+'));

    // Verify group headers
    assert!(lines[5].contains("Harry Potter"));
    assert!(lines[9].contains("Discworld"));

    // Verify no separator between grouped rows (lines 6 and 7)
    assert!(lines[6].starts_with('|'));
    assert!(lines[7].starts_with('|'));

    // Verify separator after group (line 8)
    assert!(lines[8].starts_with('+'));
}

#[test]
fn test_structured_table_standalone_after_group_has_separator() {
    // Regression test: standalone Data rows after a group must have separators.
    // Previously, they were incorrectly treated as part of the preceding group.
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        TableRow::GroupHeader("The Expanse".to_string(), 2),
        TableRow::Data(vec![
            "#1 Leviathan Wakes".to_string(),
            "James S.A. Corey".to_string(),
        ]),
        TableRow::Data(vec![
            "#2 Caliban's War".to_string(),
            "James S.A. Corey".to_string(),
        ]),
        // These are standalone — NOT part of The Expanse
        TableRow::Data(vec!["Piranesi".to_string(), "Susanna Clarke".to_string()]),
        TableRow::Data(vec![
            "Project Hail Mary".to_string(),
            "Andy Weir".to_string(),
        ]),
        TableRow::GroupHeader("Discworld".to_string(), 1),
        TableRow::Data(vec![
            "#1 The Colour of Magic".to_string(),
            "Terry Pratchett".to_string(),
        ]),
    ];
    let output = format_structured_table(&rows, &[]);
    let lines: Vec<&str> = output.lines().collect();

    // Structure:
    // 0: +=====+=====+
    // 1: | Title | Author |
    // 2: +=====+=====+
    // 3: | ── The Expanse ── |  (group header)
    // 4: | #1 Leviathan.. |    (grouped, no sep)
    // 5: | #2 Caliban's.. |    (grouped, sep after — end of group)
    // 6: +-----+-----+
    // 7: | Piranesi |          (standalone, sep after)
    // 8: +-----+-----+
    // 9: | Project Hail.. |    (standalone, sep after)
    // 10: +-----+-----+
    // 11: | ── Discworld ── |  (group header)
    // 12: | #1 Colour.. |      (grouped, sep after — end of rows)
    // 13: +-----+-----+
    assert_eq!(lines.len(), 14, "Output:\n{}", output);

    // No separator between grouped rows
    assert!(lines[4].starts_with('|'), "First grouped row");
    assert!(
        lines[5].starts_with('|'),
        "Second grouped row — no separator before"
    );

    // Separator after the group ends
    assert!(lines[6].starts_with('+'), "Separator after Expanse group");

    // Each standalone row gets its own separator
    assert!(lines[7].contains("Piranesi"), "Piranesi is standalone");
    assert!(lines[8].starts_with('+'), "Separator after Piranesi");
    assert!(lines[9].contains("Project Hail Mary"), "PHM is standalone");
    assert!(lines[10].starts_with('+'), "Separator after PHM");

    // Next group header
    assert!(lines[11].contains("Discworld"), "Discworld group header");
}

#[test]
fn test_structured_table_all_lines_same_display_width() {
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        TableRow::GroupHeader("The Expanse".to_string(), 2),
        TableRow::Data(vec![
            "#1 Leviathan Wakes".to_string(),
            "James S.A. Corey".to_string(),
        ]),
        TableRow::Data(vec![
            "#2 Caliban's War".to_string(),
            "James S.A. Corey".to_string(),
        ]),
        TableRow::Data(vec![
            "Standalone Novel".to_string(),
            "Some Author".to_string(),
        ]),
    ];
    let output = format_structured_table(&rows, &[]);
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
fn test_structured_table_group_header_with_unicode() {
    let rows = vec![
        TableRow::Header(vec!["Tittel".to_string(), "Forfatter".to_string()]),
        TableRow::GroupHeader("Ringenes Herre".to_string(), 1),
        TableRow::Data(vec![
            "#1 Ringens brorskap".to_string(),
            "J.R.R. Tolkien".to_string(),
        ]),
    ];
    let output = format_structured_table(&rows, &[]);
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
fn test_structured_table_group_header_with_zero_count() {
    // Edge case: GroupHeader with count 0 means no Data rows belong to it.
    // The group header should render, and a standalone Data row after it should
    // still get its own separator.
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        TableRow::GroupHeader("Empty Series".to_string(), 0),
        TableRow::Data(vec![
            "Standalone Book".to_string(),
            "Some Author".to_string(),
        ]),
    ];
    let output = format_structured_table(&rows, &[]);
    let lines: Vec<&str> = output.lines().collect();

    // Structure:
    // 0: +=====+=====+
    // 1: | Title | Author |
    // 2: +=====+=====+
    // 3: | ── Empty Series ── |  (group header, but 0 grouped rows)
    // 4: | Standalone Book | Some Author |  (standalone — sep after)
    // 5: +-----+-----+
    assert_eq!(lines.len(), 6, "Output:\n{}", output);
    assert!(lines[3].contains("Empty Series"));
    assert!(lines[4].contains("Standalone Book"));
    assert!(
        lines[5].starts_with('+'),
        "Standalone row after empty group should have separator"
    );
}

#[test]
fn test_structured_table_consecutive_group_headers() {
    // Edge case: Two GroupHeaders in a row (no Data between them).
    // Each group header should render, and the second group's Data rows
    // should belong to the second group, not the first.
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        TableRow::GroupHeader("First Series".to_string(), 0),
        TableRow::GroupHeader("Second Series".to_string(), 2),
        TableRow::Data(vec!["#1 Book A".to_string(), "Author A".to_string()]),
        TableRow::Data(vec!["#2 Book B".to_string(), "Author A".to_string()]),
    ];
    let output = format_structured_table(&rows, &[]);
    let lines: Vec<&str> = output.lines().collect();

    // Structure:
    // 0: +=====+=====+
    // 1: | Title | Author |
    // 2: +=====+=====+
    // 3: | ── First Series ── |   (empty group)
    // 4: | ── Second Series ── |  (group with 2 rows)
    // 5: | #1 Book A | Author A |  (grouped, no sep)
    // 6: | #2 Book B | Author A |  (grouped, sep after)
    // 7: +-----+-----+
    assert_eq!(lines.len(), 8, "Output:\n{}", output);
    assert!(lines[3].contains("First Series"));
    assert!(lines[4].contains("Second Series"));
    assert!(lines[5].contains("#1 Book A"));
    assert!(lines[6].contains("#2 Book B"));
    assert!(
        lines[7].starts_with('+'),
        "Separator after last row in second group"
    );
    // No separator between the two grouped Data rows
    assert!(
        !lines[5].starts_with('+') && !lines[6].starts_with('+'),
        "No separator between grouped rows"
    );
}

// ============================================================
// Alignment tests
// ============================================================

#[test]
fn test_structured_table_left_aligned_text_right_aligned_dates() {
    let rows = vec![
        TableRow::Header(vec![
            "Title".to_string(),
            "Author".to_string(),
            "Finished on".to_string(),
        ]),
        TableRow::Data(vec![
            "Nullpunkt".to_string(),
            "Jørn Lier Horst".to_string(),
            "2025-04-27".to_string(),
        ]),
        TableRow::Data(vec![
            "Orbital".to_string(),
            "Samantha Harvey".to_string(),
            "2026-02-20".to_string(),
        ]),
    ];
    let alignments = vec![Alignment::Left, Alignment::Left, Alignment::Right];
    let output = format_structured_table(&rows, &alignments);
    let lines: Vec<&str> = output.lines().collect();

    // Header row: left-aligned Title and Author, right-aligned "Finished on"
    let header_line = lines[1];
    // "| Title" — starts with "| " then text immediately
    assert!(
        header_line.starts_with("| Title"),
        "Title header should be left-aligned: {:?}",
        header_line
    );

    // First data row: "| Nullpunkt" — left-aligned
    let data_line = lines[3];
    assert!(
        data_line.starts_with("| Nullpunkt"),
        "Title should be left-aligned: {:?}",
        data_line
    );

    // Date should be right-aligned: ends with "2025-04-27 |"
    assert!(
        data_line.contains("2025-04-27 |"),
        "Date should be right-aligned (ends with space+pipe): {:?}",
        data_line
    );
}

#[test]
fn test_structured_table_center_alignment() {
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Flag".to_string()]),
        TableRow::Data(vec!["Some Book".to_string(), "x".to_string()]),
    ];
    let alignments = vec![Alignment::Left, Alignment::Center];
    let output = format_structured_table(&rows, &alignments);
    let lines: Vec<&str> = output.lines().collect();

    // The "x" flag should be center-aligned (equal padding on both sides)
    let data_line = lines[3];
    // Extract the Flag cell (after the second |)
    let cells: Vec<&str> = data_line.split('|').collect();
    let flag_cell = cells[2]; // 0="" (before first |), 1=title, 2=flag
    let trimmed = flag_cell.trim();
    assert_eq!(trimmed, "x");
    // Check that padding is roughly equal on both sides
    let left_spaces = flag_cell.len() - flag_cell.trim_start().len();
    let right_spaces = flag_cell.len() - flag_cell.trim_end().len();
    assert!(
        (left_spaces as i32 - right_spaces as i32).abs() <= 1,
        "Center-aligned flag should have roughly equal padding: left={}, right={}",
        left_spaces,
        right_spaces
    );
}

#[test]
fn test_structured_table_default_alignment_is_left() {
    // When no alignments are provided, all columns should be left-aligned
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        TableRow::Data(vec!["Short".to_string(), "A".to_string()]),
    ];
    let output = format_structured_table(&rows, &[]);
    let lines: Vec<&str> = output.lines().collect();

    // Data row should be left-aligned: starts with "| Short"
    assert!(
        lines[3].starts_with("| Short"),
        "Default alignment should be left: {:?}",
        lines[3]
    );
}

#[test]
fn test_structured_table_group_header_left_aligned_with_indent() {
    let rows = vec![
        TableRow::Header(vec!["Title".to_string(), "Author".to_string()]),
        TableRow::GroupHeader("The Expanse".to_string(), 1),
        TableRow::Data(vec![
            "  #1 Leviathan Wakes".to_string(),
            "James S.A. Corey".to_string(),
        ]),
    ];
    let alignments = vec![Alignment::Left, Alignment::Left];
    let output = format_structured_table(&rows, &alignments);
    let lines: Vec<&str> = output.lines().collect();

    // Group header should be left-aligned with 2-space indent:
    // "|   ── The Expanse ──"  (1 space pad + 2 indent + decoration)
    let group_line = lines[3];
    assert!(
        group_line.starts_with("|   \u{2500}\u{2500} The Expanse"),
        "Group header should be left-aligned with indent: {:?}",
        group_line
    );
}

#[test]
fn test_format_table_with_alignments() {
    // The legacy format_table should also support alignments
    let rows = vec![
        vec!["Title".to_string(), "Date".to_string()],
        vec!["My Book".to_string(), "2025-01-01".to_string()],
    ];
    let alignments = vec![Alignment::Left, Alignment::Right];
    let output = format_table(&rows, &alignments);
    let lines: Vec<&str> = output.lines().collect();

    // Title left-aligned
    assert!(
        lines[1].starts_with("| Title"),
        "Title should be left-aligned: {:?}",
        lines[1]
    );
    // Date right-aligned: ends with "Date |" or "2025-01-01 |"
    assert!(
        lines[3].contains("2025-01-01 |"),
        "Date should be right-aligned: {:?}",
        lines[3]
    );
}
