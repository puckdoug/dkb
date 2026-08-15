#![allow(clippy::pedantic)]

use dkb_core::board::Board;
use dkb_core::item::{Category, Item};
use dkb_core::storage::Location;
use dk::commands::list::{active_locations, collect_items, render_list};

#[test]
fn test_active_locations_order() {
    let locs = active_locations();
    assert_eq!(locs[0], Location::Active(Category::Yesterday));
    assert_eq!(locs[1], Location::Active(Category::Today));
    assert_eq!(locs[2], Location::Active(Category::ThisWeek));
    assert_eq!(locs[3], Location::Active(Category::NextWeek));
}

#[test]
fn test_collect_items_today() {
    let mut board = Board::default();
    let i1 = Item::new("task a");
    board.active.today.push(i1.clone());
    board.active.today.push(Item::new("task b"));
    let items = collect_items(&board, &Location::Active(Category::Today));
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, i1.id);
}

#[test]
fn test_render_list_basic() {
    let i1 = Item::new("First task");
    let i2 = Item::new("Second task");
    let items = vec![i1.clone(), i2.clone()];
    let out = render_list(&items, Some(i1.id), 80);
    assert!(out.contains("* First task"));
    assert!(out.contains("1 Second task"));
}

#[test]
fn test_render_list_no_current() {
    let i1 = Item::new("Only task");
    let items = vec![i1];
    let out = render_list(&items, None, 80);
    assert!(out.contains("0 Only task"));
    assert!(!out.contains('*'));
}
