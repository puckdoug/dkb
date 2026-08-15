use std::io::Write;
use std::path::Path;

use dkb_core::cli_state::CliState;
use dkb_core::item::Item;
use dkb_core::storage::{Location, Storage};
use uuid::Uuid;

use crate::category::parse_category;
use crate::editor_launch::launch_editor;

#[must_use]
pub fn build_item_from_body(id: Uuid, body: &str) -> Item {
    let now = chrono::Utc::now();
    Item {
        id,
        body: body.to_string(),
        created_at: now,
        updated_at: now,
        completed_at: None,
    }
}

#[must_use]
pub fn is_empty_body(body: &str) -> bool {
    let trimmed = body.trim();
    trimmed.is_empty() || trimmed == "#"
}

/// Creates a new kanban item by launching the user's editor on a seeded temp file.
///
/// The editor is opened with a single `# ` line; the edited contents become the
/// item body. If the body is empty (or only `#`), the operation aborts. The new
/// item is written to `data_dir` at the resolved `Location` and marked current in
/// `cli_state.json`.
///
/// # Errors
///
/// Returns an error if the temp file cannot be created, the editor cannot be
/// launched or exits non-zero, the edited file cannot be read back, the body is
/// empty, or persisting the item or CLI state fails.
pub fn run_new(data_dir: &Path, category_arg: Option<&str>) -> std::io::Result<Uuid> {
    let location = category_arg
        .and_then(parse_category)
        .unwrap_or(Location::Backlog);

    let id = Uuid::new_v4();
    let mut tmp = tempfile::NamedTempFile::new()?;
    writeln!(tmp, "# ")?;
    tmp.flush()?;
    let tmp_path = tmp.path().to_path_buf();

    launch_editor(&tmp_path, 1, 3)?;

    let body = std::fs::read_to_string(&tmp_path)?;
    drop(tmp);

    if is_empty_body(&body) {
        return Err(std::io::Error::other(
            "editor returned empty content; aborting",
        ));
    }

    let item = build_item_from_body(id, &body);
    Storage::write_item(data_dir, &item, &location)?;

    let mut state = CliState::load(data_dir);
    state.set_current(id);
    state.save(data_dir)?;

    Ok(id)
}
