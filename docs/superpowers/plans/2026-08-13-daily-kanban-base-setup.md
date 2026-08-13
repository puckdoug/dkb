# Daily Kanban Base Setup — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the base setup of a native macOS kanban app using GPUI from Zed, with markdown file persistence and a filesystem-enforced item lifecycle.

**Architecture:** Library + binary crate. Domain logic (item, board, storage, config) in library modules, GPUI UI in `app.rs`, entry point in `bin/dkb.rs`. Items are markdown files with YAML frontmatter, organized in a directory tree that encodes status (backlog/active/done) and category (yesterday/today/this_week/next_week).

**Tech Stack:** Rust 2024 edition, GPUI (git dep from zed-industries/zed), serde + serde_yaml for frontmatter, uuid for item IDs, chrono for timestamps, tempfile for test isolation.

**Spec:** `docs/superpowers/specs/2026-08-13-daily-kanban-base-setup-design.md`

## Global Constraints

- Rust edition 2024 (requires Rust 1.85+ stable or recent nightly)
- GPUI and gpui_platform are git dependencies from `https://github.com/zed-industries/zed` — do NOT use the crates.io `gpui` fork
- gpui_platform must have the `font-kit` feature enabled for macOS glyph rendering
- macOS only — rendering uses Metal via GPUI
- Xcode and Xcode command line tools must be installed (GPUI requirement)
- All domain logic tests use `tempfile::TempDir` for filesystem isolation — no test touches the real filesystem
- TDD: every task writes the failing test first, verifies it fails, then implements minimal code to pass
- Item ID is the filename (`<uuid>.md`); it is NOT duplicated in frontmatter
- Status and Category are derived from directory path, not stored in frontmatter or the Item struct
- Commit after each task using `cargo test` as the verification gate

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies and binary config |
| `src/lib.rs` | Module declarations only |
| `src/item.rs` | `Item` struct, `Status`/`Category` enums, frontmatter parsing/serialization, title extraction |
| `src/storage.rs` | `Location` enum, filesystem CRUD: init, load_board, write_item, move_item, delete_item, parse_item |
| `src/board.rs` | `Board`/`ActiveColumns` structs, lifecycle transition logic |
| `src/config.rs` | `Config` struct, load/create config.toml, data_dir resolution |
| `src/app.rs` | `KanbanView` root GPUI view, `Screen` enum, actions, keybindings, menus, rendering |
| `src/bin/dkb.rs` | `main()` — Application::run, window creation, menu setup |
| `tests/item.rs` | Item parsing/serialization/title tests |
| `tests/storage.rs` | Storage CRUD and lifecycle tests |
| `tests/board.rs` | Board transition logic tests |

---

### Task 1: Project Scaffolding and Dependencies

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/bin/dkb.rs`
- Modify: `src/main.rs` (delete)

**Interfaces:**
- Produces: a compilable crate with GPUI dependencies, empty `lib.rs`, and a minimal `main` that opens a blank GPUI window

- [x] **Step 1: Update Cargo.toml with all dependencies**

Replace the entire contents of `Cargo.toml` with:

```toml
[package]
name = "dkb"
version = "0.1.0"
edition = "2024"

[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
gpui = { git = "https://github.com/zed-industries/zed", features = ["test-support"] }
tempfile = "3"

[[bin]]
name = "dkb"
path = "src/bin/dkb.rs"
```

- [x] **Step 2: Create src/lib.rs with module declarations**

Create `src/lib.rs`:

```rust
pub mod app;
pub mod board;
pub mod config;
pub mod item;
pub mod storage;
```

- [x] **Step 3: Create stub modules so the crate compiles**

Create each of these files with empty content (just a comment):

`src/app.rs`:
```rust
// GPUI root view — implemented in later tasks
```

`src/board.rs`:
```rust
// Board state and lifecycle — implemented in later tasks
```

`src/config.rs`:
```rust
// Config loading — implemented in later tasks
```

`src/item.rs`:
```rust
// Item domain model — implemented in later tasks
```

`src/storage.rs`:
```rust
// Filesystem storage — implemented in later tasks
```

- [x] **Step 4: Create src/bin/dkb.rs with minimal GPUI window**

Create `src/bin/dkb.rs`:

```rust
use gpui::{App, WindowOptions, div, prelude::*, px, rgb, size};
use gpui_platform::application;

fn main() {
    application().run(|cx: &mut App| {
        cx.activate(true);

        let opts = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1000.), px(700.)),
                cx,
            ))),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Daily Kanban".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(opts, |_, cx| {
            cx.new(|_cx| dkb::app::KanbanView::new(_cx))
        })
        .unwrap();
    });
}
```

- [x] **Step 5: Add minimal KanbanView to app.rs so it compiles**

Replace `src/app.rs` with:

```rust
use gpui::{Context, FocusHandle, Focusable, Render, Window, div, prelude::*, rgb};

pub struct KanbanView {
    focus_handle: FocusHandle,
}

impl KanbanView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for KanbanView {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KanbanView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .size_full()
            .track_focus(&self.focus_handle)
    }
}
```

- [x] **Step 6: Delete src/main.rs**

```bash
rm src/main.rs
```

- [x] **Step 7: Verify the crate builds**

Run: `cargo build`
Expected: compiles successfully (may take a while fetching GPUI from git)

- [x] **Step 8: Commit**

```bash
git add -A && git commit -m "feat: scaffold project with GPUI dependencies and blank window"
```

---

### Task 2: Item Domain Model — Struct and Frontmatter Parsing

**Files:**
- Modify: `src/item.rs`
- Create: `tests/item.rs`

**Interfaces:**
- Produces: `Item` struct, `Status` enum, `Category` enum, `Item::new(title)`, `Item::title()`, `Item::extract_title(body)`, `Item::parse_frontmatter(content)`, `Item::serialize(&self)`, `ItemFrontmatter` struct (serde)

- [x] **Step 1: Write the failing test for Item creation and title extraction**

Create `tests/item.rs`:

```rust
use chrono::Utc;
use dkb::item::{Item, Status, Category};
use uuid::Uuid;

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
```

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --test item`
Expected: FAIL — `item` module has no public types

- [x] **Step 3: Implement the Item struct, enums, and basic methods**

Replace `src/item.rs` with:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Backlog,
    Active,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Yesterday,
    Today,
    ThisWeek,
    NextWeek,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Item {
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            body: title.to_string(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn title(&self) -> String {
        Self::extract_title(&self.body)
    }

    pub fn extract_title(body: &str) -> String {
        body.lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
            .to_string()
    }
}
```

- [x] **Step 4: Run test to verify it passes**

Run: `cargo test --test item`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/item.rs tests/item.rs && git commit -m "feat: add Item struct, Status/Category enums, title extraction"
```

---

### Task 3: Item Frontmatter Serialization and Parsing

**Files:**
- Modify: `src/item.rs`
- Modify: `tests/item.rs`

**Interfaces:**
- Produces: `ItemFrontmatter` struct, `Item::serialize(&self) -> String`, `Item::parse_frontmatter(content) -> (ItemFrontmatter, String)`

- [x] **Step 1: Write the failing tests for frontmatter round-trip**

Append to `tests/item.rs`:

```rust
use dkb::item::ItemFrontmatter;

#[test]
fn test_serialize_item_with_frontmatter() {
    let mut item = Item::new("My task title");
    item.body = "My task title\n\nSome **markdown** body.".to_string();
    let serialized = item.serialize();
    assert!(serialized.starts_with("---\n"));
    assert!(serialized.contains("created_at:"));
    assert!(serialized.contains("updated_at:"));
    assert!(serialized.contains("completed_at: null"));
    assert!(serialized.contains("---\nMy task title"));
}

#[test]
fn test_parse_frontmatter() {
    let content = "---\ncreated_at: 2026-08-13T10:30:00Z\nupdated_at: 2026-08-13T14:22:00Z\ncompleted_at: null\n---\nFix the login bug\n\nDetails here";
    let (frontmatter, body) = Item::parse_frontmatter(content).unwrap();
    assert_eq!(body, "Fix the login bug\n\nDetails here");
    assert!(frontmatter.completed_at.is_none());
}

#[test]
fn test_parse_frontmatter_with_completed_at() {
    let content = "---\ncreated_at: 2026-08-13T10:30:00Z\nupdated_at: 2026-08-13T14:22:00Z\ncompleted_at: 2026-08-13T16:00:00Z\n---\nDone task";
    let (frontmatter, body) = Item::parse_frontmatter(content).unwrap();
    assert!(frontmatter.completed_at.is_some());
    assert_eq!(body, "Done task");
}

#[test]
fn test_round_trip_serialize_parse() {
    let mut item = Item::new("Round trip test");
    item.body = "Round trip test\n\nBody text".to_string();
    let serialized = item.serialize();
    let (frontmatter, body) = Item::parse_frontmatter(&serialized).unwrap();
    assert_eq!(body, "Round trip test\n\nBody text");
    assert_eq!(frontmatter.created_at, item.created_at);
    assert_eq!(frontmatter.updated_at, item.updated_at);
    assert_eq!(frontmatter.completed_at, item.completed_at);
}

#[test]
fn test_parse_frontmatter_no_frontmatter() {
    // No frontmatter delimiters — should return empty frontmatter and full body
    let content = "Just a body\nwith text";
    let (frontmatter, body) = Item::parse_frontmatter(content).unwrap();
    assert_eq!(body, "Just a body\nwith text");
    assert!(frontmatter.created_at.is_none());
}
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --test item`
Expected: FAIL — `ItemFrontmatter`, `serialize`, `parse_frontmatter` do not exist

- [x] **Step 3: Implement frontmatter serialization and parsing**

Add to `src/item.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ItemFrontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

const FRONTMATTER_DELIMITER: &str = "---";

impl Item {
    pub fn serialize(&self) -> String {
        let frontmatter = ItemFrontmatter {
            created_at: Some(self.created_at),
            updated_at: Some(self.updated_at),
            completed_at: self.completed_at,
        };
        let yaml = serde_yaml::to_string(&frontmatter).unwrap_or_default();
        format!("{}\n{}\n{}\n{}", FRONTMATTER_DELIMITER, yaml.trim_end(), FRONTMATTER_DELIMITER, self.body)
    }

    pub fn parse_frontmatter(content: &str) -> Option<(ItemFrontmatter, String)> {
        let content = content.trim_start_matches('\u{feff}');
        if !content.starts_with(FRONTMATTER_DELIMITER) {
            return Some((ItemFrontmatter::default(), content.to_string()));
        }
        let after_first_delim = &content[FRONTMATTER_DELIMITER.len()..];
        let yaml_end = after_first_delim
            .find(&format!("\n{}", FRONTMATTER_DELIMITER))
            .or_else(|| after_first_delim.find(FRONTMATTER_DELIMITER))?;
        let yaml_str = after_first_delim[..yaml_end].trim();
        let body_start = yaml_end + after_first_delim[yaml_end..]
            .find('\n')
            .map(|i| i + 1)
            .unwrap_or(after_first_delim.len());
        let body = after_first_delim
            .get(body_start..)
            .unwrap_or("")
            .trim_start_matches('\n')
            .to_string();
        let frontmatter: ItemFrontmatter = if yaml_str.is_empty() {
            ItemFrontmatter::default()
        } else {
            serde_yaml::from_str(yaml_str).ok()?
        };
        Some((frontmatter, body))
    }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test --test item`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/item.rs tests/item.rs && git commit -m "feat: add frontmatter serialization and parsing for Item"
```

---

### Task 4: Storage Layer — Location Enum and Init

**Files:**
- Modify: `src/storage.rs`
- Create: `tests/storage.rs`

**Interfaces:**
- Consumes: `Item` from Task 2, `ItemFrontmatter` from Task 3
- Produces: `Location` enum with `to_path()`, `from_path()`, `status()`, `category()`; `Storage::init(data_dir)`

- [x] **Step 1: Write the failing test for Location and init**

Create `tests/storage.rs`:

```rust
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
    // Calling again should not error
    Storage::init(&data_dir).unwrap();
}
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --test storage`
Expected: FAIL — `Location`, `Storage` do not exist

- [x] **Step 3: Implement Location enum and Storage::init**

Replace `src/storage.rs` with:

```rust
use std::path::{Path, PathBuf};

use crate::item::{Category, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Backlog,
    Active(Category),
    Done,
}

impl Location {
    pub fn to_path(&self) -> PathBuf {
        match self {
            Location::Backlog => PathBuf::from("backlog"),
            Location::Active(cat) => PathBuf::from("active").join(match cat {
                Category::Yesterday => "yesterday",
                Category::Today => "today",
                Category::ThisWeek => "this_week",
                Category::NextWeek => "next_week",
            }),
            Location::Done => PathBuf::from("done"),
        }
    }

    pub fn from_path(path: &str) -> Self {
        let path = path.trim_end_matches('/');
        match path {
            "backlog" => Location::Backlog,
            "done" => Location::Done,
            "active/yesterday" => Location::Active(Category::Yesterday),
            "active/today" => Location::Active(Category::Today),
            "active/this_week" => Location::Active(Category::ThisWeek),
            "active/next_week" => Location::Active(Category::NextWeek),
            _ => Location::Backlog,
        }
    }

    pub fn status(&self) -> Status {
        match self {
            Location::Backlog => Status::Backlog,
            Location::Active(_) => Status::Active,
            Location::Done => Status::Done,
        }
    }

    pub fn category(&self) -> Option<Category> {
        match self {
            Location::Active(cat) => Some(*cat),
            _ => None,
        }
    }
}

pub struct Storage;

impl Storage {
    pub fn init(data_dir: &Path) -> std::io::Result<()> {
        let dirs = [
            "backlog",
            "active/yesterday",
            "active/today",
            "active/this_week",
            "active/next_week",
            "done",
        ];
        for dir in &dirs {
            std::fs::create_dir_all(data_dir.join(dir))?;
        }
        Ok(())
    }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test --test storage`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/storage.rs tests/storage.rs && git commit -m "feat: add Location enum and Storage::init for directory creation"
```

---

### Task 5: Storage Layer — Write and Read Items

**Files:**
- Modify: `src/storage.rs`
- Modify: `tests/storage.rs`

**Interfaces:**
- Consumes: `Item`, `ItemFrontmatter`, `Location` from prior tasks
- Produces: `Storage::write_item(data_dir, item, location)`, `Storage::parse_item(path) -> Item`, `Storage::read_item(data_dir, id, location) -> Item`

- [x] **Step 1: Write the failing tests for write and read**

Append to `tests/storage.rs`:

```rust
use dkb::item::Item;

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
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --test storage`
Expected: FAIL — `write_item`, `read_item` do not exist

- [x] **Step 3: Implement write_item, parse_item, read_item**

Add to `src/storage.rs` (inside `impl Storage`):

```rust
use crate::item::{Item, ItemFrontmatter};
use uuid::Uuid;

impl Storage {
    pub fn write_item(
        data_dir: &Path,
        item: &Item,
        location: &Location,
    ) -> std::io::Result<()> {
        let dir = data_dir.join(location.to_path());
        std::fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("{}.md", item.id));
        std::fs::write(file_path, item.serialize())?;
        Ok(())
    }

    pub fn read_item(
        data_dir: &Path,
        id: &Uuid,
        location: &Location,
    ) -> std::io::Result<Item> {
        let file_path = data_dir
            .join(location.to_path())
            .join(format!("{}.md", id));
        let content = std::fs::read_to_string(file_path)?;
        Self::parse_item_from_content(id, content)
    }

    pub fn parse_item_from_content(id: &Uuid, content: String) -> std::io::Result<Item> {
        let (frontmatter, body) = Item::parse_frontmatter(&content)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to parse frontmatter"))?;
        Ok(Item {
            id: *id,
            body,
            created_at: frontmatter.created_at.unwrap_or_default(),
            updated_at: frontmatter.updated_at.unwrap_or_default(),
            completed_at: frontmatter.completed_at,
        })
    }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test --test storage`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/storage.rs tests/storage.rs && git commit -m "feat: add Storage write_item and read_item for item persistence"
```

---

### Task 6: Storage Layer — Move and Delete Items

**Files:**
- Modify: `src/storage.rs`
- Modify: `tests/storage.rs`

**Interfaces:**
- Produces: `Storage::move_item(data_dir, id, from, to) -> Item` (returns updated item with refreshed timestamps), `Storage::delete_item(data_dir, id, location)`

- [x] **Step 1: Write the failing tests for move and delete**

Append to `tests/storage.rs`:

```rust
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
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --test storage`
Expected: FAIL — `move_item`, `delete_item` do not exist

- [x] **Step 3: Implement move_item and delete_item**

Add to `src/storage.rs` (inside `impl Storage`):

```rust
use chrono::Utc;

impl Storage {
    pub fn move_item(
        data_dir: &Path,
        id: &Uuid,
        from: &Location,
        to: &Location,
    ) -> std::io::Result<Item> {
        let from_path = data_dir
            .join(from.to_path())
            .join(format!("{}.md", id));
        let to_dir = data_dir.join(to.to_path());
        std::fs::create_dir_all(&to_dir)?;
        let to_path = to_dir.join(format!("{}.md", id));

        // Read current item
        let content = std::fs::read_to_string(&from_path)?;
        let (frontmatter, body) = Item::parse_frontmatter(&content)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to parse frontmatter"))?;

        let now = Utc::now();
        let completed_at = match to.status() {
            Status::Done => Some(now),
            _ => {
                // Clear completed_at if moving away from done
                if from.status() == Status::Done {
                    None
                } else {
                    frontmatter.completed_at
                }
            }
        };

        let updated_item = Item {
            id: *id,
            body,
            created_at: frontmatter.created_at.unwrap_or_default(),
            updated_at: now,
            completed_at,
        };

        // Write to new location, then remove old
        std::fs::write(&to_path, updated_item.serialize())?;
        std::fs::remove_file(from_path)?;

        Ok(updated_item)
    }

    pub fn delete_item(
        data_dir: &Path,
        id: &Uuid,
        location: &Location,
    ) -> std::io::Result<()> {
        let path = data_dir
            .join(location.to_path())
            .join(format!("{}.md", id));
        std::fs::remove_file(path)?;
        Ok(())
    }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test --test storage`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/storage.rs tests/storage.rs && git commit -m "feat: add Storage move_item and delete_item with timestamp management"
```

---

### Task 7: Storage Layer — Load Board

**Files:**
- Modify: `src/storage.rs`
- Modify: `src/board.rs`
- Modify: `tests/storage.rs`

**Interfaces:**
- Consumes: `Item`, `Location`, all storage methods from prior tasks
- Produces: `Board` struct, `ActiveColumns` struct, `Storage::load_board(data_dir) -> Board`

- [x] **Step 1: Write the failing tests for load_board**

Append to `tests/storage.rs`:

```rust
use dkb::board::Board;

#[test]
fn test_load_board_empty() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    assert!(board.backlog.is_empty());
    assert!(board.active.yesterday.is_empty());
    assert!(board.active.today.is_empty());
    assert!(board.active.this_week.is_empty());
    assert!(board.active.next_week.is_empty());
    assert!(board.done.is_empty());
}

#[test]
fn test_load_board_with_items() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let item1 = Item::new("Backlog item");
    Storage::write_item(&data_dir, &item1, &Location::Backlog).unwrap();

    let item2 = Item::new("Today item");
    Storage::write_item(&data_dir, &item2, &Location::Active(Category::Today)).unwrap();

    let mut item3 = Item::new("Done item");
    item3.completed_at = Some(chrono::Utc::now());
    Storage::write_item(&data_dir, &item3, &Location::Done).unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    assert_eq!(board.backlog.len(), 1);
    assert_eq!(board.backlog[0].title(), "Backlog item");
    assert_eq!(board.active.today.len(), 1);
    assert_eq!(board.active.today[0].title(), "Today item");
    assert_eq!(board.done.len(), 1);
    assert!(board.done[0].completed_at.is_some());
}

#[test]
fn test_load_board_done_sorted_by_completed_at_desc() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let mut older = Item::new("Older done");
    older.completed_at = Some(chrono::Utc::now() - chrono::Duration::hours(2));
    Storage::write_item(&data_dir, &older, &Location::Done).unwrap();

    let mut newer = Item::new("Newer done");
    newer.completed_at = Some(chrono::Utc::now());
    Storage::write_item(&data_dir, &newer, &Location::Done).unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    assert_eq!(board.done.len(), 2);
    assert_eq!(board.done[0].title(), "Newer done");
    assert_eq!(board.done[1].title(), "Older done");
}
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --test storage`
Expected: FAIL — `Board`, `load_board` do not exist

- [x] **Step 3: Implement Board struct and load_board**

Replace `src/board.rs` with:

```rust
use crate::item::Item;

#[derive(Debug, Clone, Default)]
pub struct ActiveColumns {
    pub yesterday: Vec<Item>,
    pub today: Vec<Item>,
    pub this_week: Vec<Item>,
    pub next_week: Vec<Item>,
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    pub backlog: Vec<Item>,
    pub active: ActiveColumns,
    pub done: Vec<Item>,
}
```

Add to `src/storage.rs` (inside `impl Storage`):

```rust
use crate::board::Board;

impl Storage {
    pub fn load_board(data_dir: &Path) -> std::io::Result<Board> {
        let mut board = Board::default();

        let locations: [(Location, fn(&mut Board, Vec<Item>)); 6] = [
            (Location::Backlog, |b, items| b.backlog = items),
            (Location::Active(Category::Yesterday), |b, items| b.active.yesterday = items),
            (Location::Active(Category::Today), |b, items| b.active.today = items),
            (Location::Active(Category::ThisWeek), |b, items| b.active.this_week = items),
            (Location::Active(Category::NextWeek), |b, items| b.active.next_week = items),
            (Location::Done, |b, items| b.done = items),
        ];

        for (location, setter) in locations {
            let dir = data_dir.join(location.to_path());
            if !dir.exists() {
                continue;
            }
            let mut items: Vec<Item> = Vec::new();
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let Ok(id) = Uuid::parse_str(file_stem) else {
                    continue;
                };
                let content = std::fs::read_to_string(&path)?;
                match Self::parse_item_from_content(&id, content) {
                    Ok(item) => items.push(item),
                    Err(_) => continue,
                }
            }
            setter(&mut board, items);
        }

        // Sort done by completed_at descending
        board.done.sort_by(|a, b| {
            b.completed_at
                .unwrap_or_default()
                .cmp(&a.completed_at.unwrap_or_default())
        });

        Ok(board)
    }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test --test storage`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/storage.rs src/board.rs tests/storage.rs && git commit -m "feat: add Board struct and Storage::load_board to scan items from disk"
```

---

### Task 8: Config Layer

**Files:**
- Modify: `src/config.rs`
- Create: `tests/config.rs`

**Interfaces:**
- Produces: `Config` struct with `data_dir: PathBuf`, `Config::load() -> Config`, `Config::default_data_dir() -> PathBuf`, `Config::config_file_path() -> PathBuf`

- [x] **Step 1: Write the failing tests for config**

Create `tests/config.rs`:

```rust
use dkb::config::Config;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_default_data_dir() {
    let dir = Config::default_data_dir();
    assert!(dir.to_string_lossy().contains("dkb"));
}

#[test]
fn test_config_load_creates_default_when_missing() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    // Set HOME so the config loader uses our temp dir
    // We'll test the public API: load with a custom config path
    let config = Config::load_from(&config_path).unwrap();
    assert!(config_path.exists());
    assert!(config.data_dir.to_string_lossy().contains("dkb"));
}

#[test]
fn test_config_load_reads_existing() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    let custom_data_dir = tmp.path().join("custom_data");
    std::fs::write(
        &config_path,
        format!("data_dir = \"{}\"", custom_data_dir.display()),
    )
    .unwrap();

    let config = Config::load_from(&config_path).unwrap();
    assert_eq!(config.data_dir, custom_data_dir);
}

#[test]
fn test_config_expands_tilde() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    std::fs::write(&config_path, "data_dir = \"~/dkb-data\"\n").unwrap();

    let config = Config::load_from(&config_path).unwrap();
    assert!(!config.data_dir.to_string_lossy().contains("~"));
    assert!(config.data_dir.to_string_lossy().contains("dkb-data"));
}
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --test config`
Expected: FAIL — `Config` does not exist

- [x] **Step 3: Implement Config**

Replace `src/config.rs` with:

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
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
            let default_dir = Self::default_data_dir();
            std::fs::create_dir_all(config_path.parent().unwrap_or(Path::new(".")))?;
            let content = format!("data_dir = \"{}\"\n", default_dir.display());
            std::fs::write(config_path, content)?;
            return Ok(Self { data_dir: default_dir });
        }

        let content = std::fs::read_to_string(config_path)?;
        let data_dir_str = content
            .lines()
            .find_map(|line| {
                let line = line.trim();
                line.strip_prefix("data_dir")
                    .and_then(|s| s.trim_start())
                    .and_then(|s| s.strip_prefix('='))
                    .map(|s| s.trim())
                    .and_then(|s| {
                        s.trim_matches('"')
                            .trim_matches('\'')
                            .to_string()
                            .into()
                    })
            })
            .unwrap_or_else(|| Self::default_data_dir().to_string_lossy().to_string());

        let data_dir = Self::expand_tilde(&data_dir_str);

        Ok(Self { data_dir })
    }

    fn expand_tilde(path: &str) -> PathBuf {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                return PathBuf::from(home).join(rest);
            }
        }
        PathBuf::from(path)
    }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test --test config`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/config.rs tests/config.rs && git commit -m "feat: add Config layer with TOML loading and tilde expansion"
```

---

### Task 9: Board Lifecycle Transition Logic

**Files:**
- Modify: `src/board.rs`
- Create: `tests/board.rs`

**Interfaces:**
- Consumes: `Board`, `Item`, `Location`, `Status`, `Category` from prior tasks
- Produces: `Board::find_item(&self, id) -> Option<&Item>`, `Board::find_item_location(&self, id) -> Option<Location>`, `Board::can_move(id, to_location) -> bool`, `Board::move_item(&mut self, id, to_location) -> Option<(Location, Location)>` (returns from/to locations so the caller can do the filesystem move)

- [x] **Step 1: Write the failing tests for board transitions**

Create `tests/board.rs`:

```rust
use dkb::board::Board;
use dkb::item::{Category, Item, Location};
use dkb::storage::Storage;
use tempfile::TempDir;

fn make_board_with_items() -> (TempDir, Board) {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().to_path_buf();
    Storage::init(&data_dir).unwrap();

    let backlog_item = Item::new("Backlog task");
    let today_item = Item::new("Today task");
    let done_item = {
        let mut i = Item::new("Done task");
        i.completed_at = Some(chrono::Utc::now());
        i
    };

    Storage::write_item(&data_dir, &backlog_item, &Location::Backlog).unwrap();
    Storage::write_item(&data_dir, &today_item, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(&data_dir, &done_item, &Location::Done).unwrap();

    let board = Storage::load_board(&data_dir).unwrap();
    (tmp, board)
}

#[test]
fn test_find_item_in_backlog() {
    let (_tmp, board) = make_board_with_items();
    let id = board.backlog[0].id;
    let found = board.find_item(&id);
    assert!(found.is_some());
    assert_eq!(found.unwrap().title(), "Backlog task");
}

#[test]
fn test_find_item_location() {
    let (_tmp, board) = make_board_with_items();
    let backlog_id = board.backlog[0].id;
    let today_id = board.active.today[0].id;
    let done_id = board.done[0].id;

    assert_eq!(board.find_item_location(&backlog_id), Some(Location::Backlog));
    assert_eq!(board.find_item_location(&today_id), Some(Location::Active(Category::Today)));
    assert_eq!(board.find_item_location(&done_id), Some(Location::Done));
}

#[test]
fn test_can_move_backlog_to_active() {
    let (_tmp, board) = make_board_with_items();
    let id = board.backlog[0].id;
    assert!(board.can_move(&id, &Location::Active(Category::Today)));
}

#[test]
fn test_can_move_active_to_done() {
    let (_tmp, board) = make_board_with_items();
    let id = board.active.today[0].id;
    assert!(board.can_move(&id, &Location::Done));
}

#[test]
fn test_can_move_done_to_active() {
    let (_tmp, board) = make_board_with_items();
    let id = board.done[0].id;
    assert!(board.can_move(&id, &Location::Active(Category::Today)));
}

#[test]
fn test_cannot_move_backlog_to_done() {
    let (_tmp, board) = make_board_with_items();
    let id = board.backlog[0].id;
    assert!(!board.can_move(&id, &Location::Done));
}

#[test]
fn test_move_item_updates_board_state() {
    let (_tmp, mut board) = make_board_with_items();
    let id = board.backlog[0].id;
    let from_count = board.backlog.len();

    let result = board.move_item(&id, &Location::Active(Category::Today));
    assert!(result.is_some());
    let (from, to) = result.unwrap();
    assert_eq!(from, Location::Backlog);
    assert_eq!(to, Location::Active(Category::Today));

    assert_eq!(board.backlog.len(), from_count - 1);
    assert_eq!(board.active.today.iter().filter(|i| i.id == id).count(), 1);
}

#[test]
fn test_move_item_to_done_sets_completed_at() {
    let (_tmp, mut board) = make_board_with_items();
    let id = board.active.today[0].id;

    board.move_item(&id, &Location::Done).unwrap();
    let done_item = board.done.iter().find(|i| i.id == id).unwrap();
    assert!(done_item.completed_at.is_some());
}

#[test]
fn test_move_item_from_done_clears_completed_at() {
    let (_tmp, mut board) = make_board_with_items();
    let id = board.done[0].id;

    board.move_item(&id, &Location::Active(Category::Today)).unwrap();
    let reopened = board.active.today.iter().find(|i| i.id == id).unwrap();
    assert!(reopened.completed_at.is_none());
}
```

- [x] **Step 2: Run tests to verify they fail**

Run: `cargo test --test board`
Expected: FAIL — `find_item`, `find_item_location`, `can_move`, `move_item` do not exist

- [x] **Step 3: Implement board transition methods**

Add to `src/board.rs`:

```rust
use crate::item::{Category, Status};
use crate::storage::Location;
use chrono::Utc;
use uuid::Uuid;

impl Board {
    pub fn find_item(&self, id: &Uuid) -> Option<&Item> {
        self.backlog
            .iter()
            .chain(self.active.yesterday.iter())
            .chain(self.active.today.iter())
            .chain(self.active.this_week.iter())
            .chain(self.active.next_week.iter())
            .chain(self.done.iter())
            .find(|item| item.id == *id)
    }

    pub fn find_item_mut(&mut self, id: &Uuid) -> Option<&mut Item> {
        self.backlog
            .iter_mut()
            .chain(self.active.yesterday.iter_mut())
            .chain(self.active.today.iter_mut())
            .chain(self.active.this_week.iter_mut())
            .chain(self.active.next_week.iter_mut())
            .chain(self.done.iter_mut())
            .find(|item| item.id == *id)
    }

    pub fn find_item_location(&self, id: &Uuid) -> Option<Location> {
        if self.backlog.iter().any(|i| i.id == *id) {
            Some(Location::Backlog)
        } else if self.active.yesterday.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::Yesterday))
        } else if self.active.today.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::Today))
        } else if self.active.this_week.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::ThisWeek))
        } else if self.active.next_week.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::NextWeek))
        } else if self.done.iter().any(|i| i.id == *id) {
            Some(Location::Done)
        } else {
            None
        }
    }

    pub fn can_move(&self, id: &Uuid, to: &Location) -> bool {
        let Some(from) = self.find_item_location(id) else {
            return false;
        };
        // No direct backlog -> done
        if from.status() == Status::Backlog && to.status() == Status::Done {
            return false;
        }
        // No direct done -> backlog (must go through active)
        if from.status() == Status::Done && to.status() == Status::Backlog {
            return false;
        }
        // Same location is a no-op, disallow
        if from == *to {
            return false;
        }
        true
    }

    pub fn move_item(
        &mut self,
        id: &Uuid,
        to: &Location,
    ) -> Option<(Location, Location)> {
        if !self.can_move(id, to) {
            return None;
        }
        let from = self.find_item_location(id)?;
        let now = Utc::now();

        let mut item = self.remove_item(id, &from)?;
        item.updated_at = now;
        item.completed_at = match to.status() {
            Status::Done => Some(now),
            _ => {
                if from.status() == Status::Done {
                    None
                } else {
                    item.completed_at
                }
            }
        };

        self.insert_item(item, to);
        Some((from, to.clone()))
    }

    pub fn remove_item(&mut self, id: &Uuid, location: &Location) -> Option<Item> {
        let vec = match location {
            Location::Backlog => &mut self.backlog,
            Location::Active(Category::Yesterday) => &mut self.active.yesterday,
            Location::Active(Category::Today) => &mut self.active.today,
            Location::Active(Category::ThisWeek) => &mut self.active.this_week,
            Location::Active(Category::NextWeek) => &mut self.active.next_week,
            Location::Done => &mut self.done,
        };
        let pos = vec.iter().position(|i| i.id == *id)?;
        Some(vec.remove(pos))
    }

    pub fn insert_item(&mut self, item: Item, location: &Location) {
        match location {
            Location::Backlog => self.backlog.push(item),
            Location::Active(Category::Yesterday) => self.active.yesterday.push(item),
            Location::Active(Category::Today) => self.active.today.push(item),
            Location::Active(Category::ThisWeek) => self.active.this_week.push(item),
            Location::Active(Category::NextWeek) => self.active.next_week.push(item),
            Location::Done => self.done.push(item),
        }
    }
}
```

- [x] **Step 4: Run tests to verify they pass**

Run: `cargo test --test board`
Expected: PASS

- [x] **Step 5: Run all tests to verify nothing broke**

Run: `cargo test`
Expected: PASS

- [x] **Step 6: Commit**

```bash
git add src/board.rs tests/board.rs && git commit -m "feat: add Board lifecycle transitions with state machine validation"
```

---

### Task 10: GPUI App — Actions, Keybindings, and Menus

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `Board`, `Item`, `Location`, `Config`, `Storage` from prior tasks
- Produces: `KanbanView` with full state, GPUI actions, keybindings, menu setup, `Screen` enum

- [ ] **Step 1: Define actions and Screen enum**

Replace `src/app.rs` with the action definitions and Screen enum (no tests yet — this is wiring):

```rust
use gpui::{
    App, Context, FocusHandle, Focusable, KeyBinding, Menu, MenuItem, Render, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};

use crate::board::Board;
use crate::config::Config;
use crate::item::{Category, Status};
use crate::storage::{Location, Storage};
use uuid::Uuid;

actions!(
    dkb,
    [
        NewItem,
        CloseWindow,
        Quit,
        MoveToBacklog,
        MoveToYesterday,
        MoveToToday,
        MoveToThisWeek,
        MoveToNextWeek,
        ToggleDone,
        DeleteItem,
        ShowBacklog,
        ShowActive,
        ShowDone,
    ]
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Backlog,
    Active,
    Done,
}

pub struct KanbanView {
    pub board: Board,
    pub current_screen: Screen,
    pub config: Config,
    pub focus_handle: FocusHandle,
    pub selected_item: Option<Uuid>,
}

impl KanbanView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let config = Config::load().unwrap_or_else(|_| Config {
            data_dir: Config::default_data_dir(),
        });
        Storage::init(&config.data_dir).ok();
        let board = Storage::load_board(&config.data_dir).unwrap_or_default();

        Self {
            board,
            current_screen: Screen::Active,
            config,
            focus_handle: cx.focus_handle(),
            selected_item: None,
        }
    }

    pub fn key_bindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding::new("cmd-n", NewItem, None),
            KeyBinding::new("cmd-w", CloseWindow, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-shift-b", ShowBacklog, None),
            KeyBinding::new("cmd-shift-a", ShowActive, None),
            KeyBinding::new("cmd-shift-d", ShowDone, None),
            KeyBinding::new("cmd-1", MoveToYesterday, None),
            KeyBinding::new("cmd-2", MoveToToday, None),
            KeyBinding::new("cmd-3", MoveToThisWeek, None),
            KeyBinding::new("cmd-4", MoveToNextWeek, None),
            KeyBinding::new("cmd-b", MoveToBacklog, None),
            KeyBinding::new("cmd-d", ToggleDone, None),
            KeyBinding::new("delete", DeleteItem, None),
        ]
    }

    pub fn menus() -> Vec<Menu> {
        vec![
            Menu::new("dkb").items([
                MenuItem::action("Quit", Quit),
            ]),
            Menu::new("File").items([
                MenuItem::action("New Item", NewItem),
                MenuItem::separator(),
                MenuItem::action("Close Window", CloseWindow),
            ]),
            Menu::new("View").items([
                MenuItem::action("Backlog", ShowBacklog),
                MenuItem::action("Active", ShowActive),
                MenuItem::action("Done", ShowDone),
            ]),
            Menu::new("Item").items([
                MenuItem::action("Move to Backlog", MoveToBacklog),
                MenuItem::action("Move to Yesterday", MoveToYesterday),
                MenuItem::action("Move to Today", MoveToToday),
                MenuItem::action("Move to This Week", MoveToThisWeek),
                MenuItem::action("Move to Next Week", MoveToNextWeek),
                MenuItem::separator(),
                MenuItem::action("Mark Done / Reopen", ToggleDone),
                MenuItem::separator(),
                MenuItem::action("Delete", DeleteItem),
            ]),
        ]
    }

    pub fn setup_menus(cx: &mut App) {
        cx.bind_keys(Self::key_bindings());
        cx.set_menus(Self::menus());
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
    }
}

impl Focusable for KanbanView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KanbanView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xf5f5f5))
            .size_full()
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .p(px(8.))
                    .bg(rgb(0xe0e0e0))
                    .child(self.render_tab("Backlog", Screen::Backlog))
                    .child(self.render_tab("Active", Screen::Active))
                    .child(self.render_tab("Done", Screen::Done)),
            )
            .child(
                div()
                    .flex_1()
                    .p(px(16.))
                    .child(self.render_screen_label()),
            )
    }
}

impl KanbanView {
    fn render_tab(&self, label: &str, screen: Screen) -> impl IntoElement {
        let is_active = self.current_screen == screen;
        div()
            .px(px(12.))
            .py(px(6.))
            .rounded(px(4.))
            .bg(if is_active { rgb(0xffffff) } else { rgb(0xe0e0e0) })
            .text_sm()
            .text_color(rgb(0x333333))
            .child(label.to_string())
    }

    fn render_screen_label(&self) -> impl IntoElement {
        let label = match self.current_screen {
            Screen::Backlog => "Backlog Screen",
            Screen::Active => "Active Screen",
            Screen::Done => "Done Screen",
        };
        div().text_color(rgb(0x666666)).child(label.to_string())
    }
}
```

- [ ] **Step 2: Update bin/dkb.rs to use setup_menus**

Replace `src/bin/dkb.rs` with:

```rust
use gpui::{App, Bounds, WindowBounds, WindowOptions, size, px};
use gpui_platform::application;

use dkb::app::KanbanView;

fn main() {
    application().run(|cx: &mut App| {
        cx.activate(true);

        KanbanView::setup_menus(cx);

        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1000.), px(700.)),
                cx,
            ))),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Daily Kanban".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(opts, |_, cx| {
            cx.new(|cx| KanbanView::new(cx))
        })
        .unwrap();
    });
}
```

- [ ] **Step 3: Verify it builds**

Run: `cargo build`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/bin/dkb.rs && git commit -m "feat: add KanbanView with actions, keybindings, menus, and tab bar"
```

---

### Task 11: GPUI App — Screen Switching

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `KanbanView` from Task 10
- Produces: action handlers for `ShowBacklog`, `ShowActive`, `ShowDone`, `CloseWindow`

- [ ] **Step 1: Add action handlers for screen switching and close**

Add to `impl KanbanView` in `src/app.rs`:

```rust
fn on_show_backlog(&mut self, _: &ShowBacklog, _window: &mut Window, cx: &mut Context<Self>) {
    self.current_screen = Screen::Backlog;
    cx.notify();
}

fn on_show_active(&mut self, _: &ShowActive, _window: &mut Window, cx: &mut Context<Self>) {
    self.current_screen = Screen::Active;
    cx.notify();
}

fn on_show_done(&mut self, _: &ShowDone, _window: &mut Window, cx: &mut Context<Self>) {
    self.current_screen = Screen::Done;
    cx.notify();
}

fn on_close_window(&mut self, _: &CloseWindow, window: &mut Window, _cx: &mut Context<Self>) {
    window.remove_window();
}
```

- [ ] **Step 2: Wire actions in render**

Update the `Render` impl for `KanbanView`. Replace the `render` method body to register action handlers on the root div:

```rust
impl Render for KanbanView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xf5f5f5))
            .size_full()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_show_backlog))
            .on_action(cx.listener(Self::on_show_active))
            .on_action(cx.listener(Self::on_show_done))
            .on_action(cx.listener(Self::on_close_window))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .p(px(8.))
                    .bg(rgb(0xe0e0e0))
                    .child(self.render_tab("Backlog", Screen::Backlog, cx))
                    .child(self.render_tab("Active", Screen::Active, cx))
                    .child(self.render_tab("Done", Screen::Done, cx)),
            )
            .child(
                div()
                    .flex_1()
                    .p(px(16.))
                    .child(self.render_screen_label()),
            )
    }
}
```

Update `render_tab` to accept `cx` and handle click:

```rust
fn render_tab(&self, label: &str, screen: Screen, cx: &mut Context<Self>) -> impl IntoElement {
    let is_active = self.current_screen == screen;
    div()
        .px(px(12.))
        .py(px(6.))
        .rounded(px(4.))
        .bg(if is_active { rgb(0xffffff) } else { rgb(0xe0e0e0) })
        .text_sm()
        .text_color(rgb(0x333333))
        .cursor_pointer()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |this, _, _window, cx| {
                this.current_screen = screen;
                cx.notify();
            }),
        )
        .child(label.to_string())
    }
}
```

- [ ] **Step 3: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add src/app.rs && git commit -m "feat: add screen switching via tabs and keyboard shortcuts"
```

---

### Task 12: GPUI App — Item Cards and Column Rendering

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `Board`, `Item` from prior tasks
- Produces: rendering of item cards and columns for each screen

- [ ] **Step 1: Add item card and column rendering methods**

Add to `impl KanbanView` in `src/app.rs`:

```rust
fn render_item_card(&self, item: &crate::item::Item) -> impl IntoElement {
    let is_selected = self.selected_item == Some(item.id);
    div()
        .w_full()
        .p(px(8.))
        .mb(px(4.))
        .rounded(px(4.))
        .bg(if is_selected { rgb(0xe3f2fd) } else { rgb(0xffffff) })
        .border_1()
        .border_color(if is_selected { rgb(0x2196f3) } else { rgb(0xdddddd) })
        .cursor_pointer()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(0x333333))
                .child(item.title()),
        )
}

fn render_column(&self, title: &str, items: &[crate::item::Item]) -> impl IntoElement {
    let item_count = items.len();
    div()
        .flex_1()
        .flex()
        .flex_col()
        .bg(rgb(0xeceff1))
        .rounded(px(4.))
        .p(px(8.))
        .m(px(4.))
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .mb(px(8.))
                .child(div().text_sm().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0x37474f)).child(title.to_string()))
                .child(div().text_xs().text_color(rgb(0x78909c)).child(format!("{}", item_count))),
        )
        .children(
            items
                .iter()
                .map(|item| self.render_item_card(item)),
        )
}

fn render_backlog_screen(&self) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .p(px(8.))
        .child(self.render_column("Backlog", &self.board.backlog))
}

fn render_active_screen(&self) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .p(px(4.))
        .child(self.render_column("Yesterday", &self.board.active.yesterday))
        .child(self.render_column("Today", &self.board.active.today))
        .child(self.render_column("This Week", &self.board.active.this_week))
        .child(self.render_column("Next Week", &self.board.active.next_week))
}

fn render_done_screen(&self) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .p(px(8.))
        .child(self.render_column("Done", &self.board.done))
}
```

- [ ] **Step 2: Update render to use screen-specific rendering**

Replace the `render` method's content area child. The full `Render` impl:

```rust
impl Render for KanbanView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let screen = self.current_screen;
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xf5f5f5))
            .size_full()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_show_backlog))
            .on_action(cx.listener(Self::on_show_active))
            .on_action(cx.listener(Self::on_show_done))
            .on_action(cx.listener(Self::on_close_window))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .p(px(8.))
                    .bg(rgb(0xe0e0e0))
                    .child(self.render_tab("Backlog", Screen::Backlog, cx))
                    .child(self.render_tab("Active", Screen::Active, cx))
                    .child(self.render_tab("Done", Screen::Done, cx)),
            )
            .child(div().flex_1().flex().flex_col().child(match screen {
                Screen::Backlog => self.render_backlog_screen(),
                Screen::Active => self.render_active_screen(),
                Screen::Done => self.render_done_screen(),
            }))
    }
}
```

- [ ] **Step 3: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add src/app.rs && git commit -m "feat: render item cards and columns for all three screens"
```

---

### Task 13: GPUI App — Item Selection and Movement Actions

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `Board::move_item`, `Board::can_move`, `Storage::move_item` from prior tasks
- Produces: action handlers for `MoveToYesterday`, `MoveToToday`, `MoveToThisWeek`, `MoveToNextWeek`, `MoveToBacklog`, `ToggleDone`, `DeleteItem`, mouse-click selection, `Tab`/`Shift-Tab` keyboard navigation

- [ ] **Step 1: Add selection by mouse click**

Update `render_item_card` to handle mouse-down for selection. The method needs `cx` now:

```rust
fn render_item_card(&self, item: &crate::item::Item, cx: &mut Context<Self>) -> impl IntoElement {
    let is_selected = self.selected_item == Some(item.id);
    let item_id = item.id;
    div()
        .w_full()
        .p(px(8.))
        .mb(px(4.))
        .rounded(px(4.))
        .bg(if is_selected { rgb(0xe3f2fd) } else { rgb(0xffffff) })
        .border_1()
        .border_color(if is_selected { rgb(0x2196f3) } else { rgb(0xdddddd) })
        .cursor_pointer()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |this, _, _window, cx| {
                this.selected_item = Some(item_id);
                cx.notify();
            }),
        )
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(0x333333))
                .child(item.title()),
        )
}
```

Update all call sites of `render_item_card` to pass `cx` (in `render_column`):

```rust
fn render_column(&self, title: &str, items: &[crate::item::Item], cx: &mut Context<Self>) -> impl IntoElement {
    let item_count = items.len();
    div()
        .flex_1()
        .flex()
        .flex_col()
        .bg(rgb(0xeceff1))
        .rounded(px(4.))
        .p(px(8.))
        .m(px(4.))
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .mb(px(8.))
                .child(div().text_sm().font_weight(gpui::FontWeight::BOLD).text_color(rgb(0x37474f)).child(title.to_string()))
                .child(div().text_xs().text_color(rgb(0x78909c)).child(format!("{}", item_count))),
        )
        .children(
            items
                .iter()
                .map(|item| self.render_item_card(item, cx)),
        )
}
```

Update `render_backlog_screen`, `render_active_screen`, `render_done_screen` to accept and pass `cx`:

```rust
fn render_backlog_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
    div().flex().flex_row().flex_1().p(px(8.)).child(self.render_column("Backlog", &self.board.backlog, cx))
}

fn render_active_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_1()
        .p(px(4.))
        .child(self.render_column("Yesterday", &self.board.active.yesterday, cx))
        .child(self.render_column("Today", &self.board.active.today, cx))
        .child(self.render_column("This Week", &self.board.active.this_week, cx))
        .child(self.render_column("Next Week", &self.board.active.next_week, cx))
}

fn render_done_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
    div().flex().flex_row().flex_1().p(px(8.)).child(self.render_column("Done", &self.board.done, cx))
}
```

Update the `render` method to pass `cx`:

```rust
.child(div().flex_1().flex().flex_col().child(match screen {
    Screen::Backlog => self.render_backlog_screen(cx),
    Screen::Active => self.render_active_screen(cx),
    Screen::Done => self.render_done_screen(cx),
}))
```

- [ ] **Step 2: Add move action handlers**

Add to `impl KanbanView`:

```rust
fn move_selected_to(&mut self, location: Location, cx: &mut Context<Self>) {
    let Some(id) = self.selected_item else {
        return;
    };
    if !self.board.can_move(&id, &location) {
        return;
    }
    let Some(from) = self.board.find_item_location(&id) else {
        return;
    };
    match Storage::move_item(&self.config.data_dir, &id, &from, &location) {
        Ok(updated_item) => {
            // Remove from old location and insert the disk-validated item
            let _ = self.board.remove_item(&id, &from);
            self.board.insert_item(updated_item, &location);
            cx.notify();
        }
        Err(e) => {
            eprintln!("Failed to move item: {}", e);
        }
    }
}

fn on_move_to_yesterday(&mut self, _: &MoveToYesterday, _window: &mut Window, cx: &mut Context<Self>) {
    self.move_selected_to(Location::Active(Category::Yesterday), cx);
}

fn on_move_to_today(&mut self, _: &MoveToToday, _window: &mut Window, cx: &mut Context<Self>) {
    self.move_selected_to(Location::Active(Category::Today), cx);
}

fn on_move_to_this_week(&mut self, _: &MoveToThisWeek, _window: &mut Window, cx: &mut Context<Self>) {
    self.move_selected_to(Location::Active(Category::ThisWeek), cx);
}

fn on_move_to_next_week(&mut self, _: &MoveToNextWeek, _window: &mut Window, cx: &mut Context<Self>) {
    self.move_selected_to(Location::Active(Category::NextWeek), cx);
}

fn on_move_to_backlog(&mut self, _: &MoveToBacklog, _window: &mut Window, cx: &mut Context<Self>) {
    self.move_selected_to(Location::Backlog, cx);
}

fn on_toggle_done(&mut self, _: &ToggleDone, _window: &mut Window, cx: &mut Context<Self>) {
    let Some(id) = self.selected_item else {
        return;
    };
    let Some(location) = self.board.find_item_location(&id) else {
        return;
    };
    let target = match location.status() {
        Status::Active => Location::Done,
        Status::Done => Location::Active(Category::Today),
        Status::Backlog => return, // can't go directly to done from backlog
    };
    self.move_selected_to(target, cx);
}

fn on_delete_item(&mut self, _: &DeleteItem, _window: &mut Window, cx: &mut Context<Self>) {
    let Some(id) = self.selected_item else {
        return;
    };
    let Some(location) = self.board.find_item_location(&id) else {
        return;
    };
    if Storage::delete_item(&self.config.data_dir, &id, &location).is_ok() {
        let vec = match &location {
            Location::Backlog => &mut self.board.backlog,
            Location::Active(Category::Yesterday) => &mut self.board.active.yesterday,
            Location::Active(Category::Today) => &mut self.board.active.today,
            Location::Active(Category::ThisWeek) => &mut self.board.active.this_week,
            Location::Active(Category::NextWeek) => &mut self.board.active.next_week,
            Location::Done => &mut self.board.done,
        };
        if let Some(pos) = vec.iter().position(|i| i.id == id) {
            vec.remove(pos);
        }
        self.selected_item = None;
        cx.notify();
    }
}
```

- [ ] **Step 3: Wire all action handlers in render**

Add these to the root div in `render`:

```rust
.on_action(cx.listener(Self::on_move_to_yesterday))
.on_action(cx.listener(Self::on_move_to_today))
.on_action(cx.listener(Self::on_move_to_this_week))
.on_action(cx.listener(Self::on_move_to_next_week))
.on_action(cx.listener(Self::on_move_to_backlog))
.on_action(cx.listener(Self::on_toggle_done))
.on_action(cx.listener(Self::on_delete_item))
```

- [ ] **Step 4: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add src/app.rs && git commit -m "feat: add item selection, move actions, delete, and toggle done"
```

---

### Task 14: GPUI App — Quick-Add New Item

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `Item::new`, `Storage::write_item` from prior tasks
- Produces: `NewItem` action handler that creates an item in the current screen's default location

- [ ] **Step 1: Add quick-add state and action handler**

Add `quick_add_input: Option<String>` field to `KanbanView`:

```rust
pub struct KanbanView {
    pub board: Board,
    pub current_screen: Screen,
    pub config: Config,
    pub focus_handle: FocusHandle,
    pub selected_item: Option<Uuid>,
    pub quick_add_input: Option<String>,
}
```

Update `KanbanView::new` to initialize it:

```rust
quick_add_input: None,
```

- [ ] **Step 2: Add the NewItem action handler**

```rust
fn on_new_item(&mut self, _: &NewItem, _window: &mut Window, cx: &mut Context<Self>) {
    self.quick_add_input = Some(String::new());
    cx.notify();
}

fn commit_quick_add(&mut self, cx: &mut Context<Self>) {
    let Some(title) = self.quick_add_input.take() else {
        return;
    };
    let title = title.trim().to_string();
    if title.is_empty() {
        return;
    }
    let item = crate::item::Item::new(&title);
    let location = match self.current_screen {
        Screen::Backlog => Location::Backlog,
        Screen::Active => Location::Active(Category::Today),
        Screen::Done => Location::Backlog, // new items never start in done
    };
    if Storage::write_item(&self.config.data_dir, &item, &location).is_ok() {
        match location {
            Location::Backlog => self.board.backlog.push(item),
            Location::Active(Category::Today) => self.board.active.today.push(item),
            _ => {}
        }
        cx.notify();
    }
}

fn cancel_quick_add(&mut self, cx: &mut Context<Self>) {
    self.quick_add_input = None;
    cx.notify();
}
```

- [ ] **Step 3: Render the quick-add input when active**

Add a method to render the quick-add bar:

```rust
fn render_quick_add(&self, cx: &mut Context<Self>) -> impl IntoElement {
    if self.quick_add_input.is_some() {
        div()
            .p(px(8.))
            .bg(rgb(0xffffff))
            .border_b_1()
            .border_color(rgb(0xcccccc))
            .child(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded(px(4.))
                    .border_1()
                    .border_color(rgb(0x2196f3))
                    .text_sm()
                    .text_color(rgb(0x999999))
                    .child("Type item title, press Enter to create..."),
            )
    } else {
        div()
    }
}
```

- [ ] **Step 4: Wire NewItem action and add quick-add bar to render**

Add `.on_action(cx.listener(Self::on_new_item))` to the root div in `render`.

Insert the quick-add bar between the tab bar and the content area:

```rust
.child(self.render_quick_add(cx))
.child(div().flex_1().flex().flex_col().child(match screen {
    Screen::Backlog => self.render_backlog_screen(cx),
    Screen::Active => self.render_active_screen(cx),
    Screen::Done => self.render_done_screen(cx),
}))
```

- [ ] **Step 5: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 6: Commit**

```bash
git add src/app.rs && git commit -m "feat: add quick-add new item with NewItem action"
```

---

### Task 15: GPUI App — Keyboard Navigation (Tab / Shift-Tab)

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `KanbanView`, `Board` from prior tasks
- Produces: Tab/Shift-Tab handlers that advance/reverse selection through items on the current screen

- [ ] **Step 1: Add a helper to get the ordered item list for the current screen**

Add to `impl KanbanView`:

```rust
fn current_screen_items(&self) -> Vec<(Uuid, Location)> {
    match self.current_screen {
        Screen::Backlog => self
            .board
            .backlog
            .iter()
            .map(|i| (i.id, Location::Backlog))
            .collect(),
        Screen::Active => {
            let mut items = Vec::new();
            items.extend(
                self.board
                    .active
                    .yesterday
                    .iter()
                    .map(|i| (i.id, Location::Active(Category::Yesterday))),
            );
            items.extend(
                self.board
                    .active
                    .today
                    .iter()
                    .map(|i| (i.id, Location::Active(Category::Today))),
            );
            items.extend(
                self.board
                    .active
                    .this_week
                    .iter()
                    .map(|i| (i.id, Location::Active(Category::ThisWeek))),
            );
            items.extend(
                self.board
                    .active
                    .next_week
                    .iter()
                    .map(|i| (i.id, Location::Active(Category::NextWeek))),
            );
            items
        }
        Screen::Done => self
            .board
            .done
            .iter()
            .map(|i| (i.id, Location::Done))
            .collect(),
    }
}
```

- [ ] **Step 2: Add Tab and Shift-Tab action definitions**

Add `NextItem` and `PrevItem` to the `actions!` macro in `src/app.rs`:

```rust
actions!(
    dkb,
    [
        // ... existing actions ...
        NextItem,
        PrevItem,
    ]
);
```

- [ ] **Step 3: Add keybindings for Tab and Shift-Tab**

Add to `key_bindings()`:

```rust
KeyBinding::new("tab", NextItem, None),
KeyBinding::new("shift-tab", PrevItem, None),
```

- [ ] **Step 4: Implement the action handlers**

Add to `impl KanbanView`:

```rust
fn on_next_item(&mut self, _: &NextItem, _window: &mut Window, cx: &mut Context<Self>) {
    let items = self.current_screen_items();
    if items.is_empty() {
        return;
    }
    let next = match self.selected_item {
        None => items[0].0,
        Some(current) => {
            let pos = items.iter().position(|(id, _)| *id == current);
            match pos {
                None => items[0].0,
                Some(idx) => items[(idx + 1) % items.len()].0,
            }
        }
    };
    self.selected_item = Some(next);
    cx.notify();
}

fn on_prev_item(&mut self, _: &PrevItem, _window: &mut Window, cx: &mut Context<Self>) {
    let items = self.current_screen_items();
    if items.is_empty() {
        return;
    }
    let prev = match self.selected_item {
        None => items[items.len() - 1].0,
        Some(current) => {
            let pos = items.iter().position(|(id, _)| *id == current);
            match pos {
                None => items[items.len() - 1].0,
                Some(idx) => {
                    let len = items.len();
                    items[(idx + len - 1) % len].0
                }
            }
        }
    };
    self.selected_item = Some(prev);
    cx.notify();
}
```

- [ ] **Step 5: Wire the actions in render**

Add to the root div in `render`:

```rust
.on_action(cx.listener(Self::on_next_item))
.on_action(cx.listener(Self::on_prev_item))
```

- [ ] **Step 6: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 7: Commit**

```bash
git add src/app.rs && git commit -m "feat: add Tab/Shift-Tab keyboard navigation for item selection"
```

---

### Task 16: Final Verification

**Files:**
- None modified

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 2: Build the release binary**

Run: `cargo build`
Expected: compiles without warnings (or only GPUI-internal warnings)

- [ ] **Step 3: Run the app to verify it launches**

Run: `cargo run`
Expected: a window titled "Daily Kanban" opens with a tab bar (Backlog/Active/Done) and the active screen showing four empty columns (Yesterday/Today/This Week/Next Week)

- [ ] **Step 4: Commit any final fixes**

If any issues were found and fixed during verification:

```bash
git add -A && git commit -m "fix: address issues found during final verification"
```

---

## Deferred Work

The following spec items are intentionally deferred from this base setup plan and should be addressed in follow-up plans:

- **Inline item editing** (double-click to edit markdown body): requires a text input component integrated with GPUI's `EntityInputHandler`, which is complex enough to warrant its own plan
- **GPUI view tests** (`gpui::test` macro with `TestAppContext`): the spec calls for these, but the GPUI test API requires careful setup; recommend adding after the base UI is play-tested and stable
- **Delete confirmation prompt**: the `DeleteItem` action currently deletes immediately; a confirmation dialog should be added before final release
