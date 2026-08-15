#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::Item;
use dkb_core::storage::{Location, Storage};
use dk::commands::pick::run_pick;
use tempfile::TempDir;

#[test]
fn test_pick_by_index() {
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let i1 = Item::new("first");
    let i2 = Item::new("second");
    Storage::write_item(dir.path(), &i1, &Location::Active(dkb_core::item::Category::Today)).unwrap();
    Storage::write_item(dir.path(), &i2, &Location::Active(dkb_core::item::Category::Today)).unwrap();
    let mut state = CliState::default();
    state.set_last_list(vec![i1.id, i2.id]);
    state.save(dir.path()).unwrap();

    let picked = run_pick(dir.path(), "1").unwrap();
    assert_eq!(picked, i2.id);
    let loaded = CliState::load(dir.path());
    assert_eq!(loaded.current, Some(i2.id));
}

#[test]
fn test_pick_by_filename() {
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let item = Item::new("a task");
    Storage::write_item(dir.path(), &item, &Location::Backlog).unwrap();
    let arg = format!("{}.md", item.id);
    let picked = run_pick(dir.path(), &arg).unwrap();
    assert_eq!(picked, item.id);
}

#[test]
fn test_pick_out_of_range_fails() {
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let state = CliState::default();
    state.save(dir.path()).unwrap();
    let result = run_pick(dir.path(), "0");
    assert!(result.is_err());
}
