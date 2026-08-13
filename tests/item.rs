use chrono::Utc;
use dkb::item::{Category, Item, Status};

#[test]
fn test_item_new_sets_title_from_first_line() {
    let item = Item::new("Fix the login bug");
    assert_eq!(item.title(), "Fix the login bug");
    assert_eq!(item.body, "Fix the login bug");
    assert!(item.created_at <= Utc::now());
    assert_eq!(item.created_at, item.updated_at);
    assert!(item.completed_at.is_none());
}

#[test]
fn test_item_new_with_multiline_body() {
    let item = Item::new("Fix the login bug\n\nDetails about the bug here");
    assert_eq!(item.title(), "Fix the login bug");
    assert_eq!(item.body, "Fix the login bug\n\nDetails about the bug here");
}

#[test]
fn test_extract_title_from_body() {
    assert_eq!(Item::extract_title("Hello world"), "Hello world");
    assert_eq!(Item::extract_title("Hello world\nrest"), "Hello world");
    assert_eq!(Item::extract_title("\n\nHello world\nrest"), "Hello world");
    assert_eq!(Item::extract_title(""), "");
    assert_eq!(Item::extract_title("\n\n\n"), "");
}

#[test]
fn test_status_variants() {
    let _backlog = Status::Backlog;
    let _active = Status::Active;
    let _done = Status::Done;
}

#[test]
fn test_category_variants() {
    let _y = Category::Yesterday;
    let _t = Category::Today;
    let _tw = Category::ThisWeek;
    let _nw = Category::NextWeek;
}
