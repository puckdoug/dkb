use std::path::Path;

use dkb_core::cli_state::CliState;
use dkb_core::item::Item;
use dkb_core::storage::Storage;

use crate::category::parse_category;
use crate::select::{parse_selection, resolve_selection};

#[allow(clippy::missing_errors_doc)]
pub fn run_move(data_dir: &Path, selection_arg: &str, dest_arg: &str) -> std::io::Result<()> {
    let dest = parse_category(dest_arg)
        .ok_or_else(|| std::io::Error::other(format!("unknown category: {dest_arg}")))?;

    let board = Storage::load_board(data_dir)?;
    let state = CliState::load(data_dir);
    let sel = parse_selection(selection_arg);
    let id = resolve_selection(&sel, &board, &state, data_dir)
        .map_err(std::io::Error::other)?;

    let from = board
        .find_item_location(&id)
        .ok_or_else(|| std::io::Error::other(format!("item {id} has no location")))?;

    Storage::move_item(data_dir, &id, &from, &dest)?;

    let title = board.find_item(&id).map(Item::title).unwrap_or_default();
    println!("moved: {title} -> {}", dest.display_name());
    Ok(())
}
