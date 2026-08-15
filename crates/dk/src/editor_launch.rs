use std::path::Path;
use std::process::Command;

/// Returns the configured editor command string.
///
/// Resolves `$VISUAL`, falling back to `$EDITOR`, then `"vi"`.
#[must_use]
pub fn resolve_editor() -> String {
    if let Some(v) = std::env::var_os("VISUAL") {
        return v.to_string_lossy().into_owned();
    }
    if let Some(e) = std::env::var_os("EDITOR") {
        return e.to_string_lossy().into_owned();
    }
    "vi".to_string()
}

/// Launches the resolved editor on `path`, positioning the cursor at `cursor_line:cursor_col`.
///
/// # Errors
///
/// Returns an error if the editor process cannot be spawned or exits with a non-zero status.
pub fn launch_editor(path: &Path, cursor_line: usize, cursor_col: usize) -> std::io::Result<()> {
    let editor = resolve_editor();
    let mut parts = editor.split_whitespace();
    let binary = parts.next().unwrap_or("vi");
    let mut cmd = Command::new(binary);
    for extra in parts {
        cmd.arg(extra);
    }
    cmd.arg(format!("+{cursor_line}:{cursor_col}"));
    cmd.arg(path);
    let status = cmd.status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "editor exited with code {:?}",
            status.code()
        )));
    }
    Ok(())
}
