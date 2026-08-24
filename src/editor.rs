use std::io::Write;
use tempfile::NamedTempFile;

/// Marks the start of the instruction block. Everything from this line down is
/// discarded, which lets the body keep lines that begin with `#` — a review or
/// a note may legitimately use markdown headings.
pub const SCISSORS: &str = "# ------------------------ >8 ------------------------";

/// Builds an instruction block: the scissors line followed by each line
/// commented out. Placed at the end of an editor template.
pub fn instruction_block(lines: &[&str]) -> String {
    let mut block = String::from(SCISSORS);
    for line in lines {
        block.push_str("\n# ");
        block.push_str(line);
    }
    block.push('\n');
    block
}

/// Keeps everything above the LAST scissors line, trimmed.
///
/// The separator is always the last occurrence, not the first: a template
/// appends its instruction block after the body, so the real separator is
/// whichever scissors line comes last. A user editing a review whose own
/// text happens to contain a line that looks like the scissors marker must
/// not have their trailing text truncated at that line.
///
/// Line endings are normalized to `\n`: an editor that writes CRLF (common on
/// Windows, and configurable elsewhere) must never leave `\r\n` in stored
/// text — a stray `\r` would otherwise survive into the JSON and show up on
/// every rendered diff line. This does not strip `#`-prefixed lines; the
/// scissors behaviour is unchanged.
///
/// Returns None if nothing is left, which is how the user aborts: saving an
/// untouched template leaves an empty body.
pub fn strip_editor_text(text: &str) -> Option<String> {
    let body = match text.rsplit_once(SCISSORS) {
        Some((above, _)) => above,
        None => text,
    };

    let normalized = body.replace("\r\n", "\n");
    let trimmed = normalized.trim().to_string();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Opens the user's default editor on a temp file pre-populated with `template`.
///
/// The editor is determined by checking $EDITOR, then $VISUAL, falling back to "vi".
/// Returns everything above the last scissors line (see [`strip_editor_text`]),
/// or None if the remaining body is empty (the user aborted).
pub fn get_text_from_editor(template: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut tmp = NamedTempFile::new()?;
    write!(tmp, "{}", template)?;
    tmp.flush()?;

    let path = tmp.path().to_path_buf();

    // Split editor command to support values like "code --wait" or "subl -w"
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (editor_bin, editor_args) = parts
        .split_first()
        .ok_or("$EDITOR is empty after splitting")?;

    let status = std::process::Command::new(editor_bin)
        .args(editor_args)
        .arg(&path)
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(format!("Editor '{}' exited with non-zero status", editor).into());
    }

    let content = std::fs::read_to_string(&path)?;
    Ok(strip_editor_text(&content))
}
