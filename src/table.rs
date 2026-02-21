use unicode_width::UnicodeWidthStr;

/// A row in a structured table that supports series grouping.
///
/// - `Header`: Column headers (rendered with thick `=` separators).
/// - `Data`: A normal data row (rendered with thin `-` separator after it,
///   unless it's followed by another `Data` row in the same group).
/// - `GroupHeader`: A spanning row showing the series name centered across
///   the full table width. Starts a new group — subsequent `Data` rows are
///   part of this group until the next `GroupHeader` or end of rows.
#[derive(Debug, Clone)]
pub enum TableRow {
    /// Column header row (first row in the table).
    Header(Vec<String>),
    /// A regular data row.
    Data(Vec<String>),
    /// A group header that spans the full table width (e.g. series name).
    /// The `usize` is the number of `Data` rows that belong to this group
    /// (i.e. the count of immediately following `Data` rows that should
    /// have no separators between them).
    GroupHeader(String, usize),
}

/// Formats a structured table with support for group headers.
///
/// Group headers span the full table width as a centered label.
/// The `usize` in `GroupHeader(label, count)` specifies exactly how many
/// subsequent `Data` rows belong to the group. Only those rows have their
/// inter-row separators suppressed. Data rows outside any group get normal
/// separators.
///
/// The first row must be a `Header` variant.
pub fn format_structured_table(rows: &[TableRow]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    // Extract header to determine column count and widths
    let header = match &rows[0] {
        TableRow::Header(h) => h,
        _ => return String::new(), // first row must be Header
    };
    let col_count = header.len();

    // Compute max display width per column across header and all Data rows
    let mut col_widths: Vec<usize> = vec![0; col_count];
    for row in rows {
        let cells = match row {
            TableRow::Header(cells) | TableRow::Data(cells) => cells,
            TableRow::GroupHeader(_, _) => continue,
        };
        debug_assert!(
            cells.len() == col_count,
            "All rows must have the same number of columns as the header (expected {}, got {})",
            col_count,
            cells.len()
        );
        for (i, cell) in cells.iter().enumerate() {
            let display_width = UnicodeWidthStr::width(cell.as_str()) + 2; // 1 space padding each side
            if display_width > col_widths[i] {
                col_widths[i] = display_width;
            }
        }
    }

    let total_width = col_widths.iter().sum::<usize>() + col_count + 1; // +1 for each `|` and outer `|`

    let mut output = String::new();

    // Header separator (thick)
    output.push_str(&draw_line(&col_widths, '='));
    output.push('\n');

    // Header row
    output.push_str(&format_row(header, &col_widths));
    output.push('\n');

    // Header separator (thick)
    output.push_str(&draw_line(&col_widths, '='));
    output.push('\n');

    // Track how many grouped Data rows remain in the current group.
    // 0 means we're not inside a group (next Data row is standalone).
    let mut group_rows_remaining: usize = 0;

    for row in rows.iter().skip(1) {
        match row {
            TableRow::Header(_) => {
                // Ignore extra headers (shouldn't happen, but be defensive)
            }
            TableRow::GroupHeader(label, count) => {
                // Start a new group with the specified number of Data rows
                group_rows_remaining = *count;
                output.push_str(&format_group_header(label, total_width));
                output.push('\n');
            }
            TableRow::Data(cells) => {
                output.push_str(&format_row(cells, &col_widths));
                output.push('\n');

                if group_rows_remaining > 0 {
                    group_rows_remaining -= 1;
                    // Suppress separator if more grouped rows follow
                    if group_rows_remaining > 0 {
                        continue; // no separator — next row is still in this group
                    }
                }

                // Separator after: standalone row, or last row in a group
                output.push_str(&draw_line(&col_widths, '-'));
                output.push('\n');
            }
        }
    }

    output
}

/// Prints a structured table with group support to stdout.
pub fn print_structured_table(rows: &[TableRow]) {
    print!("{}", format_structured_table(rows));
}

/// Formats a 2-D vector of strings as a pretty-printed table string.
///
/// The first row is treated as the header and separated by `=` lines.
/// Subsequent rows are separated by `-` lines.
/// Column widths are computed using Unicode display width so that
/// multi-byte characters (e.g. æ, ø, å, emoji) align correctly.
pub fn format_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let col_count = rows[0].len();

    // Compute max display width per column (with 1 space padding on each side)
    let mut col_widths: Vec<usize> = vec![0; col_count];
    for row in rows {
        debug_assert!(
            row.len() == col_count,
            "All rows must have the same number of columns as the header (expected {}, got {})",
            col_count,
            row.len()
        );
        for (i, cell) in row.iter().enumerate() {
            let display_width = UnicodeWidthStr::width(cell.as_str()) + 2; // 1 space padding each side
            if display_width > col_widths[i] {
                col_widths[i] = display_width;
            }
        }
    }

    let mut output = String::new();

    // Header separator (thick)
    output.push_str(&draw_line(&col_widths, '='));
    output.push('\n');

    // Header row
    output.push_str(&format_row(&rows[0], &col_widths));
    output.push('\n');

    // Header separator (thick)
    output.push_str(&draw_line(&col_widths, '='));
    output.push('\n');

    // Data rows
    for row in &rows[1..] {
        output.push_str(&format_row(row, &col_widths));
        output.push('\n');
        output.push_str(&draw_line(&col_widths, '-'));
        output.push('\n');
    }

    output
}

/// Prints a 2-D vector of strings as a pretty-printed table to stdout.
pub fn print_table(rows: &[Vec<String>]) {
    print!("{}", format_table(rows));
}

/// Formats a group header label centered across the full table width.
///
/// Rendered as `| ── Label ── |` with `──` decorations flanking the label
/// to visually distinguish it from data rows.
fn format_group_header(label: &str, total_width: usize) -> String {
    // Inner width is total_width minus the outer `|` characters (2)
    let inner_width = total_width.saturating_sub(2);

    // Build decorated label: "── Label ──"
    let decorated = format!("\u{2500}\u{2500} {} \u{2500}\u{2500}", label);
    let display_width = UnicodeWidthStr::width(decorated.as_str());
    let total_padding = inner_width.saturating_sub(display_width);
    let left_pad = total_padding / 2;
    let right_pad = total_padding - left_pad;
    format!(
        "|{}{}{}|",
        " ".repeat(left_pad),
        decorated,
        " ".repeat(right_pad)
    )
}

fn draw_line(col_widths: &[usize], ch: char) -> String {
    let mut line = String::from("+");
    for &width in col_widths {
        for _ in 0..width {
            line.push(ch);
        }
        line.push('+');
    }
    line
}

fn format_row(row: &[String], col_widths: &[usize]) -> String {
    let mut result = String::from("|");
    for (cell, &col_width) in row.iter().zip(col_widths.iter()) {
        let display_width = UnicodeWidthStr::width(cell.as_str());
        let total_padding = col_width - display_width;
        let left_pad = total_padding / 2;
        let right_pad = total_padding - left_pad;
        result.push_str(&" ".repeat(left_pad));
        result.push_str(cell);
        result.push_str(&" ".repeat(right_pad));
        result.push('|');
    }
    result
}
