use std::path::Path;

use dkb_core::cli_state::CliState;
use dkb_core::storage::Storage;

use crate::select::{parse_selection, resolve_selection};

#[allow(clippy::missing_errors_doc)]
pub fn run_path(data_dir: &Path, args: &[String]) -> std::io::Result<()> {
    let board = Storage::load_board(data_dir)?;
    let state = CliState::load(data_dir);

    let id = if args.is_empty() {
        state
            .current
            .ok_or_else(|| std::io::Error::other("no current item set"))?
    } else {
        let sel = parse_selection(&args[0]);
        resolve_selection(&sel, &board, &state, data_dir).map_err(std::io::Error::other)?
    };

    let location = board
        .find_item_location(&id)
        .ok_or_else(|| std::io::Error::other(format!("item {id} has no location")))?;

    let path = data_dir.join(location.to_path()).join(format!("{id}.md"));
    println!("{}", path.display());
    Ok(())
}
