use std::io::Write;
use tempfile::NamedTempFile;

/// Strips comment lines (starting with #) and trims whitespace from editor text.
/// Returns None if the resulting text is empty (indicating the user aborted).
pub fn strip_editor_text(text: &str) -> Option<String> {
    let stripped: String = text
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string();

    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

/// Opens the user's default editor on a temp file pre-populated with `template`.
///
/// The editor is determined by checking $EDITOR, then $VISUAL, falling back to "vi".
/// Returns the edited text with comment lines stripped, or None if the result is
/// empty (the user aborted).
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
