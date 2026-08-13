use dkb::item::{Category, Item, Status};
use dkb::storage::{Location, Storage};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_location_to_path_backlog() {
    let loc = Location::Backlog;
    assert_eq!(loc.to_path(), PathBuf::from("backlog"));
    assert_eq!(loc.status(), Status::Backlog);
    assert!(loc.category().is_none());
}

#[test]
fn test_location_to_path_active_today() {
    let loc = Location::Active(Category::Today);
    assert_eq!(loc.to_path(), PathBuf::from("active/today"));
    assert_eq!(loc.status(), Status::Active);
    assert_eq!(loc.category(), Some(Category::Today));
}

#[test]
fn test_location_to_path_done() {
    let loc = Location::Done;
    assert_eq!(loc.to_path(), PathBuf::from("done"));
    assert_eq!(loc.status(), Status::Done);
    assert!(loc.category().is_none());
}

#[test]
fn test_location_from_path() {
    assert_eq!(Location::from_path("backlog"), Location::Backlog);
    assert_eq!(Location::from_path("active/yesterday"), Location::Active(Category::Yesterday));
    assert_eq!(Location::from_path("active/today"), Location::Active(Category::Today));
    assert_eq!(Location::from_path("active/this_week"), Location::Active(Category::ThisWeek));
    assert_eq!(Location::from_path("active/next_week"), Location::Active(Category::NextWeek));
    assert_eq!(Location::from_path("done"), Location::Done);
}

#[test]
fn test_storage_init_creates_directories() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();
    assert!(data_dir.join("backlog").exists());
    assert!(data_dir.join("active/yesterday").exists());
    assert!(data_dir.join("active/today").exists());
    assert!(data_dir.join("active/this_week").exists());
    assert!(data_dir.join("active/next_week").exists());
    assert!(data_dir.join("done").exists());
}

#[test]
fn test_storage_init_idempotent() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();
    Storage::init(&data_dir).unwrap();
}

#[test]
fn test_write_item_creates_file() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let item = Item::new("Test task");
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();

    let expected_path = data_dir.join("backlog").join(format!("{}.md", item.id));
    assert!(expected_path.exists());
}

#[test]
fn test_write_item_active_today() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let item = Item::new("Today task");
    Storage::write_item(&data_dir, &item, &Location::Active(Category::Today)).unwrap();

    let expected_path = data_dir.join("active/today").join(format!("{}.md", item.id));
    assert!(expected_path.exists());
}

#[test]
fn test_read_item_round_trip() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let mut item = Item::new("Round trip");
    item.body = "Round trip\n\nBody text".to_string();
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();

    let read_back = Storage::read_item(&data_dir, &item.id, &Location::Backlog).unwrap();
    assert_eq!(read_back.id, item.id);
    assert_eq!(read_back.body, item.body);
    assert_eq!(read_back.title(), item.title());
    assert_eq!(read_back.created_at, item.created_at);
    assert_eq!(read_back.updated_at, item.updated_at);
    assert_eq!(read_back.completed_at, item.completed_at);
}

#[test]
fn test_read_item_with_completed_at() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let mut item = Item::new("Done task");
    item.completed_at = Some(chrono::Utc::now());
    Storage::write_item(&data_dir, &item, &Location::Done).unwrap();

    let read_back = Storage::read_item(&data_dir, &item.id, &Location::Done).unwrap();
    assert!(read_back.completed_at.is_some());
}

#[test]
fn test_move_item_backlog_to_active_today() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let item = Item::new("Move me");
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();

    let moved = Storage::move_item(
        &data_dir,
        &item.id,
        &Location::Backlog,
        &Location::Active(Category::Today),
    ).unwrap();

    // Old file should be gone
    let old_path = data_dir.join("backlog").join(format!("{}.md", item.id));
    assert!(!old_path.exists());
    // New file should exist
    let new_path = data_dir.join("active/today").join(format!("{}.md", item.id));
    assert!(new_path.exists());
    // updated_at should be refreshed
    assert!(moved.updated_at >= item.updated_at);
}

#[test]
fn test_move_item_to_done_sets_completed_at() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let item = Item::new("Complete me");
    Storage::write_item(&data_dir, &item, &Location::Active(Category::Today)).unwrap();
    assert!(item.completed_at.is_none());

    let moved = Storage::move_item(
        &data_dir,
        &item.id,
        &Location::Active(Category::Today),
        &Location::Done,
    ).unwrap();

    assert!(moved.completed_at.is_some());
}

#[test]
fn test_move_item_from_done_clears_completed_at() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let mut item = Item::new("Reopen me");
    item.completed_at = Some(chrono::Utc::now());
    Storage::write_item(&data_dir, &item, &Location::Done).unwrap();

    let moved = Storage::move_item(
        &data_dir,
        &item.id,
        &Location::Done,
        &Location::Active(Category::Today),
    ).unwrap();

    assert!(moved.completed_at.is_none());
}

#[test]
fn test_delete_item() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let item = Item::new("Delete me");
    Storage::write_item(&data_dir, &item, &Location::Backlog).unwrap();

    Storage::delete_item(&data_dir, &item.id, &Location::Backlog).unwrap();

    let path = data_dir.join("backlog").join(format!("{}.md", item.id));
    assert!(!path.exists());
}
