# Kanban Rollover, In-Editor Sub-Items, Typography & Cursor Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement automatic date rollover for Today -> Yesterday, Cmd-Backspace/Delete shortcuts, in-editor sub-item creation (Cmd-K, right-click, prompt), Cmd-Click/Cmd-Enter link following, in-editor breadcrumbs, done status indicator, default '# ' header for new items, configurable monospace font with Menlo default, and mode-dependent blinking cursors.

**Architecture:** Extend `BoardState` with `last_active_date` for automatic rollover upon date change. Enhance `ItemEditor` with an in-editor navigation stack, breadcrumb header, prompt dialog state for sub-item naming, link span resolution, and mode-specific cursor geometry rendering in `EditorElement`. Add font selection to `Config` and the settings UI.

**Tech Stack:** Rust 2024, GPUI (Zed framework), chrono, uuid, serde, serde_json, serde_yaml.

**Spec:** `docs/specs/2026-08-15-kanban-editor-subitems-cursor-improvements-design.md`

## Global Constraints

- Must compile under Rust 2024 edition (`cargo check`, `cargo test`).
- Zero clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic`).
- Maintain existing tests; write new unit and integration tests for each task.
- Follow Clean Architecture and clean code principles.
- Do not make git commits/rebases directly (per CLAUDE.md guidelines).

---

### Task 1: Kanban Date Rollover & Deletion Shortcuts

**Files:**
- Modify: `src/board.rs`
- Modify: `src/storage.rs`
- Modify: `src/app.rs`
- Test: `tests/board.rs`
- Test: `tests/storage.rs`
- Test: `tests/app.rs`

**Interfaces:**
- Produces: `BoardState.last_active_date: Option<chrono::NaiveDate>`
- Produces: `Storage::check_and_apply_rollover(data_dir: &Path, board: &mut Board, state: &mut BoardState) -> std::io::Result<()>`
- Produces: Keybindings `cmd-backspace` and `cmd-delete` mapped to `DeleteItem` in `KanbanView::key_bindings()`.

- [ ] **Step 1: Write failing tests for date rollover and delete shortcuts**

Add tests in `tests/storage.rs` and `tests/board.rs`:
```rust
#[test]
fn test_board_state_rollover_today_to_yesterday() {
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path();
    Storage::init(data_dir).unwrap();

    let today_item = Item::new("Task for today");
    let yesterday_item = Item::new("Existing yesterday task");
    Storage::write_item(data_dir, &today_item, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(data_dir, &yesterday_item, &Location::Active(Category::Yesterday)).unwrap();

    let mut state = BoardState {
        version: 1,
        order: std::collections::HashMap::new(),
        last_active_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 14).unwrap()),
    };
    Storage::save_board_state_with_date(data_dir, &Board::default(), state.last_active_date).unwrap();

    let mut board = Storage::load_board(data_dir).unwrap();
    assert_eq!(board.active.today.len(), 0);
    assert_eq!(board.active.yesterday.len(), 2);
    assert!(board.active.yesterday.iter().any(|i| i.id == today_item.id));
    assert!(board.active.yesterday.iter().any(|i| i.id == yesterday_item.id));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test storage test_board_state_rollover_today_to_yesterday`
Expected: FAIL (field `last_active_date` or rollover logic not implemented)

- [ ] **Step 3: Implement BoardState date tracking, rollover logic, and keybindings**

1. In `src/board.rs`:
   Update `BoardState`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
   pub struct BoardState {
       pub version: u32,
       pub order: HashMap<String, Vec<Uuid>>,
       #[serde(default)]
       pub last_active_date: Option<chrono::NaiveDate>,
   }
   ```
2. In `src/storage.rs`:
   In `Storage::load_board(data_dir)`:
   Check `state.last_active_date`. Compare with `chrono::Local::now().date_naive()`. If `last_date < current_date`, move files from `active/today` to `active/yesterday`, update board lists by accumulating into `active.yesterday`, clear `active.today`, set `state.last_active_date = Some(current_date)`, and save `board_state.json`.
3. In `src/app.rs`:
   Add keybindings in `KanbanView::key_bindings()`:
   ```rust
   KeyBinding::new("cmd-backspace", DeleteItem, Some("KanbanView")),
   KeyBinding::new("cmd-delete", DeleteItem, Some("KanbanView")),
   ```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test storage && cargo test --test board && cargo test --test app`
Expected: PASS

---

### Task 2: In-Editor Link Resolution & Formatting Helpers

**Files:**
- Modify: `src/link.rs`
- Test: `tests/link.rs`

**Interfaces:**
- Produces: `pub struct LinkSpan { pub range: std::ops::Range<usize>, pub target_id: Uuid, pub text: String }`
- Produces: `pub fn find_link_at_offset(content: &str, offset: usize) -> Option<LinkSpan>`
- Produces: `pub fn format_markdown_link(text: &str, id: Uuid) -> String`

- [ ] **Step 1: Write failing tests for link span detection**

In `tests/link.rs`:
```rust
#[test]
fn test_find_link_at_offset_markdown() {
    let id = Uuid::new_v4();
    let body = format!("Check this item: [My Sub Task]({}.md) for details", id);
    let offset = 22; // within "[My Sub Task]"
    let span = crate::link::find_link_at_offset(&body, offset).expect("should find link");
    assert_eq!(span.target_id, id);
    assert_eq!(span.text, "My Sub Task");

    // Offset outside link
    assert!(crate::link::find_link_at_offset(&body, 2).is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test link test_find_link_at_offset_markdown`
Expected: FAIL

- [ ] **Step 3: Implement `find_link_at_offset` and `format_markdown_link`**

In `src/link.rs`:
Implement regular expression / slice scanner that identifies all markdown links `[text](uuid.md)` and wikilinks `[[uuid.md]]` with their exact character/byte ranges `start..end`. Return `Some(LinkSpan)` if `offset >= start && offset <= end`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test link`
Expected: PASS

---

### Task 3: Monospace Font Selection & Editor/Gutter Typography

**Files:**
- Modify: `src/config.rs`
- Modify: `src/editor.rs`
- Modify: `src/app.rs`
- Test: `tests/config.rs`
- Test: `tests/app.rs`

**Interfaces:**
- Produces: `Config.font_family: String` (defaults to `"Menlo"`)
- Produces: `Config::MONOSPACE_FONTS: &[&'static str]` = `["Menlo", "SF Mono", "Monaco", "Courier New", "Courier", "Consolas", "Fira Code", "JetBrains Mono"]`
- Produces: Gutter line numbers and `EditorElement` styled with `font_family`.

- [ ] **Step 1: Write failing tests for font configuration**

In `tests/config.rs`:
```rust
#[test]
fn test_config_font_family_default_and_serialization() {
    let config = Config::default();
    assert_eq!(config.font_family, "Menlo");
    let serialized = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.font_family, "Menlo");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config test_config_font_family_default_and_serialization`
Expected: FAIL

- [ ] **Step 3: Implement `font_family` in `Config`, Settings UI, and `ItemEditor`**

1. In `src/config.rs`:
   Add `pub font_family: String` to `Config`, default `"Menlo".to_string()`.
   Add `pub const MONOSPACE_FONTS: &[&'static str] = &["Menlo", "SF Mono", "Monaco", "Courier New", "Courier", "Consolas", "Fira Code", "JetBrains Mono"];`
2. In `src/editor.rs`:
   In `ItemEditor::render` and `EditorElement::prepaint`/`paint`:
   Apply `style.font_family(self.config.font_family.clone())` to text run and gutter line numbers. Set identical font size (`px(13.)`) and line height for gutter and editor text.
3. In `src/app.rs`:
   In `render_settings_screen`: Add a dropdown selector for Font Family with trigger and option list.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test config && cargo test --test app`
Expected: PASS

---

### Task 4: New Item Header Default & Blinking Mode-Specific Cursor

**Files:**
- Modify: `src/editor.rs`
- Modify: `src/app.rs`
- Test: `tests/app.rs`

**Interfaces:**
- Produces: When `is_new: true`, initial editor content is `"# "` with cursor offset at 2.
- Produces: Normal/Command mode paints solid block cursor; Insert mode paints 2px vertical line cursor; cursor blinks when focused.

- [ ] **Step 1: Write failing test for new item initial header**

In `tests/app.rs`:
```rust
#[gpui::test]
async fn test_new_item_opens_with_header_prefix(cx: &mut gpui::TestAppContext) {
    let mut view = cx.new_window(|cx| KanbanView::new(cx));
    cx.dispatch_action(&view, NewItem);
    // Verify editor content starts with "# " and cursor is at 2
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test app test_new_item_opens_with_header_prefix`
Expected: FAIL

- [ ] **Step 3: Implement initial header and mode-specific cursor rendering**

1. In `src/app.rs`:
   When creating editor for `NewItem`, initialize content with `"# "` and cursor at 2.
2. In `src/editor.rs`:
   In `EditorElement::prepaint`:
   Calculate cursor quad based on `editor.vi_state.mode`:
   - For `ViMode::Normal` / `ViMode::Command` / `ViMode::Visual`: Solid block cursor with width equal to character glyph width (or font advance) and height equal to `line_height`.
   - For `ViMode::Insert` (or non-Vi mode): Line cursor with width `px(2.)` and height equal to `line_height`.
   - Add blinking opacity calculation / ensure visibility on focus.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test app`
Expected: PASS

---

### Task 5: In-Editor Sub-Item Creation & Prompt Dialog (`Cmd-K` and Right-Click)

**Files:**
- Modify: `src/editor.rs`
- Modify: `src/app.rs`
- Test: `tests/app.rs`

**Interfaces:**
- Produces: `EditorCreateSubItem` action bound to `cmd-k`.
- Produces: `ItemEditor.subitem_prompt_open: bool` and `prompt_input: String`.
- Produces: Sub-item creation replaces selection with `[Selection](<uuid>.md)` or inserts prompt link at cursor.

- [ ] **Step 1: Write failing tests for in-editor sub-item creation**

In `tests/app.rs`:
```rust
#[gpui::test]
async fn test_editor_create_subitem_with_selection(cx: &mut gpui::TestAppContext) {
    // Select text "Sub task 1", dispatch EditorCreateSubItem, check inserted markdown link and created file
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test app test_editor_create_subitem_with_selection`
Expected: FAIL

- [ ] **Step 3: Implement sub-item creation with selection and prompt dialog**

1. In `src/editor.rs`:
   - Add `EditorCreateSubItem` to `dkb_editor` actions.
   - Bind `cmd-k` to `EditorCreateSubItem`.
   - Implement `on_create_sub_item`:
     - If `state.selected_range()` is not empty:
       Create new `Item` with title = selected text, save to `Location::Active(Category::Today)`, replace selection with `[selected_text](<id>.md)`, auto-save parent.
     - If selection is empty:
       Set `subitem_prompt_open = true`, focus prompt input.
   - Add Prompt Dialog overlay rendering in `ItemEditor::render` (textbox + Create/Cancel buttons, Enter to submit).
   - Add right-click context menu in editor: "Create Sub-item from Selection" / "Create Sub-item...".

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test app`
Expected: PASS

---

### Task 6: In-Editor Link Navigation, Breadcrumbs & Done Badge

**Files:**
- Modify: `src/editor.rs`
- Modify: `src/app.rs`
- Test: `tests/app.rs`

**Interfaces:**
- Produces: `ItemEditor.history_stack: Vec<Uuid>`
- Produces: `EditorFollowLink` bound to `cmd-enter`
- Produces: `EditorNavigateBack` bound to `cmd-left`, `cmd-[`
- Produces: Cmd-Click link navigation in `EditorElement`
- Produces: Clickable breadcrumb header and `✅ Done` badge in `ItemEditor`

- [ ] **Step 1: Write failing tests for link following and breadcrumbs**

In `tests/app.rs`:
```rust
#[gpui::test]
async fn test_editor_follow_link_and_navigate_back(cx: &mut gpui::TestAppContext) {
    // Create parent with link to child, place cursor on link, dispatch EditorFollowLink
    // Verify editor is now displaying child, history_stack has parent
    // Dispatch EditorNavigateBack, verify editor displays parent again
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test app test_editor_follow_link_and_navigate_back`
Expected: FAIL

- [ ] **Step 3: Implement link following, history stack, breadcrumbs, and Done badge**

1. In `src/editor.rs`:
   - Add `history_stack: Vec<Uuid>` to `ItemEditor`.
   - Implement `follow_link_at_offset`:
     - Find link span at offset.
     - If found: save current content, push current `editing_item_id` to `history_stack`, load target item body and ID into editor.
   - Implement `on_follow_link` (`EditorFollowLink` on `cmd-enter`).
   - In `EditorElement::paint`: handle mouse down with `modifiers.command` (Cmd-click) over a link span.
   - Implement `on_navigate_back` (`EditorNavigateBack` on `cmd-left` / `cmd-[`) and breadcrumb click listeners.
   - In `ItemEditor::render`:
     - Render top header with breadcrumb trail: clickable items for each history element + current title.
     - If current item is in `Location::Done` or `completed_at.is_some()`, render `✅ Done` badge.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test app`
Expected: PASS

---

### Task 7: Full Verification, Clippy Pedantic & Regression Testing

**Files:**
- Modify/Verify: All project files and tests

- [ ] **Step 1: Run complete test suite**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 2: Run clippy pedantic on workspace and all targets**

Run: `cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic`
Expected: Clean build with 0 warnings.

- [ ] **Step 3: Build debug and release binaries**

Run: `cargo build && cargo build --release`
Expected: SUCCESS

---
