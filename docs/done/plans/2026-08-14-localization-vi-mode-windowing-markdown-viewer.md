# Localization, Vi-Mode, Windowing & Markdown Viewer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement full 44-language macOS localization with OS default detection and runtime switching, comprehensive Vi-mode editing with Ex command line (`:`) and search (`/`, `?`), scoped `cmd-w` window/modal closing, multiple main window reopening (`cmd-option-n`), and priority-based external Markdown viewer launching (`cmd-shift-m`, context menus, Settings file picker).

**Architecture:** 
- `src/i18n/`: A dedicated localization module with string catalogs across 44 macOS languages, OS locale detection, fallback resolution, and reactive GPUI menu/view refreshes.
- `src/vi.rs`: Complete Vi state machine handling counts, operators, text objects, motions, search, and Ex command parsing (`:w`, `:q`, `:wq`, `:x`, `:q!`, `:<line>`, `:%s`).
- `src/viewer.rs`: Auto-detection for Marked, Marked 2, MD-Viewer with fallback and native file picker config.
- `src/app.rs` & `src/editor.rs`: Multi-window management (`cmd-option-n`), modal `cmd-w` event trapping, right-click card context menus, and localized UI/menu rendering.

**Tech Stack:** Rust (2024 edition), GPUI, Serde, TOML, `rfd` (native file dialogs), `unicode-segmentation`.

**Spec:** `docs/specs/2026-08-14-localization-vi-mode-windowing-markdown-viewer-design.md`

## Global Constraints
- Clean Architecture and Clean Code principles with strong types and explicit module boundaries.
- macOS standard keyboard shortcuts: `cmd-option-n` for new main window, `cmd-w` for closing modal or window, `cmd-shift-m` for opening markdown viewer.
- All 44 macOS-supported languages plus System Auto detection must be supported.
- `cargo clippy` and `cargo test` must pass with zero errors and zero warnings.

---

### Task 1: Localization Engine & OS Detection (`i18n/`)

**Files:**
- Create: `src/i18n/mod.rs`
- Create: `src/i18n/locales.rs`
- Modify: `src/lib.rs`
- Modify: `src/config.rs`
- Test: `tests/i18n.rs`
- Test: `tests/config.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
  pub enum Language {
      #[default]
      Auto,
      Ar, Ca, ZhHans, ZhHant, Hr, Cs, Da, Nl,
      EnAu, EnCa, EnIn, EnJp, EnGb, EnUs,
      Fi, FrCa, FrFr, De, El, He, Hi, Hu, Id, It,
      Ja, Ko, Ms, Nb, Pl, PtBr, PtPt, Ro, Ru, Sk,
      EsCl, Es419, EsMx, EsEs, EsUs, Sv, Th, Tr, Uk, Vi,
  }
  pub fn detect_system_language() -> Language;
  pub fn t(key: &str, lang: Language) -> &'static str;
  ```

- [ ] **Step 1: Write failing unit test for `Language` enum, detection, and translation keys**

```rust
// tests/i18n.rs
use dkb::i18n::{detect_system_language, t, Language};

#[test]
fn test_supported_languages_count() {
    assert_eq!(Language::all().len(), 45); // Auto + 44 languages
}

#[test]
fn test_translation_fallback() {
    assert_eq!(t("tab.backlog", Language::EnUs), "Backlog");
    assert_eq!(t("tab.backlog", Language::EsEs), "Pendientes");
    assert_eq!(t("tab.backlog", Language::FrFr), "Backlog");
    assert_eq!(t("tab.backlog", Language::De), "Rückstand");
    // Fallback to English when key is missing in custom language
    assert_eq!(t("tab.backlog", Language::Auto), t("tab.backlog", detect_system_language()));
}

#[test]
fn test_language_display_names() {
    assert_eq!(Language::EnUs.display_name(), "English (US)");
    assert_eq!(Language::Auto.display_name(), "System Default (Auto)");
    assert_eq!(Language::Ja.display_name(), "Japanese");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test i18n`
Expected: Compilation failure because `i18n` module does not exist yet.

- [ ] **Step 3: Implement `src/i18n/mod.rs` and `src/i18n/locales.rs`**

Implement `Language` enum with all 44 macOS languages + `Auto`, display names, string catalogs covering all UI keys (`tab.*`, `col.*`, `menu.*`, `settings.*`, `editor.*`, `viewer.*`), `detect_system_language()`, and `t(key, lang)` resolution with hierarchy fallback.

- [ ] **Step 4: Update `src/config.rs` to include `language: Language`**

Add `pub language: Language` to `Config` with default `Language::Auto`, serialized as `language = "auto"`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test i18n && cargo test --test config`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/i18n/ src/lib.rs src/config.rs tests/i18n.rs tests/config.rs
git commit -m "feat(i18n): add 44-language localization catalog, detection, and config"
```

---

### Task 2: External Markdown Viewer Launcher & Priority Auto-Detection (`viewer.rs`)

**Files:**
- Create: `src/viewer.rs`
- Modify: `src/lib.rs`
- Modify: `src/config.rs`
- Modify: `Cargo.toml` (add `rfd = "0.15"`)
- Test: `tests/viewer.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub enum ViewerPreference {
      Auto,
      Custom(PathBuf),
  }
  pub fn detect_default_viewer() -> Option<PathBuf>;
  pub fn resolve_viewer_path(pref: &ViewerPreference) -> Option<PathBuf>;
  pub fn open_in_viewer(file_path: &Path, pref: &ViewerPreference) -> std::io::Result<()>;
  pub fn pick_viewer_file_dialog() -> Option<PathBuf>;
  ```

- [ ] **Step 1: Write failing unit test for viewer auto-detection priority and launch command building**

```rust
// tests/viewer.rs
use std::path::{Path, PathBuf};
use dkb::viewer::{resolve_viewer_path, ViewerPreference};

#[test]
fn test_viewer_priority_ordering() {
    let pref_auto = ViewerPreference::Auto;
    let resolved = resolve_viewer_path(&pref_auto);
    // If Marked / Marked 2 / MD-Viewer exist on system, resolved will be Some(path)
    if let Some(path) = resolved {
        assert!(path.to_string_lossy().contains(".app") || path.exists());
    }
}

#[test]
fn test_viewer_custom_path() {
    let custom = ViewerPreference::Custom(PathBuf::from("/Applications/Marked 2.app"));
    assert_eq!(resolve_viewer_path(&custom), Some(PathBuf::from("/Applications/Marked 2.app")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test viewer`
Expected: FAIL (module not defined).

- [ ] **Step 3: Implement `src/viewer.rs` and update `src/config.rs`**

Add `rfd` to `Cargo.toml`.
Implement `detect_default_viewer()` checking in order:
1. `/Applications/Marked.app` or `~/Applications/Marked.app`
2. `/Applications/Marked 2.app` or `~/Applications/Marked 2.app`
3. `/Applications/MD-Viewer.app` or `~/Applications/MD-Viewer.app`
4. Fallback to `None` (uses default system markdown handler).
Implement `open_in_viewer(file_path, pref)` using `std::process::Command::new("open")`.
Implement `pick_viewer_file_dialog()` using `rfd::FileDialog`.
Update `Config` to include `pub markdown_viewer: ViewerPreference`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test viewer`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/viewer.rs src/lib.rs src/config.rs tests/viewer.rs
git commit -m "feat(viewer): implement markdown viewer auto-detection, launching, and config"
```

---

### Task 3: Comprehensive Vi Engine — Counts, Operators, Text Objects, Search & Ex Commands (`vi.rs`)

**Files:**
- Modify: `src/vi.rs`
- Test: `tests/vi.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum ViMode {
      Normal,
      Insert,
      Visual,
      VisualLine,
      Command,
      Search(SearchDirection),
      Replace,
  }
  pub enum ExCommand {
      Write,
      Quit { force: bool },
      WriteQuit,
      GotoLine(usize),
      Substitute { pattern: String, replacement: String, global: bool, ignore_case: bool },
      DeleteLine,
  }
  pub enum ViActionResult {
      None,
      Handled,
      ExecuteEx(ExCommand),
      SaveAndClose,
      CloseWithoutSaving,
      Save,
  }
  ```

- [ ] **Step 1: Write comprehensive failing tests for Vi motions, counts, operators, text objects, and Ex commands**

```rust
// In tests/vi.rs
#[test]
fn test_vi_counts_and_motions() {
    let mut state = TextInputState::new("line 1\nline 2\nline 3\nline 4\nline 5");
    let mut vi = ViState::new();
    // 3j moves down 3 lines
    vi.handle_key("3", &mut state);
    vi.handle_key("j", &mut state);
    assert_eq!(state.cursor_offset(), 21); // line 4
    // 2k moves up 2 lines
    vi.handle_key("2", &mut state);
    vi.handle_key("k", &mut state);
    assert_eq!(state.cursor_offset(), 7); // line 2
}

#[test]
fn test_vi_text_objects() {
    let mut state = TextInputState::new("hello \"world test\" end");
    state.move_to(9); // inside "world test"
    let mut vi = ViState::new();
    // ci" replaces inside quotes
    vi.handle_key("c", &mut state);
    vi.handle_key("i", &mut state);
    vi.handle_key("\"", &mut state);
    assert_eq!(state.content(), "hello \"\" end");
    assert_eq!(vi.mode, ViMode::Insert);
}

#[test]
fn test_vi_ex_commands() {
    let mut state = TextInputState::new("hello world");
    let mut vi = ViState::new();
    vi.handle_key(":", &mut state);
    assert_eq!(vi.mode, ViMode::Command);
    for ch in "%s/world/rust/g".chars() {
        vi.handle_command_input(&ch.to_string(), &mut state);
    }
    let res = vi.handle_command_enter(&mut state);
    assert_eq!(state.content(), "hello rust");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test vi`
Expected: FAIL (missing methods / variants).

- [ ] **Step 3: Implement the full Vi state machine in `src/vi.rs`**

- Support counts accumulator `Option<usize>`.
- Support operator pending state with `d`, `c`, `y`, `gu`, `gU`, `>`, `<`.
- Implement word motions (`w`, `W`, `b`, `B`, `e`, `E`, `ge`, `gE`), line motions (`0`, `^`, `$`), buffer motions (`gg`, `G`, `%`), find char (`f`, `F`, `t`, `T`, `;`, `,`).
- Implement text objects (`iw`, `aw`, `i"`, `a"`, `i'`, `a'`, `i(`, `a(`, `i[`, `a[`, `i{`, `a{`, `ip`, `ap`).
- Implement Search state (`/`, `?`, `n`, `N`, `*`, `#`).
- Implement Ex command line parser and runner for `:w`, `:q`, `:wq`, `:x`, `:q!`, `:<line>`, `:%s/find/repl/flags`, `:d`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test vi`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/vi.rs tests/vi.rs
git commit -m "feat(vi): implement full vi motions, counts, operators, text objects, search and ex commands"
```

---

### Task 4: Editor Command Line & Search UI and Modal `cmd-w` Scoping (`editor.rs`)

**Files:**
- Modify: `src/editor.rs`
- Modify: `src/app.rs`
- Test: `tests/app.rs`

**Interfaces:**
- In `ItemEditor`:
  - Renders interactive command-line / search bar at the bottom when in `ViMode::Command` or `ViMode::Search`.
  - Dispatches `SaveEditor`, `CloseWindow`, `CancelEditor` upon Ex commands `:w`, `:q`, `:wq`, etc.
  - `cmd-w` handling in `ItemEditor` emits appropriate dismiss event when attached vs torn-off.

- [ ] **Step 1: Write test for modal `cmd-w` action isolation**

```rust
// tests/app.rs
#[test]
fn test_modal_cmd_w_cancels_editor_without_closing_window() {
    // Verify that when KanbanView has an active modal editor, CancelEditor action is triggered
    // rather than CloseWindow.
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test app`
Expected: Test failure or assertion check.

- [ ] **Step 3: Update `src/editor.rs` and `src/app.rs`**

- In `src/editor.rs`:
  - Add command-line bar UI and search bar UI at the bottom of the editor when active.
  - Handle key strokes in command/search modes.
  - Wire Ex command execution directly into `on_save`, `on_close`, `on_cancel`.
- In `src/app.rs`:
  - Update `on_close_window`: If `self.editing.is_some()`, dismiss editor (`self.editing = None`), do NOT remove the window.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test app && cargo test --test vi`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/editor.rs src/app.rs tests/app.rs
git commit -m "feat(editor): render vi command bar, search bar, and scope cmd-w to dismiss modal"
```

---

### Task 5: Multi-Window Support (`cmd-option-n`), Context Menus, and Markdown Viewer Shortcut (`cmd-shift-m`) (`app.rs`, `bin/dkb.rs`)

**Files:**
- Modify: `src/app.rs`
- Modify: `src/bin/dkb.rs`
- Test: `tests/app.rs`

**Interfaces:**
- Produces:
  - `OpenNewMainWindow` action bound to `cmd-option-n` and "File -> New Window".
  - `OpenInMarkdownViewer` action bound to `cmd-shift-m` and "Item -> Open in Markdown Viewer".
  - Card right-click context menu with options:
    - "Open in Markdown Viewer"
    - "Open / Edit"
    - "Mark Done / Reopen"
    - "Move to..." sub-actions
    - "Delete"

- [ ] **Step 1: Write test for multi-window action and viewer shortcut registration**

```rust
// In tests/app.rs
#[test]
fn test_key_bindings_include_new_window_and_viewer() {
    let bindings = KanbanView::key_bindings(Language::EnUs);
    assert!(bindings.iter().any(|b| b.action() == "dkb::OpenNewMainWindow"));
    assert!(bindings.iter().any(|b| b.action() == "dkb::OpenInMarkdownViewer"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test app`
Expected: Compilation / test failure.

- [ ] **Step 3: Implement actions, keybindings, context menu, and multi-window opening**

- In `src/app.rs`:
  - Define `actions!(dkb, [..., OpenNewMainWindow, OpenInMarkdownViewer])`.
  - Implement `on_open_new_main_window` opening a new window via `cx.open_window(...)`.
  - Implement `on_open_in_markdown_viewer` locating the item file in `data_dir` and launching via `crate::viewer::open_in_viewer`.
  - Add `on_mouse_down(MouseButton::Right, ...)` on item card rendering an interactive context menu overlay or popup menu with item actions including "Open in Markdown Viewer".

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test app`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/bin/dkb.rs tests/app.rs
git commit -m "feat(app): add multi-window support (cmd-option-n), viewer shortcut (cmd-shift-m), and card context menu"
```

---

### Task 6: Settings Screen Updates & Live Localized GPUI UI (`app.rs`, `editor.rs`, `config.rs`)

**Files:**
- Modify: `src/app.rs`
- Modify: `src/editor.rs`
- Modify: `src/config.rs`
- Test: `tests/app.rs`
- Test: `tests/config.rs`

**Interfaces:**
- Settings screen includes:
  - Language selection list / dropdown with 44 macOS languages + System Default (Auto), showing native and localized names.
  - Markdown viewer selector displaying current path / auto-detected status, with "Browse..." (file dialog) and "Reset to Auto-Detect".
  - Theme mode, Vi-mode, Line numbers, Storage directory.
- Dynamic runtime language switching immediately updates macOS application menus and all UI tabs/columns/buttons.

- [ ] **Step 1: Write tests for settings serialization and language change propagation**

```rust
// tests/config.rs
#[test]
fn test_config_with_language_and_viewer() {
    let mut config = Config::load_from(&path).unwrap();
    config.language = Language::Ja;
    config.markdown_viewer = ViewerPreference::Custom(PathBuf::from("/Applications/Marked 2.app"));
    config.save_to(&path).unwrap();
    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.language, Language::Ja);
    assert_eq!(loaded.markdown_viewer, ViewerPreference::Custom(PathBuf::from("/Applications/Marked 2.app")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config`
Expected: PASS/FAIL based on step completion.

- [ ] **Step 3: Implement Settings UI controls and live i18n rendering**

- In `src/app.rs`:
  - Wire all strings across tabs, headers, columns, buttons, cards, menus, and settings through `crate::i18n::t(key, self.config.language)`.
  - In `render_settings_screen`, render a searchable/scrollable Language picker for all 44 languages + Auto.
  - In `render_settings_screen`, render the Markdown Viewer row with "Browse..." invoking `pick_viewer_file_dialog()`.
  - When language changes in Settings: update config, save to disk, call `Self::setup_menus(cx)` with new language, and `cx.notify()`.

- [ ] **Step 4: Run full test suite and clippy**

Run: `cargo test && cargo clippy --all-targets`
Expected: All tests pass with zero warnings.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/editor.rs src/config.rs tests/app.rs tests/config.rs
git commit -m "feat(settings): add language picker, viewer browser, and live localization across all UI elements"
```

---

## 5. Verification Checklist

1. `cargo test` runs and passes all unit and integration tests.
2. `cargo clippy --all-targets` runs with 0 warnings.
3. `cargo build --release` produces binary.
4. Verify runtime features:
   - Language selector shows all 44 macOS languages + Auto, changes UI & menu bar strings immediately.
   - Vi-mode in editor supports `:` Ex commands (`:w`, `:q`, `:wq`, `:q!`, `:<num>`, `:%s`), counts, operators, text objects, and `/` search.
   - `cmd-w` closes only the modal editor when attached, and closes the window when no modal is open.
   - `cmd-option-n` opens multiple main windows independently.
   - `cmd-shift-m`, Item menu, and right-click context menu open the selected markdown file in Marked / Marked 2 / MD-Viewer / custom viewer.
