use std::io::Write;
use std::path::Path;

use dkb_core::cli_state::CliState;
use dkb_core::storage::Storage;
use uuid::Uuid;

use crate::editor_launch::launch_editor;
use crate::select::{parse_selection, resolve_selection};

#[allow(clippy::missing_errors_doc)]
pub fn edit_single(data_dir: &Path, id: Uuid) -> std::io::Result<()> {
    let board = Storage::load_board(data_dir)?;
    let item = board
        .find_item(&id)
        .ok_or_else(|| std::io::Error::other(format!("item {id} not found")))?
        .clone();
    let location = board
        .find_item_location(&id)
        .ok_or_else(|| std::io::Error::other(format!("item {id} has no location")))?;

    let mut tmp = tempfile::NamedTempFile::new()?;
    tmp.write_all(item.body.as_bytes())?;
    tmp.flush()?;
    let tmp_path = tmp.path().to_path_buf();

    launch_editor(&tmp_path, 1, 1)?;

    let new_body = std::fs::read_to_string(&tmp_path)?;
    drop(tmp);

    let mut updated = item;
    updated.body = new_body;
    updated.updated_at = chrono::Utc::now();
    Storage::write_item(data_dir, &updated, &location)?;
    Ok(())
}

#[allow(clippy::missing_errors_doc)]
pub fn run_edit(data_dir: &Path, args: &[String]) -> std::io::Result<()> {
    if args.is_empty() {
        let state = CliState::load(data_dir);
        let id = state
            .current
            .ok_or_else(|| std::io::Error::other("no current item set"))?;
        return edit_single(data_dir, id);
    }

    for arg in args {
        let board = Storage::load_board(data_dir)?;
        let state = CliState::load(data_dir);
        let sel = parse_selection(arg);
        let id = resolve_selection(&sel, &board, &state, data_dir)
            .map_err(std::io::Error::other)?;
        edit_single(data_dir, id)?;
    }
    Ok(())
}
