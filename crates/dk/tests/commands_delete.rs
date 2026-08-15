#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::{Category, Item};
use dkb_core::storage::{Location, Storage};
use dk::commands::delete::{delete_single, run_delete};
use tempfile::TempDir;

#[test]
fn test_delete() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let item = Item::new("# to delete");
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();

    delete_single(&data_dir, item.id).unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    assert!(!board.backlog.iter().any(|i| i.id == item.id));
}

#[test]
fn test_delete_clears_current() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let item = Item::new("# current");
    Storage::write_item(&data_dir, &item, &Location::Active(Category::Today)).unwrap();
    let mut state = CliState::default();
    state.set_current(item.id);
    state.save(&data_dir).unwrap();

    delete_single(&data_dir, item.id).unwrap();

    let loaded = CliState::load(&data_dir);
    assert!(loaded.current.is_none());
}

#[test]
fn test_delete_by_index() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let item = Item::new("# indexed");
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();
    let mut state = CliState::default();
    state.set_last_list(vec![item.id]);
    state.save(&data_dir).unwrap();

    run_delete(&data_dir, &["0".to_string()]).unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    assert!(board.backlog.is_empty());
}
