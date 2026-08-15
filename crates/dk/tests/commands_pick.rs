#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::Item;
use dkb_core::storage::{Location, Storage};
use dk::commands::pick::run_pick;
use tempfile::TempDir;

#[test]
fn test_pick_by_index() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let i1 = Item::new("first");
    let i2 = Item::new("second");
    Storage::write_item(&data_dir, &i1, &Location::Active(dkb_core::item::Category::Today)).unwrap();
    Storage::write_item(&data_dir, &i2, &Location::Active(dkb_core::item::Category::Today)).unwrap();
    let mut state = CliState::default();
    state.set_last_list(vec![i1.id, i2.id]);
    state.save(&data_dir).unwrap();

    let picked = run_pick(&data_dir, "1").unwrap();
    assert_eq!(picked, i2.id);
    let loaded = CliState::load(&data_dir);
    assert_eq!(loaded.current, Some(i2.id));
}

#[test]
fn test_pick_by_filename() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let item = Item::new("a task");
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();
    let arg = format!("{}.md", item.id);
    let picked = run_pick(&data_dir, &arg).unwrap();
    assert_eq!(picked, item.id);
}

#[test]
fn test_pick_out_of_range_fails() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let state = CliState::default();
    state.save(&data_dir).unwrap();
    let result = run_pick(&data_dir, "0");
    assert!(result.is_err());
}
