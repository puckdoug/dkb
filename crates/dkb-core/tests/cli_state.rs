#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_default_state() {
    let dir = TempDir::new().unwrap();
    let state = CliState::load(dir.path());
    assert!(state.last_list.is_empty());
    assert!(state.current.is_none());
}

#[test]
fn test_save_and_reload() {
    let dir = TempDir::new().unwrap();
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut state = CliState::default();
    state.set_last_list(vec![id1, id2]);
    state.set_current(id2);
    state.save(dir.path()).unwrap();

    let loaded = CliState::load(dir.path());
    assert_eq!(loaded.last_list, vec![id1, id2]);
    assert_eq!(loaded.current, Some(id2));
}

#[test]
fn test_resolve_index() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut state = CliState::default();
    state.set_last_list(vec![id1, id2]);
    assert_eq!(state.resolve_index(0), Some(id1));
    assert_eq!(state.resolve_index(1), Some(id2));
    assert_eq!(state.resolve_index(2), None);
}

#[test]
fn test_clear_current() {
    let id = Uuid::new_v4();
    let mut state = CliState::default();
    state.set_current(id);
    state.clear_current();
    assert!(state.current.is_none());
}
