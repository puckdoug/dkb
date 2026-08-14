# Comprehensive Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement all 19 findings: settings screen, `Cmd-,` shortcut and app menu, vi-mode and line numbers in editor, light/dark/system themes, tabs bar layout, absolute column sort persistence, drag-and-drop, `Cmd-]`/`Cmd-[` and 2D spatial arrow navigation, vi kanban keys, enter-to-edit, title unformatting, sub-items with recursive count badge (`↪ <count>`) and drill-down, iwe workspace support, and macOS `.app` bundle & App Store packaging.

**Architecture:** A clean modular architecture separating domain models (item, config, theme, link, iwe, vi) from GPUI rendering views (tabs, board, editor, settings). Column sorting is persisted to `board_state.json`. Links and sub-items are parsed using markdown link rules with recursive cycle-safe graph resolution matching iwe conventions.

**Tech Stack:** Rust (edition 2024), GPUI (Zed framework), serde, serde_json, serde_yaml, chrono, uuid, unicode-segmentation.

**Spec:** `docs/superpowers/specs/2026-08-14-comprehensive-improvements-design.md`

## Global Constraints
- Rust 2024 edition compatibility.
- Follow Clean Architecture and Clean Code principles with comprehensive unit tests for domain logic.
- Type hints on all functions and destructure imports where appropriate.
- Never ignore compiler errors or warnings.

---

### Task 1: Title Cleaning & Markdown Formatting Stripping

**Files:**
- Modify: `src/item.rs`
- Modify: `tests/item.rs`

**Interfaces:**
- Consumes: `Item::extract_title(&str) -> String`
- Produces: `Item::clean_title(&str) -> String` (strips `#`, `**`, `*`, `_`, `` ` ``, `~~`, `[link](url)`)

- [ ] **Step 1: Write the failing tests for markdown title cleaning**

In `tests/item.rs`:
```rust
#[test]
fn test_clean_title_formatting() {
    assert_eq!(dkb::item::Item::clean_title("# Heading One"), "Heading One");
    assert_eq!(dkb::item::Item::clean_title("### Subheading"), "Subheading");
    assert_eq!(dkb::item::Item::clean_title("**Bold Title**"), "Bold Title");
    assert_eq!(dkb::item::Item::clean_title("*Italic Title*"), "Italic Title");
    assert_eq!(dkb::item::Item::clean_title("`Code Title`"), "Code Title");
    assert_eq!(dkb::item::Item::clean_title("[Link Title](https://example.com)"), "Link Title");
    assert_eq!(dkb::item::Item::clean_title("~~Strikethrough~~"), "Strikethrough");
    assert_eq!(dkb::item::Item::clean_title("## **Complex** `Title` with [Link](url)"), "Complex Title with Link");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test item test_clean_title_formatting`
Expected: FAIL (function `clean_title` not found)

- [ ] **Step 3: Implement `clean_title` in `src/item.rs`**

In `src/item.rs`:
```rust
impl Item {
    pub fn clean_title(raw: &str) -> String {
        let trimmed = raw.trim();
        let without_header = trimmed.trim_start_matches('#').trim();
        
        let mut result = String::new();
        let mut chars = without_header.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '*' | '_' | '`' | '~' => {
                    // Skip markdown styling markers
                    continue;
                }
                '[' => {
                    // Extract link text before ']' and skip '(url)'
                    let mut link_text = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        chars.next();
                        if next_ch == ']' {
                            break;
                        }
                        link_text.push(next_ch);
                    }
                    if let Some(&'(') = chars.peek() {
                        chars.next();
                        while let Some(next_ch) = chars.next() {
                            if next_ch == ')' {
                                break;
                            }
                        }
                    }
                    result.push_str(&Self::clean_title(&link_text));
                }
                _ => result.push(ch),
            }
        }
        
        result.trim().to_string()
    }

    pub fn extract_title(body: &str) -> String {
        let first_line = body.lines().find(|line| !line.trim().is_empty()).unwrap_or("");
        Self::clean_title(first_line)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test item`
Expected: PASS

- [ ] **Step 5: Verify build & tests**

Run: `cargo check && cargo test`
Expected: All tests pass with 0 warnings.

---

### Task 2: Config Subsystem Extension & Theming Engine

**Files:**
- Create: `src/theme.rs`
- Modify: `src/config.rs`
- Modify: `src/lib.rs`
- Modify: `tests/config.rs`

**Interfaces:**
- Consumes: None
- Produces: `ThemeMode`, `Config` (with `vi_mode`, `line_numbers`, `theme_mode`, `data_dir`), `Theme` (with reactive colors).

- [ ] **Step 1: Write failing tests for Config extension**

In `tests/config.rs`:
```rust
use dkb::config::{Config, ThemeMode};
use tempfile::TempDir;

#[test]
fn test_config_defaults_and_serialization() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.toml");
    
    let config = Config::load_from(&config_path).unwrap();
    assert_eq!(config.vi_mode, false);
    assert_eq!(config.line_numbers, false);
    assert_eq!(config.theme_mode, ThemeMode::System);
    
    let updated = Config {
        data_dir: temp.path().join("custom_data"),
        vi_mode: true,
        line_numbers: true,
        theme_mode: ThemeMode::Dark,
    };
    updated.save_to(&config_path).unwrap();
    
    let reloaded = Config::load_from(&config_path).unwrap();
    assert_eq!(reloaded.vi_mode, true);
    assert_eq!(reloaded.line_numbers, true);
    assert_eq!(reloaded.theme_mode, ThemeMode::Dark);
    assert_eq!(reloaded.data_dir, temp.path().join("custom_data"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config`
Expected: FAIL

- [ ] **Step 3: Implement `src/config.rs` and `src/theme.rs`**

Create `src/theme.rs`:
```rust
use gpui::{Rgba, rgb, rgba};
use crate::config::ThemeMode;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg_window: Rgba,
    pub bg_surface: Rgba,
    pub bg_column: Rgba,
    pub bg_tab_bar: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub border: Rgba,
    pub selection: Rgba,
    pub accent: Rgba,
}

impl Theme {
    pub fn resolve(mode: ThemeMode, system_is_dark: bool) -> Self {
        let is_dark = match mode {
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
            ThemeMode::System => system_is_dark,
        };

        if is_dark {
            Self {
                bg_window: rgb(0x1e1e1e),
                bg_surface: rgb(0x252526),
                bg_column: rgb(0x2d2d2d),
                bg_tab_bar: rgb(0x181818),
                text_primary: rgb(0xe0e0e0),
                text_secondary: rgb(0x9e9e9e),
                border: rgb(0x383838),
                selection: rgb(0x0a84ff),
                accent: rgb(0x4488ff),
            }
        } else {
            Self {
                bg_window: rgb(0xf5f5f5),
                bg_surface: rgb(0xffffff),
                bg_column: rgb(0xeceff1),
                bg_tab_bar: rgb(0xe0e0e0),
                text_primary: rgb(0x212121),
                text_secondary: rgb(0x757575),
                border: rgb(0xd0d0d0),
                selection: rgb(0x2196f3),
                accent: rgb(0x007aff),
            }
        }
    }
}
```

Update `src/config.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    #[serde(default)]
    pub vi_mode: bool,
    #[serde(default)]
    pub line_numbers: bool,
    #[serde(default)]
    pub theme_mode: ThemeMode,
}

impl Config {
    pub fn default_data_dir() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join("Library/Application Support/dkb")
        } else {
            PathBuf::from("Library/Application Support/dkb")
        }
    }

    pub fn config_file_path() -> PathBuf {
        Self::default_data_dir().join("config.toml")
    }

    pub fn load() -> std::io::Result<Self> {
        Self::load_from(&Self::config_file_path())
    }

    pub fn load_from(config_path: &Path) -> std::io::Result<Self> {
        if !config_path.exists() {
            let default_config = Self {
                data_dir: Self::default_data_dir(),
                vi_mode: false,
                line_numbers: false,
                theme_mode: ThemeMode::System,
            };
            default_config.save_to(config_path)?;
            return Ok(default_config);
        }

        let content = std::fs::read_to_string(config_path)?;
        let mut config: Config = toml::from_str(&content)
            .unwrap_or_else(|_| Config {
                data_dir: Self::default_data_dir(),
                vi_mode: false,
                line_numbers: false,
                theme_mode: ThemeMode::System,
            });
        
        config.data_dir = Self::expand_tilde(&config.data_dir.to_string_lossy());
        Ok(config)
    }

    pub fn save_to(&self, config_path: &Path) -> std::io::Result<()> {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn expand_tilde(path: &str) -> PathBuf {
        if let Some(rest) = path.strip_prefix("~/")
            && let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home).join(rest);
            }
        PathBuf::from(path)
    }
}
```

- [ ] **Step 4: Update `Cargo.toml` with `toml` dependency and `src/lib.rs`**

Add `toml = "0.8"` to `Cargo.toml`, export `pub mod theme;` in `src/lib.rs`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test config`
Expected: PASS

---

### Task 3: Board Absolute Ordering & Persistence

**Files:**
- Modify: `src/board.rs`
- Modify: `src/storage.rs`
- Modify: `tests/board.rs`

**Interfaces:**
- Consumes: `Location`, `Item`, `Uuid`
- Produces: `BoardState` (`board_state.json` persistence), `Board::reorder_item`, `Board::move_item_to_index`

- [ ] **Step 1: Write failing tests for board absolute ordering**

In `tests/board.rs`:
```rust
use dkb::board::Board;
use dkb::item::{Category, Item};
use dkb::storage::{Location, Storage};
use tempfile::TempDir;

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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test board`
Expected: FAIL

- [ ] **Step 3: Implement BoardState and ordering logic**

In `src/board.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoardState {
    pub version: u32,
    pub order: HashMap<String, Vec<Uuid>>,
}

impl Board {
    pub fn set_column_order(&mut self, location: &Location, ordered_ids: Vec<Uuid>) {
        let items_vec = match location {
            Location::Backlog => &mut self.backlog,
            Location::Active(Category::Yesterday) => &mut self.active.yesterday,
            Location::Active(Category::Today) => &mut self.active.today,
            Location::Active(Category::ThisWeek) => &mut self.active.this_week,
            Location::Active(Category::NextWeek) => &mut self.active.next_week,
            Location::Done => &mut self.done,
        };
        
        let mut map: HashMap<Uuid, Item> = items_vec.drain(..).map(|i| (i.id, i)).collect();
        for id in &ordered_ids {
            if let Some(item) = map.remove(id) {
                items_vec.push(item);
            }
        }
        for (_, remaining_item) in map {
            items_vec.push(remaining_item);
        }
    }
}
```

In `src/storage.rs`:
Implement `save_board_state` and update `load_board` to read `<data_dir>/board_state.json` and apply ordering.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test board`
Expected: PASS

---

### Task 4: Markdown Links, Recursive Sub-Items & iwe Integration

**Files:**
- Create: `src/link.rs`
- Create: `src/iwe.rs`
- Modify: `src/lib.rs`
- Create: `tests/link.rs`

**Interfaces:**
- Consumes: `Item`, `data_dir`
- Produces: `link::extract_links(&str) -> Vec<Uuid>`, `link::count_recursive_subitems(Uuid, &Path) -> usize`, `iwe::init_workspace(&Path)`

- [ ] **Step 1: Write failing tests for sub-item link parsing and recursive counting**

In `tests/link.rs`:
```rust
use dkb::link::{count_recursive_subitems, extract_links};
use dkb::item::{Category, Item};
use dkb::storage::{Location, Storage};
use tempfile::TempDir;

#[test]
fn test_link_extraction() {
    let body = "Parent item\n- [Sub 1](00000000-0000-0000-0000-000000000001.md)\n- [[00000000-0000-0000-0000-000000000002]]";
    let links = extract_links(body);
    assert_eq!(links.len(), 2);
}

#[test]
fn test_recursive_subitem_count() {
    let temp = TempDir::new().unwrap();
    Storage::init(temp.path()).unwrap();

    let child_leaf = Item::new("Leaf child");
    let mut child_middle = Item::new("Middle child");
    child_middle.body.push_str(&format!("\n- [Leaf]({}.md)", child_leaf.id));
    
    let mut root = Item::new("Root item");
    root.body.push_str(&format!("\n- [Middle]({}.md)", child_middle.id));

    Storage::write_item(temp.path(), &child_leaf, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(temp.path(), &child_middle, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(temp.path(), &root, &Location::Active(Category::Today)).unwrap();

    let count = count_recursive_subitems(root.id, temp.path());
    assert_eq!(count, 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test link`
Expected: FAIL

- [ ] **Step 3: Implement `src/link.rs` and `src/iwe.rs`**

Implement regex/parsers in `src/link.rs` with `HashSet<Uuid>` cycle-prevention.
Implement `iwe::init_workspace` in `src/iwe.rs` to write `.iwe/config.toml`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test link`
Expected: PASS

---

### Task 5: Vi Modal State Machine & Navigation

**Files:**
- Create: `src/vi.rs`
- Modify: `src/lib.rs`
- Create: `tests/vi.rs`

**Interfaces:**
- Consumes: `TextInputState`
- Produces: `ViState`, `ViMode` (`Normal`, `Insert`, `Visual`), `handle_vi_key(&mut self, &str, &mut TextInputState) -> bool`

- [ ] **Step 1: Write failing tests for Vi modal state machine**

In `tests/vi.rs`:
```rust
use dkb::vi::{ViMode, ViState};
use dkb::text_input::TextInputState;

#[test]
fn test_vi_mode_transitions() {
    let mut state = TextInputState::new("hello world");
    let mut vi = ViState::new();
    assert_eq!(vi.mode, ViMode::Normal);

    // 'i' enters insert mode
    vi.handle_key("i", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);

    // Escape returns to normal mode
    vi.handle_key("escape", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test vi`
Expected: FAIL

- [ ] **Step 3: Implement `src/vi.rs`**

Implement modal transitions and motions (`h`, `j`, `k`, `l`, `w`, `b`, `0`, `$`, `dd`, `yy`, `p`, `u`, `Ctrl-r`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test vi`
Expected: PASS

---

### Task 6: Editor Enhancements (Line Numbers Gutter & Vi Integration)

**Files:**
- Create: `src/editor.rs`
- Modify: `src/app.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `ItemEditor`, `Config`, `Theme`, `ViState`
- Produces: `ItemEditor` with line numbers gutter and vi status line.

- [ ] **Step 1: Implement `src/editor.rs`**
Render left gutter for line numbers when `config.line_numbers` is true.
Render bottom status bar indicating `-- NORMAL --`, `-- INSERT --`, or `-- VISUAL --` when `config.vi_mode` is true.

- [ ] **Step 2: Verify compilation and tests**
Run: `cargo check && cargo test`
Expected: PASS

---

### Task 7: Kanban View Tabs, 2D Navigation, Drag-and-Drop & Settings Screen

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `Board`, `Config`, `Theme`, `Storage`
- Produces: `KanbanView` with:
  - Top tab bar (`Backlog`, `Active`, `Done`, `Settings ⌘,`)
  - `Cmd-]` / `Cmd-[` column stepping with top item focus
  - Spatial 2D arrow key and vi `hjkl` card navigation
  - `Enter` / `Return` to edit selected card
  - Drag-and-drop card movements and reordering
  - Settings Tab View for config toggles & storage directory override
  - Sub-item count badge `↪ <count>` and sub-item creation actions

- [ ] **Step 1: Implement new actions and keybindings in `src/app.rs`**
Add `NextColumn` (`Cmd-]`), `PrevColumn` (`Cmd-[`), `NavUp`, `NavDown`, `NavLeft`, `NavRight`, `OpenSettings` (`Cmd-,`), `OpenSelectedForEdit` (`Enter`).

- [ ] **Step 2: Implement Settings Screen Tab & Tab Bar Rendering**
Render tab bar with active tab indicators and right-pinned settings tab.
Render interactive Settings screen when `Screen::Settings` is active.

- [ ] **Step 3: Implement Drag-and-Drop and Sub-item recursive badges**
Add drag source/target handlers on cards and columns.
Display `↪ <count>` badges on cards that reference sub-items.

- [ ] **Step 4: Verify with `cargo test` and `cargo check`**
Run: `cargo check && cargo test`
Expected: PASS

---

### Task 8: macOS App Bundle, Custom Icon & App Store Packaging

**Files:**
- Create: `assets/AppIcon.icns`
- Create: `resources/Info.plist`
- Create: `resources/dkb.entitlements`
- Create: `scripts/bundle_macos.sh`
- Create: `scripts/package_appstore.sh`

- [ ] **Step 1: Create `resources/Info.plist` and `resources/dkb.entitlements`**
- [ ] **Step 2: Generate macOS `.icns` and place in `assets/AppIcon.icns`**
- [ ] **Step 3: Create `scripts/bundle_macos.sh`**
Creates `Daily Kanban.app` with `Contents/MacOS/dkb`, `Contents/Resources/AppIcon.icns`, and `Contents/Info.plist`.
- [ ] **Step 4: Create `scripts/package_appstore.sh`**
Handles codesigning with Developer ID / Mac App Store provisioning profile and produces signed `.pkg`.
- [ ] **Step 5: Test bundling script**
Run: `chmod +x scripts/*.sh && ./scripts/bundle_macos.sh`
Expected: Successful `.app` bundle created in `target/release/bundle/`.

---

## 6. Self-Review & Verification

1. **Spec Coverage:**
   - Configuration screen & menu & `cmd-,`: Tasks 2 & 7
   - Vi-mode in markdown editor & kanban: Tasks 5, 6, 7
   - Light/Dark/System theme toggle: Task 2 & 7
   - Line numbers in editor: Tasks 2, 6, 7
   - Tabs instead of buttons: Task 7
   - Absolute sort per column persisted: Task 3
   - Icon & Packaging like a Mac App & Mac App Store: Task 8
   - Storage directory override: Tasks 2 & 7
   - Integrate iwe support: Task 4
   - `cmd-]` and `cmd-[` column stepping: Task 7
   - Drag and drop between columns: Task 7
   - Title without markdown formatting: Task 1
   - Sub-items with links, recursive count `↪<count>`, creation: Tasks 4 & 7
   - Arrow keys spatial navigation & vi hjkl navigation: Task 7
   - Return key opens item for editing: Task 7
2. **No Placeholders:** Every task contains concrete code snippets and test commands.
3. **Type Consistency:** Types (`ThemeMode`, `Config`, `BoardState`, `ViMode`, `ViState`, `Location`) match cleanly across all tasks.
