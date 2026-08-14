# Daily Kanban (dkb) — Comprehensive Improvements Design Spec

## 1. Overview & Scope

This specification addresses all findings and feature requirements identified in `docs/2026-08-14-findings.md`, modernizing `dkb` into a full-featured, idiomatic macOS GPUI application.

### Key Capabilities
1. **Configuration & Theming**: Persistent settings tab (pinned far-right), `Cmd-,` shortcut, menu item in app menu, Light/Dark/System theme engine, and configurable storage directory.
2. **Editor Enhancements**: Configurable line numbers gutter and Vi-mode modal editing (Normal/Insert/Visual modes).
3. **Board UX & Tabs**: Tab bar layout (Backlog, Active, Done, Settings), absolute item ordering per column persisted in workspace board state, and click-and-drag reordering/cross-column moving.
4. **Enhanced Navigation**: `Cmd-]` and `Cmd-[` column stepping (with top-item autofocus), 2D spatial Arrow key navigation, Vi navigation (`h`/`j`/`k`/`l`), and Enter/Return to edit.
5. **Sub-Items & iwe Integration**: Standard markdown link parsing and generation for sub-items, recursive item count badge (`↪ <count>`), sub-item creation in editor and card, drill-down breadcrumbs navigation, and `.iwe` workspace initialization.
6. **Title Cleaning**: Markdown formatting stripped from first line for item title display.
7. **macOS Packaging & App Store Readiness**: App bundle creation (`dkb.app`), custom app icon (`.icns`), `Info.plist`, entitlements for sandboxing, and distribution scripts for the Mac App Store.

---

## 2. Architecture & Module Structure

```
dkb/
├── Cargo.toml
├── build.rs                    # Icon / asset embedding helper
├── assets/
│   ├── AppIcon.icns
│   └── AppIcon.png
├── resources/
│   ├── Info.plist
│   └── dkb.entitlements
├── scripts/
│   ├── bundle_macos.sh         # Creates dkb.app with icon and Info.plist
│   └── package_appstore.sh     # Signs and packages for Mac App Store
├── src/
│   ├── lib.rs
│   ├── bin/
│   │   └── dkb.rs              # App entrypoint, menu bar wiring
│   ├── app.rs                  # KanbanView, top tab bar, drag state, keybindings
│   ├── board.rs                # In-memory board state, column transitions
│   ├── config.rs               # Config model, TOML persistence, UI settings view
│   ├── theme.rs                # Light / Dark / System color palette provider
│   ├── item.rs                 # Item data model, title unformatting, frontmatter
│   ├── link.rs                 # Markdown link parser, recursive sub-item graph
│   ├── iwe.rs                  # iwe workspace integration & config init
│   ├── storage.rs              # File I/O, item files, board order state persistence
│   ├── text_input.rs           # Low-level text buffer, cursor/selection tracking
│   ├── vi.rs                   # Vi mode state machine (Normal/Insert/Visual)
│   └── editor.rs               # ItemEditor view, line numbers gutter, vi status bar
└── tests/
    ├── config.rs
    ├── item.rs
    ├── board.rs
    ├── storage.rs
    ├── link.rs
    ├── vi.rs
    └── text_input.rs
```

---

## 3. Detailed Subsystem Specifications

### 3.1 Configuration & Theming Subsystem (`config.rs`, `theme.rs`)

#### Data Model
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub vi_mode: bool,
    pub line_numbers: bool,
    pub theme_mode: ThemeMode,
}
```

#### Default Values
- `data_dir`: `~/Library/Application Support/dkb`
- `vi_mode`: `false`
- `line_numbers`: `false`
- `theme_mode`: `ThemeMode::System`

#### Storage Format (`config.toml`)
```toml
data_dir = "~/Library/Application Support/dkb"
vi_mode = false
line_numbers = false
theme_mode = "system" # "light" | "dark" | "system"
```

#### Settings Screen & Menu
- Accessible via:
  - Top tab bar: "Settings" tab pinned to the far right.
  - Global shortcut: `Cmd-,`.
  - macOS App Menu: `dkb -> Settings...` (with `Cmd-,`).
- **Settings View Controls**:
  - Storage directory text input with "Browse..." button (or folder validation) and "Reset to Default" button.
  - Vi Mode toggle checkbox / switch.
  - Line Numbers toggle checkbox / switch.
  - Theme Mode radio/segmented selector: `[Light | Dark | Follow System]`.

#### Theme Palette (`theme.rs`)
`Theme` provides colors resolved dynamically based on `ThemeMode` and window appearance:
- `bg_window`: Window background (e.g. Light: `#F5F5F7`, Dark: `#1E1E1E`).
- `bg_surface`: Card and editor background (Light: `#FFFFFF`, Dark: `#252526`).
- `bg_column`: Kanban column background (Light: `#EAEAEA`, Dark: `#2D2D2D`).
- `bg_header_tab`: Tab bar background (Light: `#E0E0E0`, Dark: `#181818`).
- `text_primary`: Primary body text (Light: `#1C1C1E`, Dark: `#E0E0E0`).
- `text_secondary`: Muted labels and counts (Light: `#6E6E73`, Dark: `#8E8E93`).
- `border`: Card and divider borders (Light: `#D1D1D6`, Dark: `#383838`).
- `selection`: Active card border and selection fill (Light: `#007AFF`, Dark: `#0A84FF`).
- `accent`: Interactive buttons and badges.

---

### 3.2 Kanban UI, Tabs & Absolute Ordering (`app.rs`, `board.rs`, `storage.rs`)

#### Top Bar Tabs
The tab bar header is rendered with distinct tab elements:
- Left cluster: `Backlog`, `Active`, `Done`.
- Right cluster (pushed with `flex_1` spacer): `Settings` tab (`⚙ Settings ⌘,`).
- Active tab styling: high contrast background, border, bold text, indicator underline or pill background.

#### Absolute Item Sorting (`board.rs`, `storage.rs`)
- Each column maintains an explicit ordered list of item UUIDs.
- Stored in `<data_dir>/board_state.json`:
```json
{
  "version": 1,
  "order": {
    "backlog": ["uuid-1", "uuid-2"],
    "active_yesterday": ["uuid-3"],
    "active_today": ["uuid-4", "uuid-5"],
    "active_this_week": [],
    "active_next_week": [],
    "done": ["uuid-6"]
  }
}
```
- When `load_board` runs:
  1. Reads all files in column directories.
  2. Applies the stored ordering in `board_state.json`. Any new items not yet in `board_state.json` are placed at the beginning/end cleanly.
  3. Preserves this exact order across restarts.
- When an item is moved or reordered, `board_state.json` is automatically updated and saved atomically.

#### Drag-and-Drop
- Cards are interactive drag sources and column containers/cards are drop targets.
- GPUI drag-and-drop state:
  - Dragging a card shows a floating preview and placeholder line at the target position.
  - Dropping onto a column moves the item to that column and places it at the specific dropped index.
  - Dropping within the same column reorders the column's absolute sort.

---

### 3.3 Keyboard Navigation & Vi Mode (`app.rs`, `vi.rs`)

#### Column & Item Navigation
1. **Column Stepping**:
   - `Cmd-]` (Next Column): Steps right through kanban columns (`Yesterday` -> `Today` -> `This Week` -> `Next Week`). Automatically focuses the top item of the destination column.
   - `Cmd-[` (Previous Column): Steps left through kanban columns. Automatically focuses the top item of the destination column.
2. **2D Spatial Arrow Key Navigation**:
   - `Down Arrow` / `Up Arrow`: Moves selection to the next / previous item within the current column.
   - `Right Arrow` / `Left Arrow`: Finds the visually closest item (closest vertical center Y-coordinate) in the adjacent column to the right / left. If the adjacent column is empty, moves to the column header.
3. **Vi Navigation in Kanban View** (when `vi_mode` is enabled):
   - `j`: Down.
   - `k`: Up.
   - `h`: Left (closest item in left column).
   - `l`: Right (closest item in right column).
4. **Open for Editing**:
   - Pressing `Enter` / `Return` when an item is selected immediately opens the editor.

---

### 3.4 Markdown Editor Enhancements (`editor.rs`, `vi.rs`, `text_input.rs`)

#### Line Numbers Gutter
- When `config.line_numbers` is `true`, a left gutter renders line numbers `1, 2, 3...` aligned with the text rows.
- Gutter background is distinct (`bg_column`), with muted line number text and proper right alignment and padding.

#### Vi Mode Modal State Machine (`vi.rs`)
When `vi_mode` is enabled, the editor operates in modal states:
- **Normal Mode**:
  - `h`, `j`, `k`, `l` cursor movement.
  - `w`, `b`, `e` word motions.
  - `0`, `$` line start/end.
  - `i`: Enter Insert mode at cursor.
  - `a`: Enter Insert mode after cursor.
  - `o`: Open new line below and enter Insert mode.
  - `O`: Open new line above and enter Insert mode.
  - `x`: Delete character under cursor.
  - `dd`: Delete line.
  - `yy`: Yank (copy) line.
  - `p`: Paste yanked text after cursor.
  - `u`: Undo.
  - `Ctrl-r`: Redo.
  - `v`: Enter Visual mode.
  - `:`: Command mode (e.g. `:w` save, `:q` close).
- **Insert Mode**:
  - Regular typing and text entry.
  - `Escape`: Return to Normal mode.
- **Visual Mode**:
  - Selection tracking using motion keys.
  - `y`: Yank selection.
  - `d` / `x`: Delete selection.
  - `Escape`: Return to Normal mode.
- **Editor Status Bar**:
  - A subtle footer status bar indicates `-- NORMAL --`, `-- INSERT --`, or `-- VISUAL --`.

---

### 3.5 Title Formatting & Sub-Items with iwe Integration (`item.rs`, `link.rs`, `iwe.rs`)

#### Title Formatting (`item.rs`)
The item title displayed on kanban cards strips markdown formatting from the first non-empty line:
- `# Heading 1` -> `Heading 1`
- `## Heading 2` -> `Heading 2`
- `**Bold Title**` -> `Bold Title`
- `*Italic*` -> `Italic`
- `[Link Text](url)` -> `Link Text`
- `` `Code Title` `` -> `Code Title`
- `~~Strikethrough~~` -> `Strikethrough`

#### Sub-Items via Links (`link.rs`)
- Sub-items are linked markdown files referenced within a parent item's markdown body.
- Supported link formats:
  - Standard markdown link: `[Child Item Title](<uuid>.md)` or `[Child Item Title](active/today/<uuid>.md)`
  - Wikilink: `[[<uuid>]]` or `[[<uuid>|Child Item Title]]`
- **Recursive Sub-Item Count**:
  - `link::count_recursive_subitems(root_id, &data_dir) -> usize` traverses the link graph without circular loop traps (using a `HashSet<Uuid>` visited set).
  - Total recursive count is displayed on the card: `↪ <count>` (e.g. `↪5`).
- **Sub-Item Visibility & Drill-Down**:
  - Sub-items linked from parent items are treated as sub-tasks and excluded from top-level root kanban views.
  - On the parent card:
    - Right badge shows `↪ <count>` with hover tooltip "View sub-items".
    - Clicking the badge or pressing `Cmd-Right` drills down into a scoped sub-board for the item, displaying breadcrumbs at the top: `Active > Root Item > Sub Item`.
    - "+" button on card or `Cmd-Shift-N` creates a new sub-item, creates its file in storage, and appends `\n- [New Item Title](<new-uuid>.md)` to the parent item body.

#### iwe Integration (`iwe.rs`)
- On storage init, `dkb` initializes an `.iwe` workspace in `data_dir` if not present:
  - Writes `.iwe/config.toml` configuring document schemas and link resolution settings.
  - If `iwe` CLI is present on the system (`/opt/homebrew/bin/iwe` or in `$PATH`), `dkb` registers workspace paths for seamless CLI/LSP interop.
  - Follows iwe link conventions (relative paths, markdown links, inclusion links `![[...]]`).

---

### 3.6 macOS Packaging, Icon & Mac App Store Readiness

#### App Icon (`assets/AppIcon.icns`)
- Create an application icon asset containing standard resolutions (16x16, 32x32, 64x64, 128x128, 256x256, 512x512, 1024x1024).
- Represents Daily Kanban with modern macOS Big Sur+ squircle aesthetic.

#### App Bundle Structure (`scripts/bundle_macos.sh`)
```
Daily Kanban.app/
└── Contents/
    ├── Info.plist
    ├── PkgInfo
    ├── MacOS/
    │   └── dkb (compiled binary)
    └── Resources/
        └── AppIcon.icns
```

#### Info.plist (`resources/Info.plist`)
- `CFBundleIdentifier`: `com.doug.dkb`
- `CFBundleName`: `Daily Kanban`
- `CFBundleDisplayName`: `Daily Kanban`
- `CFBundleIconFile`: `AppIcon`
- `CFBundleShortVersionString`: `0.1.0`
- `NSHighResolutionCapable`: `true`
- `LSMinimumSystemVersion`: `13.0`

#### Mac App Store Entitlements (`resources/dkb.entitlements`)
- `com.apple.security.app-sandbox`: `true`
- `com.apple.security.files.user-selected.read-write`: `true` (for custom storage directories)
- `com.apple.security.files.bookmarks.app-scope`: `true`

---

## 4. Testing Strategy

1. **Unit Tests**:
   - `tests/config.rs`: Serialization, deserialization, default path expansion, theme modes.
   - `tests/item.rs`: Title unformatting (stripping headers, bold, italics, links, inline code).
   - `tests/board.rs`: Column ordering, insertion, movement, persistence verification.
   - `tests/link.rs`: Markdown link extraction, wikilink extraction, recursive sub-item counting, cyclic link protection.
   - `tests/vi.rs`: Vi state machine transitions (Normal -> Insert, Normal -> Visual, motions, deletions, yanks).
2. **GPUI Tests**:
   - Tab switching between Backlog, Active, Done, and Settings.
   - Shortcut actions (`Cmd-,`, `Cmd-]`, `Cmd-[`, arrow navigation).
   - Editor line numbering rendering and modal text input.

---

## 5. Verification Commands
- `cargo test`
- `cargo build --release`
- `./scripts/bundle_macos.sh` -> Produces `target/release/bundle/Daily Kanban.app`
- Launch and verify native GUI rendering, settings persistence, keyboard shortcuts, drag-and-drop, and sub-item counts.

---
*End of Design Specification*
