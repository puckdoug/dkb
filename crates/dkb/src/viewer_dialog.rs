use std::path::PathBuf;

#[must_use]
pub fn pick_viewer_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Applications", &["app"])
        .pick_file()
}
