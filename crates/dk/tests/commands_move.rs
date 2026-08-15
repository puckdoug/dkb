#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::{Category, Item};
use dkb_core::storage::{Location, Storage};
use dk::commands::move_cmd::run_move;
use tempfile::TempDir;

#[test]
fn test_move_by_index() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let item = Item::new("# move me");
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();

    let mut state = CliState::default();
    state.set_last_list(vec![item.id]);
    state.save(&data_dir).unwrap();

    run_move(&data_dir, "0", "today").unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    assert!(board.active.today.iter().any(|i| i.id == item.id));
    assert!(!board.backlog.iter().any(|i| i.id == item.id));
}

#[test]
fn test_move_to_done_sets_completed() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let item = Item::new("# finish me");
    Storage::write_item(&data_dir, &item, &Location::Active(Category::Today)).unwrap();

    let mut state = CliState::default();
    state.set_last_list(vec![item.id]);
    state.save(&data_dir).unwrap();

    run_move(&data_dir, "0", "done").unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    let moved = board.done.iter().find(|i| i.id == item.id).unwrap();
    assert!(moved.completed_at.is_some());
}
