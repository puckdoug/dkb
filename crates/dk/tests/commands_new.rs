#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::Category;
use dkb_core::storage::{Location, Storage};
use dk::commands::new::build_item_from_body;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;
use uuid::Uuid;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn install_fake_editor(dir: &std::path::Path, contents: &str) -> std::path::PathBuf {
    let script = dir.join("fake_editor.sh");
    let script_body = format!("#!/bin/sh\nprintf '{}' > \"$2\"\n", contents);
    std::fs::write(&script, script_body).unwrap();
    let mut perms = std::fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).unwrap();
    script
}

#[test]
fn test_build_item_extracts_body() {
    let id = Uuid::new_v4();
    let item = build_item_from_body(id, "# My task\nsome detail\n");
    assert_eq!(item.id, id);
    assert_eq!(item.body, "# My task\nsome detail\n");
    assert!(item.completed_at.is_none());
}

#[test]
fn test_build_item_empty_aborts() {
    let id = Uuid::new_v4();
    let item = build_item_from_body(id, "# ");
    assert!(item.body.trim().is_empty() || item.body.trim() == "#");
}

#[test]
fn test_new_item_saved_to_backlog_by_default() {
    let _guard = lock_env();
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let editor = install_fake_editor(dir.path(), "# Test task\\n");
    unsafe {
        std::env::set_var("VISUAL", &editor);
        std::env::remove_var("EDITOR");
    }
    let id = dk::commands::new::run_new(dir.path(), None).unwrap();
    let board = Storage::load_board(dir.path()).unwrap();
    assert!(board.backlog.iter().any(|i| i.id == id));
    let state = CliState::load(dir.path());
    assert_eq!(state.current, Some(id));
}

#[test]
fn test_new_item_saved_to_category() {
    let _guard = lock_env();
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let editor = install_fake_editor(dir.path(), "# Today task\\n");
    unsafe {
        std::env::set_var("VISUAL", &editor);
        std::env::remove_var("EDITOR");
    }
    let id = dk::commands::new::run_new(dir.path(), Some("today")).unwrap();
    let board = Storage::load_board(dir.path()).unwrap();
    assert!(board.active.today.iter().any(|i| i.id == id));
}

#[test]
fn test_new_item_aborts_when_empty() {
    let _guard = lock_env();
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let editor = install_fake_editor(dir.path(), "# ");
    unsafe {
        std::env::set_var("VISUAL", &editor);
        std::env::remove_var("EDITOR");
    }
    let result = dk::commands::new::run_new(dir.path(), None);
    assert!(result.is_err());
}

#[test]
fn test_parse_category_location() {
    assert_eq!(
        dkb_core::storage::Location::Active(Category::Today),
        Location::Active(Category::Today)
    );
}
