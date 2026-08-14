use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ViewerPreference {
    #[default]
    Auto,
    Custom(PathBuf),
}

const CANDIDATE_APPS: &[&str] = &["Marked.app", "Marked 2.app", "MD-Viewer.app"];

#[must_use]
pub fn candidate_viewer_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    for app in CANDIDATE_APPS {
        paths.push(PathBuf::from("/Applications").join(app));
        if let Some(ref home_dir) = home {
            paths.push(home_dir.join("Applications").join(app));
        }
    }
    paths
}

#[must_use]
pub fn detect_default_viewer() -> Option<PathBuf> {
    candidate_viewer_paths().into_iter().find(|p| p.exists())
}

#[must_use]
pub fn resolve_viewer_path(pref: &ViewerPreference) -> Option<PathBuf> {
    match pref {
        ViewerPreference::Auto => detect_default_viewer(),
        ViewerPreference::Custom(path) => Some(path.clone()),
    }
}

#[must_use]
pub fn build_open_command_with_app(file_path: &Path, app_path: Option<&Path>) -> Command {
    let mut cmd = Command::new("open");
    if let Some(app) = app_path {
        cmd.arg("-a").arg(app);
    }
    cmd.arg(file_path);
    cmd
}

#[must_use]
pub fn build_open_command(file_path: &Path, pref: &ViewerPreference) -> Command {
    let resolved = resolve_viewer_path(pref);
    build_open_command_with_app(file_path, resolved.as_deref())
}

/// Opens the given markdown file in the external viewer or system default handler.
///
/// # Errors
/// Returns an `std::io::Error` if spawning the command fails or if the command exits with non-zero status.
pub fn open_in_viewer(file_path: &Path, pref: &ViewerPreference) -> std::io::Result<()> {
    let mut cmd = build_open_command(file_path, pref);
    let status = cmd.status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "open command failed with exit code {:?}",
            status.code()
        )));
    }
    Ok(())
}

#[must_use]
pub fn pick_viewer_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Applications", &["app"])
        .pick_file()
}
