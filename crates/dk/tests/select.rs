#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::{Category, Item};
use dkb_core::storage::{Location, Storage};
use dk::select::{parse_selection, resolve_selection, Selection};
use tempfile::TempDir;
use uuid::Uuid;

fn seed_item(dir: &std::path::Path, loc: &Location, title: &str) -> Uuid {
    Storage::init(dir).unwrap_or(());
    let item = Item::new(title);
    Storage::write_item(dir, &item, loc).unwrap();
    item.id
}

#[test]
fn test_parse_empty_is_current() {
    assert!(matches!(parse_selection(""), Selection::Current));
}

#[test]
fn test_parse_number() {
    assert!(matches!(parse_selection("3"), Selection::Index(n) if n == 3));
}

#[test]
fn test_parse_category_index() {
    let s = parse_selection("yesterday/1");
    match s {
        Selection::CategoryIndex(loc, n) => {
            assert_eq!(loc, Location::Active(Category::Yesterday));
            assert_eq!(n, 1);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_parse_uuid_file() {
    let id = Uuid::new_v4();
    let s = parse_selection(&format!("{id}.md"));
    assert!(matches!(s, Selection::File(u) if u == id));
}

#[test]
fn test_resolve_current() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let id = seed_item(&data_dir, &Location::Active(Category::Today), "task");
    let board = dkb_core::storage::Storage::load_board(&data_dir).unwrap();
    let mut state = CliState::default();
    state.set_current(id);
    let resolved = resolve_selection(&Selection::Current, &board, &state, &data_dir);
    assert_eq!(resolved.unwrap(), id);
}

#[test]
fn test_resolve_index() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let id1 = seed_item(&data_dir, &Location::Active(Category::Today), "a");
    let id2 = seed_item(&data_dir, &Location::Active(Category::Today), "b");
    let board = dkb_core::storage::Storage::load_board(&data_dir).unwrap();
    let mut state = CliState::default();
    state.set_last_list(vec![id1, id2]);
    let r0 = resolve_selection(&Selection::Index(0), &board, &state, &data_dir);
    let r1 = resolve_selection(&Selection::Index(1), &board, &state, &data_dir);
    assert_eq!(r0.unwrap(), id1);
    assert_eq!(r1.unwrap(), id2);
}

#[test]
fn test_resolve_index_out_of_range() {
    let state = CliState::default();
    let board = dkb_core::board::Board::default();
    let dir = TempDir::new().unwrap();
    let r = resolve_selection(&Selection::Index(0), &board, &state, dir.path());
    assert!(r.is_err());
}

#[test]
fn test_resolve_category_index() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let id1 = seed_item(&data_dir, &Location::Backlog, "first");
    let id2 = seed_item(&data_dir, &Location::Backlog, "second");

    let mut board = dkb_core::board::Board::default();
    board.backlog.push(Storage::read_item(&data_dir, &id1, &Location::Backlog).unwrap());
    board.backlog.push(Storage::read_item(&data_dir, &id2, &Location::Backlog).unwrap());
    Storage::save_board_state(&data_dir, &board).unwrap();

    let board = dkb_core::storage::Storage::load_board(&data_dir).unwrap();
    let state = CliState::default();
    let r = resolve_selection(
        &Selection::CategoryIndex(Location::Backlog, 0),
        &board,
        &state,
        &data_dir,
    );
    assert_eq!(r.unwrap(), id1);
}

#[test]
fn test_resolve_file() {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    Storage::init(&data_dir).unwrap();
    let id = seed_item(&data_dir, &Location::Done, "done task");
    let board = dkb_core::storage::Storage::load_board(&data_dir).unwrap();
    let state = CliState::default();
    let r = resolve_selection(&Selection::File(id), &board, &state, &data_dir);
    assert_eq!(r.unwrap(), id);
}
