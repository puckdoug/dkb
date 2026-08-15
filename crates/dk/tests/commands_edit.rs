#![allow(clippy::pedantic)]

use dkb_core::cli_state::CliState;
use dkb_core::item::{Category, Item};
use dkb_core::storage::{Location, Storage};
use dk::commands::edit::run_edit;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn fake_editor(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
    let script = dir.join("fake_editor.sh");
    let escaped = content.replace('\'', "'\\''");
    std::fs::write(
        &script,
        format!("#!/bin/sh\nfor f in \"$@\"; do :; done\necho '{escaped}' > \"$f\"\n"),
    )
    .unwrap();
    let mut perms = std::fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).unwrap();
    script
}

#[test]
fn test_edit_current_item() {
    let _guard = lock_env();
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let item = Item::new("# Original");
    Storage::write_item(dir.path(), &item, &Location::Active(Category::Today)).unwrap();
    let mut state = CliState::default();
    state.set_current(item.id);
    state.save(dir.path()).unwrap();

    let editor = fake_editor(dir.path(), "# Edited title");
    unsafe {
        std::env::set_var("VISUAL", &editor);
    }

    run_edit(dir.path(), &[]).unwrap();

    let board = Storage::load_board(dir.path()).unwrap();
    let edited = board.find_item(&item.id).unwrap();
    assert!(edited.body.contains("Edited title"));
}

#[test]
fn test_edit_by_index() {
    let _guard = lock_env();
    let dir = TempDir::new().unwrap();
    Storage::init(dir.path()).unwrap();
    let i1 = Item::new("# first");
    let i2 = Item::new("# second");
    Storage::write_item(dir.path(), &i1, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(dir.path(), &i2, &Location::Active(Category::Today)).unwrap();
    let mut state = CliState::default();
    state.set_last_list(vec![i1.id, i2.id]);
    state.save(dir.path()).unwrap();

    let editor = fake_editor(dir.path(), "# changed");
    unsafe {
        std::env::set_var("VISUAL", &editor);
    }

    run_edit(dir.path(), &["1".to_string()]).unwrap();
    let board = Storage::load_board(dir.path()).unwrap();
    let edited = board.find_item(&i2.id).unwrap();
    assert!(edited.body.contains("changed"));
}
