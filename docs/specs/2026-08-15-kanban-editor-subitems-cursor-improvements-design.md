# Daily Kanban (dkb) — Sub-Items, Editor Typography, Cursor & Date Rollover Design Spec

## 1. Overview & Scope

This specification addresses all findings and improvements listed in `docs/2026-08-15-findings.md`, covering:
1. **Kanban Rollover & Deletion Shortcuts**:
   - Automatic rollover of items from the 'Today' column into 'Yesterday' on day transition (accumulating with existing yesterday items).
   - Additional deletion shortcuts: `Cmd-Backspace` and `Cmd-Delete` in Kanban view.
2. **In-Editor Sub-Item Creation & Navigation**:
   - Starting a sub-item from within the editor via:
     - Right-click context menu on a highlighted word / phrase.
     - Right-click context menu with no selection (prompts for the word/short phrase).
     - Keyboard shortcut `Cmd-K` when text is selected (turns selection into `[Selection](uuid.md)`).
     - Keyboard shortcut `Cmd-K` when no text is selected (prompts for the word/short phrase, then inserts link at cursor).
   - Following a link from the editor:
     - `Cmd-Click` on a link (`[text](uuid.md)` or `[[uuid.md]]`).
     - Keyboard shortcut `Cmd-Enter` when cursor is on or inside a link.
   - Breadcrumb navigation: Clickable breadcrumbs at the top of the editor and keyboard shortcut `Cmd-Left` / `Cmd-[` to return to parent item (auto-saving current buffer).
   - Done indicator: Green checkmark badge (`✅ Done`) displayed in the editor top bar when the active item is completed.
3. **Editor Typography & Cursor Enhancements**:
   - First line of new items automatically begins with `"# "` and cursor placed immediately after for instant title entry.
   - Fixed-width font in the editor (Menlo by default, with a selectable monospace font family dropdown in Settings).
   - Gutter line numbers rendered with the exact same monospace font, font size, and metrics as the editor text.
   - Mode-dependent cursor rendering:
     - Vi Command / Normal mode: solid, blinking box cursor.
     - Edit / Insert mode: blinking vertical line cursor.
     - Cursor is always visible and blinks smoothly when editor is focused.

---

## 2. Architecture & Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                         KanbanView                          │
│                                                             │
│  - Checks local date vs BoardState.last_active_date         │
│  - Rollover Today -> Yesterday on startup / date change     │
│  - Handles Cmd-Backspace / Cmd-Delete to DeleteItem         │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                      ItemEditor                       │  │
│  │                                                       │  │
│  │  - Breadcrumb Bar (Parent > Child) & ✅ Done badge     │  │
│  │  - Context Menu: Create Sub-Item                      │  │
│  │  - Sub-Item Prompt Dialog (for no-selection Cmd-K)    │  │
│  │  - Monospace EditorElement (Menlo / Config Font)      │  │
│  │  - Mode Cursor: Normal/Command = Box, Insert = Line   │  │
│  │  - Navigation Stack: Cmd-Click, Cmd-Enter, Cmd-[      │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Subsystem Specifications

### 3.1 Kanban Date Rollover & Deletion Keybindings

#### Date Tracking & Rollover Logic (`src/board.rs`, `src/storage.rs`, `src/app.rs`)
- `BoardState` will be extended with:
  ```rust
  pub struct BoardState {
      pub version: u32,
      pub order: HashMap<String, Vec<Uuid>>,
      #[serde(default)]
      pub last_active_date: Option<chrono::NaiveDate>,
  }
  ```
- When `Storage::load_board(data_dir)` runs (or when `KanbanView::new` initializes):
  1. Retrieve `today_date = chrono::Local::now().date_naive()`.
  2. If `state.last_active_date` is `Some(last_date)` and `last_date < today_date`:
     - Move each item from `Location::Active(Category::Today)` to `Location::Active(Category::Yesterday)` using `Storage::move_item`.
     - In-memory `Board.active.yesterday` accumulates the rolled over items appended to any existing yesterday items.
     - `Board.active.today` is cleared.
     - `state.last_active_date = Some(today_date)`.
     - Save updated `board_state.json`.
  3. If `state.last_active_date` is `None`: initialize it to `Some(today_date)` and save.

#### Deletion Keybindings
In `KanbanView::key_bindings()`:
- Bind `cmd-backspace` to `DeleteItem`.
- Bind `cmd-delete` to `DeleteItem`.
- Retain existing `delete` and `backspace`.

---

### 3.2 In-Editor Sub-Item Creation, Link Navigation & Breadcrumbs

#### Actions (`src/editor.rs`, `src/app.rs`)
Define editor actions:
- `EditorCreateSubItem`: Bound to `cmd-k`.
- `EditorFollowLink`: Bound to `cmd-enter`.
- `EditorNavigateBack`: Bound to `cmd-left`, `cmd-[`.

#### Sub-Item Creation Flow
1. **With Selection**:
   - Extract selected text from `TextInputState::selected_range()`.
   - Create new `Item` with title = selected text, saved to `Location::Active(Category::Today)`.
   - Replace selected range in parent document with `[Selected Text](<new_item_id>.md)`.
   - Auto-save parent document to storage.
2. **Without Selection / Prompt Modal**:
   - `ItemEditor` displays an inline prompt input modal ("Create Sub-item: Enter title/phrase").
   - When confirmed (Enter / Submit):
     - Create new `Item` with entered title.
     - Insert `[Title](<new_item_id>.md)` at the cursor position.
     - Auto-save parent document.
     - Dismiss prompt dialog.
3. **Right-Click Context Menu in Editor**:
   - Right-click detects cursor/mouse position:
     - If text is selected (or cursor on a word): context menu offers "Create Sub-item from Selection".
     - If no text selected: context menu offers "Create Sub-item...".
   - Triggers the corresponding sub-item creation flow.

#### Link Detection & Following
- Add helper in `src/link.rs`:
  ```rust
  pub struct LinkSpan {
      pub range: Range<usize>,
      pub target_id: Uuid,
      pub text: String,
  }
  pub fn find_link_at_offset(content: &str, offset: usize) -> Option<LinkSpan>;
  ```
- **Cmd-Click Handling**:
  - In `EditorElement::paint` / event listener, intercept mouse down with `modifiers.command` or `cmd`.
  - Convert mouse click position to character offset via shaped text glyph layout.
  - If a link span is at that offset:
    - Save current item content.
    - Push current item ID onto `history_stack: Vec<Uuid>`.
    - Load target item into editor.
- **Cmd-Enter Handling**:
  - `EditorFollowLink` action: Checks `find_link_at_offset(content, cursor_offset)`.
  - If found: saves parent, pushes to `history_stack`, loads target item.

#### In-Editor Breadcrumb & History Navigation
- `ItemEditor` stores `history_stack: Vec<Uuid>`.
- Top bar of `ItemEditor` renders breadcrumbs:
  - Each segment in `history_stack` + current item title.
  - Clicking any ancestor segment saves current editor state and jumps back to that item, popping deeper stack items.
- `Cmd-Left` / `Cmd-[` (`EditorNavigateBack`):
  - If `history_stack` is not empty, pops the last item, saves current state, and opens the popped item in editor.

#### Done Status Badge
- `ItemEditor` inspects whether the loaded item is currently in `Location::Done` or has `completed_at.is_some()`.
- If done, render `✅ Done` badge (green background/border with white text) in the editor top header bar.

---

### 3.3 Editor Typography, Cursor & Header Initialization

#### New Item Header
- When `ItemEditor::new` is invoked for a new item (`is_new: true`), the initial content is set to `"# "`.
- Cursor starts at position 2 (after `# `).

#### Monospace Font Selection & Line Numbers Alignment
- Update `Config` in `src/config.rs`:
  ```rust
  pub struct Config {
      ...
      pub font_family: String, // default: "Menlo".into()
  }
  ```
- Monospace font options: `["Menlo", "SF Mono", "Monaco", "Courier New", "Courier", "Consolas", "Fira Code", "JetBrains Mono"]`.
- In `Settings` screen, add a "Font Family" dropdown picker allowing the user to select their preferred monospace font.
- In `ItemEditor`:
  - `window.text_style()` uses `font_family: self.config.font_family.clone()`.
  - Line numbers column in gutter uses the identical font family, font size (`px(13.)`), line height, and styling as the editor body text, ensuring pixel-perfect vertical alignment.

#### Blinking Mode-Specific Cursor Rendering
- Cursor shape:
  - **Normal / Command mode**: Solid block cursor `Bounds::new(pos, size(char_width, line_height))` painted with selection/accent color.
  - **Insert / Edit mode**: Vertical line cursor `Bounds::new(pos, size(px(2.), line_height))` painted with accent color.
- Cursor blinking:
  - Blinking cycle tracked via GPUI animation or focus state.
  - Solid on initial focus / keypress, smoothly pulsing/blinking every 500ms.
  - Cursor is guaranteed visible with high-contrast RGBA fill.

---

## 4. Testing Strategy

1. **Kanban Rollover Unit & Integration Tests (`tests/storage.rs`, `tests/board.rs`)**:
   - Test date rollover: mock `last_active_date` as yesterday; verify items in `active/today` move to `active/yesterday` while keeping existing items in `yesterday`.
   - Test same-day loading does not trigger rollover.
   - Test `cmd-backspace` and `cmd-delete` actions delete selected item.
2. **In-Editor Link Detection & Creation Tests (`tests/link.rs`, `tests/editor.rs`)**:
   - Unit tests for `find_link_at_offset` for markdown links and wikilinks.
   - Test sub-item creation replaces text with `[Text](id.md)`.
   - Test prompt dialog creation inserts link at cursor.
   - Test navigation stack push/pop and breadcrumb navigation.
3. **Editor Styling & Cursor Tests (`tests/config.rs`, `tests/app.rs`)**:
   - Test config serialization with `font_family`.
   - Test new item initializes with `# ` header and cursor position.
   - Test Vi mode cursor variant dimensions (box in normal vs line in insert).
   - Test Done indicator rendering when item status is Done.

---

## 5. Verification Commands
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic`
- `cargo build`
