# Daily Kanban (dkb) — Design Spec

## Overview

Daily Kanban is a native macOS application built with the GPUI framework from Zed. It provides a kanban board for tracking work items organized by time horizon (yesterday, today, this week, next week), plus a backlog and done screen. Items are stored as individual markdown files on disk, with the filesystem directory structure enforcing the item lifecycle state machine.

## Decisions

- **GPUI dependency**: Git dependency from `https://github.com/zed-industries/zed` (not the unofficial crates.io fork). Uses `gpui` + `gpui_platform` with the `font-kit` feature for macOS glyph rendering.
- **Persistence**: One markdown file per item, stored in a directory structure that encodes status and category. YAML frontmatter holds timestamps only.
- **Project structure**: Library + binary (mirrors the `st` project pattern). Domain logic in `lib.rs` modules, GPUI entry point in `bin/dkb.rs`.
- **Data directory**: Configurable via `config.toml`, defaulting to `~/Library/Application Support/dkb/`.

## Project Structure

```
dkb/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Module declarations
│   ├── bin/
│   │   └── dkb.rs          # GPUI entry point (Application::run, window setup, menus)
│   ├── app.rs              # Root GPUI view (KanbanView), window management, keybindings
│   ├── item.rs             # Item domain model (id, title, body, frontmatter parsing)
│   ├── board.rs            # Board state: collection of items, column/screen transitions
│   ├── storage.rs          # Filesystem operations: read/write/move item files
│   └── config.rs           # Config loading, data_dir resolution
└── tests/
    ├── item.rs
    ├── board.rs
    └── storage.rs
```

### Cargo.toml

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

## Domain Model

### Item

```rust
pub struct Item {
    pub id: Uuid,              // Also the filename; not duplicated in frontmatter
    pub title: String,         // First line of markdown body
    pub body: String,          // Full markdown body (including title line)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

### Status and Category

```rust
pub enum Status {
    Backlog,
    Active,
    Done,
}

pub enum Category {
    Yesterday,
    Today,
    ThisWeek,
    NextWeek,
}
```

Status and category are derived from the file's location in the directory tree, not from frontmatter fields. This makes invalid states (e.g. "done/next_week" or "backlog/yesterday") impossible at the filesystem level.

The `Item` struct does not store `Status` or `Category` — they are implicit from which field of `Board` the item lives in. `load_board` uses the file's directory path to route each item into the correct `Board` field. The `Status` and `Category` enums are used in transition logic (e.g. validating moves) and in UI rendering (determining which column to display an item in).

### File Format

Each item is a file named `<uuid>.md`:

```markdown
---
created_at: 2026-08-13T10:30:00Z
updated_at: 2026-08-13T14:22:00Z
completed_at: null
---
Fix the login bug

The rest of the body in **markdown**.
Details go here.
```

The first non-empty line of the body (after frontmatter) is the title. The remaining lines are detail content. Both are stored in `body`; `title` is extracted for display.

### Directory Structure

```
<data_dir>/
  config.toml
  backlog/
    <uuid>.md
  active/
    yesterday/
      <uuid>.md
    today/
      <uuid>.md
    this_week/
      <uuid>.md
    next_week/
      <uuid>.md
  done/
    <uuid>.md
```

### Board State (in-memory)

```rust
pub struct Board {
    pub backlog: Vec<Item>,
    pub active: ActiveColumns,
    pub done: Vec<Item>,
}

pub struct ActiveColumns {
    pub yesterday: Vec<Item>,
    pub today: Vec<Item>,
    pub this_week: Vec<Item>,
    pub next_week: Vec<Item>,
}
```

The board loads from the filesystem on startup and after any mutation. Each mutation (move, create, edit, delete) writes to disk first, then updates the in-memory state.

### Item Lifecycle

Transitions:
- `backlog` to `active/<category>` — user assigns a backlog item to a time horizon
- `active/<category>` to `active/<other_category>` — user re-categorizes
- `active/<category>` to `done` — user completes item; sets `completed_at` timestamp
- `done` to `active/<category>` — user reopens; clears `completed_at`

There is no direct `backlog` to `done` path. Items must pass through active to be completed.

When an item moves to `done`, `completed_at` is set to the current time and `updated_at` is refreshed. When an item is reopened from `done`, `completed_at` is cleared (set to null) and `updated_at` is refreshed.

### Config

Config file at `~/Library/Application Support/dkb/config.toml`:

```toml
data_dir = "~/Library/Application Support/dkb"
```

If the config file does not exist on startup, the app creates it with default values. The `data_dir` path is expanded (tilde expansion) and used as the root for the backlog/active/done directory tree. If the directory structure does not exist, the app creates it on first run.

## Storage Layer

`storage.rs` provides filesystem operations:

- `init(data_dir)` — creates the directory structure if it does not exist
- `load_board(data_dir) -> Board` — scans all directories, parses each item file, returns a populated `Board`
- `write_item(item, location) -> ()` — serializes frontmatter + body to `<data_dir>/<location>/<id>.md`
- `move_item(id, from_location, to_location) -> ()` — filesystem rename; sets `completed_at` to now when destination is `done`, clears `completed_at` when source is `done`, refreshes `updated_at` on every move
- `delete_item(id, location) -> ()` — removes the file
- `parse_item(path) -> Item` — reads frontmatter + body, infers status/category from path

Location is a path relative to `data_dir`, e.g. `backlog`, `active/today`, `done`.

## UI Architecture

### Screens

The app has three screens, switched via tabs/sidebar and keyboard shortcuts:

1. **Backlog screen** — single column list of backlog items
2. **Active screen** — four-column kanban: Yesterday, Today, This Week, Next Week
3. **Done screen** — single column list of completed items, sorted by `completed_at` descending

### Root View

```rust
pub struct KanbanView {
    board: Board,
    current_screen: Screen,
    config: Config,
    focus_handle: FocusHandle,
    editing: Option<EditingState>,
    quick_add: Option<QuickAddState>,
    selected_item: Option<Uuid>,
}

pub enum Screen {
    Backlog,
    Active,
    Done,
}
```

### Interactions

**Item creation (quick-add):**
- Triggered by `cmd-n` or a text field at the top of the current screen
- Creates a new item in the current screen's default location: backlog for the backlog screen, `active/today` for the active screen, backlog for the done screen (new items never start in done)
- User types the title (first line), presses enter to create

**Item selection:**
- Mouse click selects an item
- `Tab` advances selection to the next item in the current screen
- `Shift-Tab` moves to the previous item
- Selection state tracked by `selected_item: Option<Uuid>`

**Item movement:**
- `cmd-1` through `cmd-4` move the selected item to yesterday/today/this_week/next_week respectively
- `cmd-b` sends the selected item to backlog
- `cmd-d` toggles the selected item: marks done if active, reopens (moves to `active/today`) if done

**Item editing:**
- Double-click opens an inline editor for the full markdown body
- First line is always the title
- Escape or click-outside commits the edit

**Item deletion:**
- `delete` key removes the selected item's file permanently
- Confirmation prompt before deletion

**Screen switching:**
- `cmd-shift-b` — backlog screen
- `cmd-shift-a` — active screen
- `cmd-shift-d` — done screen
- Also clickable tabs

### Menu Bar (macOS)

- **File**: New Item, Close Window
- **Edit**: Undo, Redo, Cut, Copy, Paste
- **View**: Backlog, Active, Done
- **Item**: Move to Backlog, Move to Today, Mark Done, Reopen, Delete

### Window

Single window, 1000x700 default size, resizable. Title bar shows "Daily Kanban".

### Rendering

Uses GPUI `div()` elements with flexbox layout for columns and lists. Each item rendered as a card (styled `div` with title and truncated body preview). No canvas or custom elements — all standard GPUI `div` + `Styled` API.

The interaction model described above is a first draft. It is expected to need adjustment based on play-testing.

## Testing Strategy

### Unit Tests (in `tests/`, using `tempfile` for filesystem isolation)

- **`item.rs`**: frontmatter parsing/serialization round-trip, title extraction from body (first non-empty line), edge cases (empty body, no frontmatter, body with no title line)
- **`board.rs`**: lifecycle transitions (backlog to active, active to done, done to active, category changes), invalid transitions rejected, board loads from filesystem correctly
- **`storage.rs`**: write item to temp dir, read it back, move item between directories, `completed_at` set on move to done, cleared on reopen, directory structure created on first run

### GPUI Tests (using `gpui::test` macro with `TestAppContext`)

- **`app.rs`**: root view renders without panic, screen switching changes rendered output, quick-add creates an item in the right location, keyboard selection advances through items

### Test Data

`tempfile::TempDir` for each storage/board test so tests are fully isolated. No tests touch the real filesystem.

No integration/E2E layer — the app is simple enough that unit + GPUI view tests cover the surface. Integration tests can be added if specific cross-cutting concerns emerge during play-testing.
