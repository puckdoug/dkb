use std::path::Path;

use dkb_core::cli_state::CliState;
use dkb_core::item::Item;
use dkb_core::storage::Storage;
use uuid::Uuid;

use crate::select::{parse_selection, resolve_selection};

#[allow(clippy::missing_errors_doc)]
pub fn run_pick(data_dir: &Path, arg: &str) -> std::io::Result<Uuid> {
    let board = Storage::load_board(data_dir)?;
    let mut state = CliState::load(data_dir);
    let sel = parse_selection(arg);
    let id = resolve_selection(&sel, &board, &state, data_dir)
        .map_err(std::io::Error::other)?;

    state.set_current(id);
    state.save(data_dir)?;

    let item = board.find_item(&id);
    let title = item.map(Item::title).unwrap_or_default();
    println!("picked: {title} ({id})");
    Ok(id)
}
