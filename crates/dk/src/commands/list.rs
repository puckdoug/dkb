use std::path::Path;

use dkb_core::board::Board;
use dkb_core::cli_state::CliState;
use dkb_core::item::{Category, Item};
use dkb_core::storage::{Location, Storage};
use uuid::Uuid;

use crate::category::parse_category;
use crate::terminal::{format_list_line, terminal_width};

#[must_use]
pub fn active_locations() -> [Location; 4] {
    [
        Location::Active(Category::Yesterday),
        Location::Active(Category::Today),
        Location::Active(Category::ThisWeek),
        Location::Active(Category::NextWeek),
    ]
}

#[must_use]
pub fn collect_items(board: &Board, loc: &Location) -> Vec<Item> {
    match loc {
        Location::Backlog => board.backlog.clone(),
        Location::Active(Category::Yesterday) => board.active.yesterday.clone(),
        Location::Active(Category::Today) => board.active.today.clone(),
        Location::Active(Category::ThisWeek) => board.active.this_week.clone(),
        Location::Active(Category::NextWeek) => board.active.next_week.clone(),
        Location::Done => board.done.clone(),
    }
}

#[must_use]
pub fn render_list(items: &[Item], current: Option<Uuid>, width: usize) -> String {
    let max_index = items.len().saturating_sub(1);
    let index_col_width = max_index.to_string().len();
    let mut lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let is_current = current == Some(item.id);
        let title = item.title();
        let line = format_list_line(i, is_current, &title, index_col_width, width);
        lines.push(line);
    }
    lines.join("\n")
}

/// Runs the `dk list` command, printing items and persisting the list order.
///
/// # Errors
///
/// Returns `io::Error` if the board cannot be loaded or `cli_state.json`
/// cannot be saved.
pub fn run_list(data_dir: &Path, category_arg: Option<&str>) -> std::io::Result<()> {
    let board = Storage::load_board(data_dir)?;
    let width = terminal_width();

    let (items, ids): (Vec<Item>, Vec<Uuid>) = if let Some(s) = category_arg {
        let loc = parse_category(s)
            .ok_or_else(|| std::io::Error::other(format!("unknown category: {s}")))?;
        let items = collect_items(&board, &loc);
        let ids = items.iter().map(|i| i.id).collect();
        (items, ids)
    } else {
        let mut items = Vec::new();
        for loc in active_locations() {
            items.extend(collect_items(&board, &loc));
        }
        let ids = items.iter().map(|i| i.id).collect();
        (items, ids)
    };

    let mut state = CliState::load(data_dir);
    let current = state.current;
    let output = render_list(&items, current, width);
    if output.is_empty() {
        println!("(no items)");
    } else {
        println!("{output}");
    }
    state.set_last_list(ids);
    state.save(data_dir)?;
    Ok(())
}
