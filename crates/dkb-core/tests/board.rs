#![allow(clippy::pedantic)]

use dkb_core::board::{Board, BoardState};
use dkb_core::item::{Category, Item};
use dkb_core::storage::{Location, Storage};
use tempfile::TempDir;

fn make_board_with_items() -> (TempDir, Board) {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let backlog_item = Item::new("Backlog task");
    let today_item = Item::new("Today task");
    let done_item = {
        let mut i = Item::new("Done task");
        i.completed_at = Some(chrono::Utc::now());
        i
    };

    Storage::write_item(&data_dir, &backlog_item, &Location::Backlog).unwrap();
    Storage::write_item(&data_dir, &today_item, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(&data_dir, &done_item, &Location::Done).unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    (tmp, board)
}

#[test]
fn test_find_item_in_backlog() {
    let (_tmp, board) = make_board_with_items();
    let id = board.backlog[0].id;
    let found = board.find_item(&id);
    assert!(found.is_some());
    assert_eq!(found.unwrap().title(), "Backlog task");
}

#[test]
fn test_find_item_location() {
    let (_tmp, board) = make_board_with_items();
    let backlog_id = board.backlog[0].id;
    let today_id = board.active.today[0].id;
    let done_id = board.done[0].id;

    assert_eq!(board.find_item_location(&backlog_id), Some(Location::Backlog));
    assert_eq!(board.find_item_location(&today_id), Some(Location::Active(Category::Today)));
    assert_eq!(board.find_item_location(&done_id), Some(Location::Done));
}

#[test]
fn test_can_move_backlog_to_active() {
    let (_tmp, board) = make_board_with_items();
    let id = board.backlog[0].id;
    assert!(board.can_move(&id, &Location::Active(Category::Today)));
}

#[test]
fn test_can_move_active_to_done() {
    let (_tmp, board) = make_board_with_items();
    let id = board.active.today[0].id;
    assert!(board.can_move(&id, &Location::Done));
}

#[test]
fn test_can_move_done_to_active() {
    let (_tmp, board) = make_board_with_items();
    let id = board.done[0].id;
    assert!(board.can_move(&id, &Location::Active(Category::Today)));
}

#[test]
fn test_can_move_done_to_backlog() {
    let (_tmp, board) = make_board_with_items();
    let id = board.done[0].id;
    assert!(board.can_move(&id, &Location::Backlog));
}

#[test]
fn test_can_move_active_to_backlog() {
    let (_tmp, board) = make_board_with_items();
    let id = board.active.today[0].id;
    assert!(board.can_move(&id, &Location::Backlog));
}

#[test]
fn test_cannot_move_backlog_to_done() {
    let (_tmp, board) = make_board_with_items();
    let id = board.backlog[0].id;
    assert!(!board.can_move(&id, &Location::Done));
}

#[test]
fn test_move_item_updates_board_state() {
    let (_tmp, mut board) = make_board_with_items();
    let id = board.backlog[0].id;
    let from_count = board.backlog.len();

    let result = board.move_item(&id, &Location::Active(Category::Today));
    assert!(result.is_some());
    let (from, to) = result.unwrap();
    assert_eq!(from, Location::Backlog);
    assert_eq!(to, Location::Active(Category::Today));

    assert_eq!(board.backlog.len(), from_count - 1);
    assert_eq!(board.active.today.iter().filter(|i| i.id == id).count(), 1);
}

#[test]
fn test_move_item_to_done_sets_completed_at() {
    let (_tmp, mut board) = make_board_with_items();
    let id = board.active.today[0].id;

    board.move_item(&id, &Location::Done).unwrap();
    let done_item = board.done.iter().find(|i| i.id == id).unwrap();
    assert!(done_item.completed_at.is_some());
}

#[test]
fn test_move_item_from_done_clears_completed_at() {
    let (_tmp, mut board) = make_board_with_items();
    let id = board.done[0].id;

    board.move_item(&id, &Location::Active(Category::Today)).unwrap();
    let reopened = board.active.today.iter().find(|i| i.id == id).unwrap();
    assert!(reopened.completed_at.is_none());
}

#[test]
fn test_board_absolute_order_persistence() {
    let temp = TempDir::new().unwrap();
    Storage::init(temp.path()).unwrap();

    let item1 = Item::new("Item 1");
    let item2 = Item::new("Item 2");
    let item3 = Item::new("Item 3");

    Storage::write_item(temp.path(), &item1, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(temp.path(), &item2, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(temp.path(), &item3, &Location::Active(Category::Today)).unwrap();

    let mut board = Storage::load_board(temp.path()).unwrap();

    // Explicit reorder: item3, item1, item2
    board.set_column_order(&Location::Active(Category::Today), vec![item3.id, item1.id, item2.id]);
    Storage::save_board_state(temp.path(), &board).unwrap();

    let loaded_board = Storage::load_board(temp.path()).unwrap();
    let today_ids: Vec<_> = loaded_board.active.today.iter().map(|i| i.id).collect();
    assert_eq!(today_ids, vec![item3.id, item1.id, item2.id]);
}

#[test]
fn test_board_state_serialization_with_last_active_date() {
    let state = BoardState {
        version: 1,
        order: std::collections::HashMap::new(),
        last_active_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap()),
    };
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("2026-08-15"));

    let deserialized: BoardState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.last_active_date, state.last_active_date);

    // Backwards compatibility: JSON without last_active_date
    let old_json = r#"{"version":1,"order":{}}"#;
    let from_old: BoardState = serde_json::from_str(old_json).unwrap();
    assert_eq!(from_old.last_active_date, None);
}

