use unicode_width::UnicodeWidthStr;

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
