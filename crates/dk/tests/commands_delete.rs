#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::{Category, Item};
use dkb_core::storage::{Location, Storage};
use dk::commands::delete::{delete_single, run_delete};
use tempfile::TempDir;

#[test]
fn test_delete_force() {
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let item = Item::new("# to delete");
    Storage::write_item(dir.path(), &item, &Location::Backlog).unwrap();

    delete_single(dir.path(), item.id, true).unwrap();

    let board = Storage::load_board(dir.path()).unwrap();
    assert!(!board.backlog.iter().any(|i| i.id == item.id));
}

#[test]
fn test_delete_clears_current() {
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let item = Item::new("# current");
    Storage::write_item(dir.path(), &item, &Location::Active(Category::Today)).unwrap();
    let mut state = CliState::default();
    state.set_current(item.id);
    state.save(dir.path()).unwrap();

    delete_single(dir.path(), item.id, true).unwrap();

    let loaded = CliState::load(dir.path());
    assert!(loaded.current.is_none());
}

#[test]
fn test_delete_by_index_force() {
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let item = Item::new("# indexed");
    Storage::write_item(dir.path(), &item, &Location::Backlog).unwrap();
    let mut state = CliState::default();
    state.set_last_list(vec![item.id]);
    state.save(dir.path()).unwrap();

    run_delete(dir.path(), &["0".to_string()], true).unwrap();

    let board = Storage::load_board(dir.path()).unwrap();
    assert!(board.backlog.is_empty());
}
