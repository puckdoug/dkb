use std::io::Write;
use std::path::Path;

use dkb_core::cli_state::CliState;
use dkb_core::storage::Storage;
use uuid::Uuid;

use crate::select::{parse_selection, resolve_selection};

#[allow(clippy::missing_errors_doc)]
pub fn delete_single(data_dir: &Path, id: Uuid, force: bool) -> std::io::Result<()> {
    let board = Storage::load_board(data_dir)?;
    let item = board
        .find_item(&id)
        .ok_or_else(|| std::io::Error::other(format!("item {id} not found")))?
        .clone();
    let location = board
        .find_item_location(&id)
        .ok_or_else(|| std::io::Error::other(format!("item {id} has no location")))?;

    if !force {
        eprint!("delete \"{}\"? [y/N] ", item.title());
        std::io::stderr().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.starts_with('y') && !input.starts_with('Y') {
            println!("aborted");
            return Ok(());
        }
    }

    Storage::delete_item(data_dir, &id, &location)?;

    let mut state = CliState::load(data_dir);
    if state.current == Some(id) {
        state.clear_current();
        state.save(data_dir)?;
    }
    println!("deleted: {}", item.title());
    Ok(())
}

#[allow(clippy::missing_errors_doc)]
pub fn run_delete(data_dir: &Path, args: &[String], force: bool) -> std::io::Result<()> {
    if args.is_empty() {
        let state = CliState::load(data_dir);
        let id = state
            .current
            .ok_or_else(|| std::io::Error::other("no current item set"))?;
        return delete_single(data_dir, id, force);
    }

    for arg in args {
        let board = Storage::load_board(data_dir)?;
        let state = CliState::load(data_dir);
        let sel = parse_selection(arg);
        let id = resolve_selection(&sel, &board, &state, data_dir)
            .map_err(std::io::Error::other)?;
        delete_single(data_dir, id, force)?;
    }
    Ok(())
}
