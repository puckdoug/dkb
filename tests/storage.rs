use dkb::item::{Status, Category};
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
