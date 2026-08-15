#![allow(clippy::pedantic)]

use dk::editor_launch::resolve_editor;
use std::env;
use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn test_resolve_editor_visual() {
    let _guard = lock_env();
    unsafe {
        env::set_var("VISUAL", "vim");
        env::remove_var("EDITOR");
    }
    assert_eq!(resolve_editor(), "vim");
}

#[test]
fn test_resolve_editor_fallback_to_editor() {
    let _guard = lock_env();
    unsafe {
        env::remove_var("VISUAL");
        env::set_var("EDITOR", "nano");
    }
    assert_eq!(resolve_editor(), "nano");
}

#[test]
fn test_resolve_editor_default_vi() {
    let _guard = lock_env();
    unsafe {
        env::remove_var("VISUAL");
        env::remove_var("EDITOR");
    }
    assert_eq!(resolve_editor(), "vi");
}
