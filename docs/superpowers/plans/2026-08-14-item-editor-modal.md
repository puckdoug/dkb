# Item Editor Modal — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the broken quick-add placeholder with a proper multi-line markdown item editor that opens as a modal overlay, receives keyboard focus immediately, and supports a tear-off to a separate window.

**Architecture:** Port `TextInputState` from the `st` project as a pure state struct (no GPUI dependency) to `src/text_input.rs`. Build an `ItemEditor` GPUI view that wraps `TextInputState`, implements `EntityInputHandler`, and renders as a modal overlay. The modal lives inside `KanbanView::render` — when `editing_item` is `Some`, a semi-transparent overlay covers the main window with a centered editor panel. A tear-off button opens the same editor in a new GPUI window and dismisses the modal. The `TextInputState` struct is kept separate from rendering so vi-mode can be added later by swapping/wrapping the state layer.

**Tech Stack:** Rust 2024, GPUI (git dep from zed-industries/zed), unicode-segmentation for grapheme/word boundaries, serde + chrono for item persistence.

**Spec:** Design approved in brainstorming session. Key decisions:
- Modal overlay (not separate window by default), with tear-off to separate window
- Multi-line markdown editing (not single-line title entry)
- Editor receives keyboard focus immediately on open
- Tear-off uses expand icon (box with arrow out top-right), tooltip "tear off window"
- `TextInputState` is a separate pure-state struct to support future vi-mode

---

## Global Constraints

- Rust edition 2024
- GPUI and gpui_platform are git dependencies from `https://github.com/zed-industries/zed` — NOT the crates.io fork
- gpui_platform must have the `font-kit` feature enabled
- TDD: every task writes failing tests first, verifies they fail, then implements
- `TextInputState` must have NO GPUI imports — it is a pure state struct testable without GPUI
- The `EntityInputHandler` impl on `ItemEditor` delegates to `TextInputState` — it does not contain editing logic itself
- All tests use `tempfile::TempDir` for filesystem isolation where storage is involved
- Commit after each task
- GPUI is pre-1.0 — some APIs referenced in the plan (`on_double_click`, `tooltip`, `shadow_lg`, `overflow_y_scroll`) may not exist in this version. If a method doesn't compile, check the GPUI source or the `st` project for the equivalent working API, or omit the styling if no equivalent exists. The reference project is at `/Users/doug/Development/rust/st/`.

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Add `unicode-segmentation` dependency |
| `src/lib.rs` | Add `pub mod text_input;` |
| `src/text_input.rs` | `TextInputState` pure state struct — ported from `st`, adapted for multi-line |
| `src/app.rs` | `ItemEditor` view, modal overlay rendering, tear-off, wiring into `KanbanView` |
| `tests/text_input.rs` | Unit tests for `TextInputState` (ported from `st`'s test suite) |

---

### Task 1: Add unicode-segmentation dependency and text_input module

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/text_input.rs` (stub)
- Create: `tests/text_input.rs` (stub)

**Interfaces:**
- Produces: `pub mod text_input;` in lib.rs, empty `TextInputState` struct

- [ ] **Step 1: Add unicode-segmentation to Cargo.toml**

Add to `[dependencies]` section:

```toml
unicode-segmentation = "1.12"
```

- [ ] **Step 2: Add text_input module to lib.rs**

Add `pub mod text_input;` to `src/lib.rs`:

```rust
pub mod app;
pub mod board;
pub mod config;
pub mod item;
pub mod storage;
pub mod text_input;
```

- [ ] **Step 3: Create stub text_input.rs**

Create `src/text_input.rs`:

```rust
// Multi-line text editing state — ported from st project
```

- [ ] **Step 4: Create stub test file**

Create `tests/text_input.rs`:

```rust
// Tests for TextInputState
```

- [ ] **Step 5: Verify it builds**

Run: `cargo build`
Expected: compiles (unicode-segmentation fetched from crates.io)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/text_input.rs tests/text_input.rs
git commit -m "chore: add unicode-segmentation dep and text_input module stub"
```

---

### Task 2: Port TextInputState — core state and cursor movement

**Files:**
- Modify: `src/text_input.rs`
- Modify: `tests/text_input.rs`

**Interfaces:**
- Produces: `TextInputState::new(initial)`, `content()`, `cursor_offset()`, `selected_range()`, `move_to(offset)`, `move_right()`, `move_left()`, `move_to_home()`, `move_to_end()`

- [ ] **Step 1: Write the failing tests for construction and cursor movement**

Replace `tests/text_input.rs` with:

```rust
use dkb::text_input::TextInputState;

#[test]
fn test_new_with_content() {
    let state = TextInputState::new("hello");
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 0);
    assert!(state.selected_range().is_empty());
}

#[test]
fn test_new_empty() {
    let state = TextInputState::new("");
    assert_eq!(state.content(), "");
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_move_right() {
    let mut state = TextInputState::new("abc");
    state.move_right();
    assert_eq!(state.cursor_offset(), 1);
    state.move_right();
    assert_eq!(state.cursor_offset(), 2);
    state.move_right();
    assert_eq!(state.cursor_offset(), 3);
    state.move_right();
    assert_eq!(state.cursor_offset(), 3);
}

#[test]
fn test_move_left() {
    let mut state = TextInputState::new("abc");
    state.move_to(3);
    state.move_left();
    assert_eq!(state.cursor_offset(), 2);
    state.move_left();
    assert_eq!(state.cursor_offset(), 1);
    state.move_left();
    assert_eq!(state.cursor_offset(), 0);
    state.move_left();
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_move_to_home_end() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    assert_eq!(state.cursor_offset(), 5);
    state.move_to_home();
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_move_right_with_multibyte() {
    let mut state = TextInputState::new("café");
    state.move_right();
    assert_eq!(state.cursor_offset(), 1);
    state.move_right();
    assert_eq!(state.cursor_offset(), 2);
    state.move_right();
    assert_eq!(state.cursor_offset(), 3);
    state.move_right();
    assert_eq!(state.cursor_offset(), 5);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test text_input`
Expected: FAIL — `TextInputState` does not exist

- [ ] **Step 3: Implement TextInputState core**

Replace `src/text_input.rs` with:

```rust
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub struct TextInputState {
    content: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
}

impl TextInputState {
    pub fn new(initial: &str) -> Self {
        Self {
            content: initial.to_string(),
            selected_range: 0..0,
            selection_reversed: false,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn selected_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    pub fn move_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
    }

    pub fn move_right(&mut self) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            self.move_to(next);
        } else {
            self.move_to(self.selected_range.end);
        }
    }

    pub fn move_left(&mut self) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            self.move_to(prev);
        } else {
            self.move_to(self.selected_range.start);
        }
    }

    pub fn move_to_home(&mut self) {
        self.move_to(0);
    }

    pub fn move_to_end(&mut self) {
        self.move_to(self.content.len());
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test text_input`
Expected: PASS (6 tests)

- [ ] **Step 5: Commit**

```bash
git add src/text_input.rs tests/text_input.rs
git commit -m "feat: port TextInputState core with cursor movement"
```

---

### Task 3: Port TextInputState — text editing and selection

**Files:**
- Modify: `src/text_input.rs`
- Modify: `tests/text_input.rs`

**Interfaces:**
- Produces: `select_to(offset)`, `select_right()`, `select_left()`, `select_all()`, `insert(text)`, `backspace()`, `delete()`, `replace_range(range, text)`

- [ ] **Step 1: Write the failing tests for editing and selection**

Append to `tests/text_input.rs`:

```rust
// -- Text insertion --

#[test]
fn test_insert_at_cursor() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 5);
}

#[test]
fn test_insert_in_middle() {
    let mut state = TextInputState::new("hllo");
    state.move_right();
    state.insert("e");
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 2);
}

#[test]
fn test_insert_replaces_selection() {
    let mut state = TextInputState::new("hello world");
    state.move_to(0);
    state.select_to(5);
    state.insert("goodbye");
    assert_eq!(state.content(), "goodbye world");
    assert_eq!(state.cursor_offset(), 7);
}

#[test]
fn test_insert_multiline() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.insert("\nworld");
    assert_eq!(state.content(), "hello\nworld");
    assert_eq!(state.cursor_offset(), 11);
}

// -- Backspace / Delete --

#[test]
fn test_backspace() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.backspace();
    assert_eq!(state.content(), "hell");
    assert_eq!(state.cursor_offset(), 4);
}

#[test]
fn test_backspace_at_start() {
    let mut state = TextInputState::new("hello");
    state.backspace();
    assert_eq!(state.content(), "hello");
}

#[test]
fn test_backspace_deletes_selection() {
    let mut state = TextInputState::new("hello world");
    state.select_to(5);
    state.backspace();
    assert_eq!(state.content(), " world");
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_delete_forward() {
    let mut state = TextInputState::new("hello");
    state.delete();
    assert_eq!(state.content(), "ello");
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_delete_at_end() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.delete();
    assert_eq!(state.content(), "hello");
}

// -- Selection --

#[test]
fn test_select_right() {
    let mut state = TextInputState::new("hello");
    state.select_right();
    assert_eq!(state.selected_range(), 0..1);
    state.select_right();
    assert_eq!(state.selected_range(), 0..2);
}

#[test]
fn test_select_left() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.select_left();
    assert_eq!(state.selected_range(), 4..5);
}

#[test]
fn test_select_all() {
    let mut state = TextInputState::new("hello");
    state.select_all();
    assert_eq!(state.selected_range(), 0..5);
}

#[test]
fn test_move_collapses_selection() {
    let mut state = TextInputState::new("hello");
    state.select_all();
    state.move_right();
    assert!(state.selected_range().is_empty());
    assert_eq!(state.cursor_offset(), 5);
}

// -- Replace range --

#[test]
fn test_replace_range() {
    let mut state = TextInputState::new("hello world");
    state.replace_range(0..5, "goodbye");
    assert_eq!(state.content(), "goodbye world");
    assert_eq!(state.cursor_offset(), 7);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test text_input`
Expected: FAIL — `select_to`, `insert`, `backspace`, etc. do not exist

- [ ] **Step 3: Implement editing and selection methods**

Add to `impl TextInputState` in `src/text_input.rs`:

```rust
    pub fn select_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
    }

    pub fn select_right(&mut self) {
        let next = self.next_boundary(self.cursor_offset());
        self.select_to(next);
    }

    pub fn select_left(&mut self) {
        let prev = self.previous_boundary(self.cursor_offset());
        self.select_to(prev);
    }

    pub fn select_all(&mut self) {
        self.selected_range = 0..self.content.len();
        self.selection_reversed = false;
    }

    pub fn insert(&mut self, text: &str) {
        let range = self.selected_range.clone();
        self.content = self.content[..range.start].to_owned() + text + &self.content[range.end..];
        let new_pos = range.start + text.len();
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }

    pub fn backspace(&mut self) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if prev == self.cursor_offset() {
                return;
            }
            self.select_to(prev);
        }
        self.insert("");
    }

    pub fn delete(&mut self) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if next == self.cursor_offset() {
                return;
            }
            self.select_to(next);
        }
        self.insert("");
    }

    pub fn replace_range(&mut self, range: Range<usize>, text: &str) {
        self.content = self.content[..range.start].to_owned() + text + &self.content[range.end..];
        let new_pos = range.start + text.len();
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test text_input`
Expected: PASS (all tests)

- [ ] **Step 5: Commit**

```bash
git add src/text_input.rs tests/text_input.rs
git commit -m "feat: port TextInputState text editing and selection"
```

---

### Task 4: Port TextInputState — undo/redo, UTF-16, word boundaries

**Files:**
- Modify: `src/text_input.rs`
- Modify: `tests/text_input.rs`

**Interfaces:**
- Produces: `UndoEntry` struct, `undo()`, `redo()`, `offset_to_utf16()`, `offset_from_utf16()`, `range_to_utf16()`, `range_from_utf16()`, `word_start()`, `word_end()`, `select_word_at()`

- [ ] **Step 1: Write the failing tests for undo/redo, UTF-16, and word boundaries**

Append to `tests/text_input.rs`:

```rust
// -- Undo / Redo --

#[test]
fn test_undo_insert() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    assert_eq!(state.content(), "hello");
    state.undo();
    assert_eq!(state.content(), "");
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_redo_after_undo() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    state.undo();
    assert_eq!(state.content(), "");
    state.redo();
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 5);
}

#[test]
fn test_undo_backspace() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.backspace();
    assert_eq!(state.content(), "hell");
    state.undo();
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 5);
}

#[test]
fn test_multiple_undos() {
    let mut state = TextInputState::new("");
    state.insert("a");
    state.insert("b");
    state.insert("c");
    assert_eq!(state.content(), "abc");
    state.undo();
    assert_eq!(state.content(), "ab");
    state.undo();
    assert_eq!(state.content(), "a");
    state.undo();
    assert_eq!(state.content(), "");
}

#[test]
fn test_new_edit_clears_redo_stack() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    state.undo();
    assert_eq!(state.content(), "");
    state.insert("world");
    state.redo();
    assert_eq!(state.content(), "world");
}

// -- UTF-16 conversion --

#[test]
fn test_utf16_offset_ascii() {
    let state = TextInputState::new("hello");
    assert_eq!(state.offset_to_utf16(3), 3);
    assert_eq!(state.offset_from_utf16(3), 3);
}

#[test]
fn test_utf16_offset_multibyte() {
    let state = TextInputState::new("€");
    assert_eq!(state.offset_to_utf16(3), 1);
    assert_eq!(state.offset_from_utf16(1), 3);
}

// -- Word boundaries --

#[test]
fn test_word_start_in_middle_of_word() {
    let state = TextInputState::new("hello world");
    assert_eq!(state.word_start(3), 0);
}

#[test]
fn test_word_end_from_beginning() {
    let state = TextInputState::new("hello world");
    assert_eq!(state.word_end(0), 5);
}

#[test]
fn test_select_word_at() {
    let mut state = TextInputState::new("hello world");
    state.select_word_at(3);
    assert_eq!(state.selected_range(), 0..5);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test text_input`
Expected: FAIL — `undo`, `redo`, `offset_to_utf16`, etc. do not exist

- [ ] **Step 3: Implement undo/redo, UTF-16, and word boundaries**

Add `UndoEntry` struct and undo/redo infrastructure to `src/text_input.rs`:

```rust
#[derive(Clone)]
struct UndoEntry {
    content: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
}
```

Add `undo_stack` and `redo_stack` fields to `TextInputState`:

```rust
pub struct TextInputState {
    content: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
}
```

Update `TextInputState::new` to initialize the stacks:

```rust
pub fn new(initial: &str) -> Self {
    Self {
        content: initial.to_string(),
        selected_range: 0..0,
        selection_reversed: false,
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
    }
}
```

Add a `save_undo` method and update `insert`, `backspace`, `delete`, and `replace_range` to call it:

```rust
    fn save_undo(&mut self) {
        self.undo_stack.push(UndoEntry {
            content: self.content.clone(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        });
        self.redo_stack.clear();
    }
```

Update `insert` to call `save_undo()` first:

```rust
    pub fn insert(&mut self, text: &str) {
        self.save_undo();
        let range = self.selected_range.clone();
        self.content = self.content[..range.start].to_owned() + text + &self.content[range.end..];
        let new_pos = range.start + text.len();
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }
```

Update `replace_range` to call `save_undo()` first:

```rust
    pub fn replace_range(&mut self, range: Range<usize>, text: &str) {
        self.save_undo();
        self.content = self.content[..range.start].to_owned() + text + &self.content[range.end..];
        let new_pos = range.start + text.len();
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }
```

Add undo/redo methods:

```rust
    pub fn undo(&mut self) {
        if let Some(entry) = self.undo_stack.pop() {
            self.redo_stack.push(UndoEntry {
                content: self.content.clone(),
                selected_range: self.selected_range.clone(),
                selection_reversed: self.selection_reversed,
            });
            self.content = entry.content;
            self.selected_range = entry.selected_range;
            self.selection_reversed = entry.selection_reversed;
        }
    }

    pub fn redo(&mut self) {
        if let Some(entry) = self.redo_stack.pop() {
            self.undo_stack.push(UndoEntry {
                content: self.content.clone(),
                selected_range: self.selected_range.clone(),
                selection_reversed: self.selection_reversed,
            });
            self.content = entry.content;
            self.selected_range = entry.selected_range;
            self.selection_reversed = entry.selection_reversed;
        }
    }
```

Add UTF-16 conversion methods:

```rust
    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    pub fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }
```

Add word boundary methods:

```rust
    pub fn word_start(&self, offset: usize) -> usize {
        for (idx, segment) in self.content.split_word_bound_indices() {
            let end = idx + segment.len();
            if offset >= idx && offset < end {
                return idx;
            }
        }
        if let Some((idx, _)) = self.content.split_word_bound_indices().next_back() {
            return idx;
        }
        0
    }

    pub fn word_end(&self, offset: usize) -> usize {
        for (idx, segment) in self.content.split_word_bound_indices() {
            let end = idx + segment.len();
            if offset >= idx && offset < end {
                return end;
            }
        }
        self.content.len()
    }

    pub fn select_word_at(&mut self, offset: usize) {
        let start = self.word_start(offset);
        let end = self.word_end(offset);
        self.selected_range = start..end;
        self.selection_reversed = false;
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test text_input`
Expected: PASS (all tests)

- [ ] **Step 5: Commit**

```bash
git add src/text_input.rs tests/text_input.rs
git commit -m "feat: port TextInputState undo/redo, UTF-16, word boundaries"
```

---

### Task 5: ItemEditor view — EntityInputHandler and rendering

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `TextInputState` from Task 2-4
- Produces: `ItemEditor` GPUI view with `EntityInputHandler` impl, modal overlay rendering, Save/Cancel/Tear-Off buttons

**Note:** This task has no unit tests — it is GPUI rendering code. The verification gate is `cargo build` succeeding. The `TextInputState` it wraps is fully tested in Tasks 2-4.

- [ ] **Step 1: Add ItemEditor struct and EntityInputHandler impl to app.rs**

Add these imports to the top of `src/app.rs`:

```rust
use gpui::{
    App, Context, FocusHandle, Focusable, KeyBinding, Menu, MenuItem, Render, Window,
    actions, div, prelude::*, px, rgb,
    ClipboardItem, ElementInputHandler, EntityInputHandler, UTF16Selection,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Bounds, SharedString, TextRun, TextAlign, CursorStyle,
};
use crate::text_input::TextInputState;
```

Add `actions` for the editor:

```rust
actions!(
    dkb_editor,
    [
        EditorBackspace,
        EditorDelete,
        EditorLeft,
        EditorRight,
        EditorSelectLeft,
        EditorSelectRight,
        EditorSelectAll,
        EditorUndo,
        EditorRedo,
        EditorEscape,
        EditorSave,
        EditorPaste,
        EditorCut,
        EditorCopy,
    ]
);
```

Add the `ItemEditor` struct:

```rust
pub struct ItemEditor {
    pub state: TextInputState,
    pub focus_handle: FocusHandle,
    pub editing_item_id: Option<Uuid>,
    pub is_torn_off: bool,
}

impl ItemEditor {
    pub fn new(cx: &mut Context<Self>, initial: &str, editing_item_id: Option<Uuid>) -> Self {
        Self {
            state: TextInputState::new(initial),
            focus_handle: cx.focus_handle().tab_stop(true),
            editing_item_id,
            is_torn_off: false,
        }
    }

    pub fn content(&self) -> &str {
        self.state.content()
    }
}
```

Add `Focusable` impl:

```rust
impl Focusable for ItemEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
```

Add `EntityInputHandler` impl (delegates to `TextInputState`):

```rust
impl EntityInputHandler for ItemEditor {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.state.range_from_utf16(&range_utf16);
        actual_range.replace(self.state.range_to_utf16(&range));
        Some(self.state.content()[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let range = self.state.selected_range();
        Some(UTF16Selection {
            range: self.state.range_to_utf16(&range),
            reversed: range.start > range.end,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.state.range_from_utf16(r))
            .unwrap_or_else(|| self.state.selected_range());
        self.state.replace_range(range, new_text);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.state.range_from_utf16(r))
            .unwrap_or_else(|| self.state.selected_range());
        self.state.replace_range(range, new_text);
        cx.notify();
    }
}
```

- [ ] **Step 2: Add Render impl for ItemEditor**

```rust
impl Render for ItemEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content: SharedString = self.state.content().to_string().into();
        let selected_range = self.state.selected_range();
        let cursor_offset = self.state.cursor_offset();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xffffff))
            .track_focus(&self.focus_handle)
            .key_context("ItemEditor")
            .on_action(cx.listener(|this, _: &EditorBackspace, _, cx| { this.state.backspace(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorDelete, _, cx| { this.state.delete(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorLeft, _, cx| { this.state.move_left(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorRight, _, cx| { this.state.move_right(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorSelectLeft, _, cx| { this.state.select_left(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorSelectRight, _, cx| { this.state.select_right(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorSelectAll, _, cx| { this.state.select_all(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorUndo, _, cx| { this.state.undo(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorRedo, _, cx| { this.state.redo(); cx.notify(); }))
            .on_action(cx.listener(|this, _: &EditorPaste, _, cx| {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    this.state.insert(&text);
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &EditorCopy, _, cx| {
                let range = this.state.selected_range();
                if !range.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        this.state.content()[range].to_string(),
                    ));
                }
            }))
            .on_action(cx.listener(|this, _: &EditorCut, _, cx| {
                let range = this.state.selected_range();
                if !range.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        this.state.content()[range].to_string(),
                    ));
                    this.state.insert("");
                    cx.notify();
                }
            }))
            .child(
                div()
                    .flex_1()
                    .p(px(16.))
                    .overflow_y_scroll()
                    .child(
                        div()
                            .id("editor-text")
                            .w_full()
                            .h_full()
                            .text_sm()
                            .text_color(rgb(0x333333))
                            .text(content.clone())
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .p(px(12.))
                    .border_t_1()
                    .border_color(rgb(0xe0e0e0))
                    .child(
                        div()
                            .px(px(16.))
                            .py(px(6.))
                            .rounded(px(4.))
                            .bg(rgb(0x4488ff))
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .cursor_pointer()
                            .child("Save")
                    )
                    .child(
                        div()
                            .px(px(16.))
                            .py(px(6.))
                            .rounded(px(4,))
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xcccccc))
                            .text_color(rgb(0x333333))
                            .text_sm()
                            .cursor_pointer()
                            .child("Cancel")
                    )
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(6.))
                            .rounded(px(4.))
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xcccccc))
                            .text_sm()
                            .cursor_pointer()
                            .child("⤢")
                    )
            )
    }
}
```

- [ ] **Step 3: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: add ItemEditor view with EntityInputHandler and rendering"
```

---

### Task 6: Wire ItemEditor into KanbanView as modal overlay

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `ItemEditor` from Task 5, `Storage`, `Item`, `Board` from prior tasks
- Produces: modal overlay when `editing_item` is `Some`, replaces `quick_add_active` placeholder

- [ ] **Step 1: Replace quick_add_active with editing_item field**

In `KanbanView`, replace:

```rust
pub quick_add_active: bool,
```

with:

```rust
pub editing_item: Option<EditingState>,
```

Add the `EditingState` struct:

```rust
pub struct EditingState {
    pub editor: gpui::Entity<ItemEditor>,
    pub is_new: bool,
    pub item_id: Option<Uuid>,
}
```

Update `KanbanView::new` to replace `quick_add_active: false` with `editing_item: None`.

- [ ] **Step 2: Replace on_new_item and commit_quick_add with editor-based flow**

Replace `on_new_item`:

```rust
fn on_new_item(&mut self, _: &NewItem, _window: &mut Window, cx: &mut Context<Self>) {
    let editor = cx.new(|cx| ItemEditor::new(cx, "", None));
    self.editing_item = Some(EditingState {
        editor,
        is_new: true,
        item_id: None,
    });
    // Focus the editor
    cx.defer(|this, cx| {
        if let Some(ref editing) = this.editing_item {
            cx.focus(&editing.editor.read(cx).focus_handle);
        }
    });
    cx.notify();
}
```

Replace `commit_quick_add` with:

```rust
fn save_editor(&mut self, cx: &mut Context<Self>) {
    let Some(editing) = self.editing_item.take() else {
        return;
    };
    let content = editing.editor.read(cx).content().to_string();
    let title = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("").to_string();
    if title.is_empty() {
        cx.notify();
        return;
    }

    if editing.is_new {
        // Create new item
        let item = Item::new(&content);
        let location = match self.current_screen {
            Screen::Backlog => Location::Backlog,
            Screen::Active => Location::Active(Category::Today),
            Screen::Done => Location::Backlog,
        };
        if Storage::write_item(&self.config.data_dir, &item, &location).is_ok() {
            self.board.insert_item(item, &location);
        }
    } else if let Some(id) = editing.item_id {
        // Update existing item
        if let Some(location) = self.board.find_item_location(&id) {
            if let Some(item) = self.board.find_item_mut(&id) {
                item.body = content;
                item.updated_at = chrono::Utc::now();
            }
            let item_ref = self.board.find_item(&id).cloned();
            if let Some(item) = item_ref {
                Storage::write_item(&self.config.data_dir, &item, &location).ok();
            }
        }
    }
    cx.notify();
}

fn cancel_editor(&mut self, cx: &mut Context<Self>) {
    self.editing_item = None;
    cx.notify();
}
```

- [ ] **Step 3: Add double-click to edit existing items**

Add an `on_double_click` to `render_item_card`:

```rust
fn render_item_card(&self, item: &Item, cx: &mut Context<Self>) -> impl IntoElement {
    let is_selected = self.selected_item == Some(item.id);
    let item_id = item.id;
    let item_body = item.body.clone();
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
        .on_double_click(
            cx.listener(move |this, _, _window, cx| {
                let editor = cx.new(|cx| ItemEditor::new(cx, &item_body, Some(item_id)));
                this.editing_item = Some(EditingState {
                    editor,
                    is_new: false,
                    item_id: Some(item_id),
                });
                cx.defer(|this, cx| {
                    if let Some(ref editing) = this.editing_item {
                        cx.focus(&editing.editor.read(cx).focus_handle);
                    }
                });
                cx.notify();
            }),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x333333))
                .child(item.title()),
        )
}
```

- [ ] **Step 4: Render the modal overlay**

Replace `render_quick_add_bar` with `render_editor_modal`:

```rust
fn render_editor_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
    if let Some(ref editing) = self.editing_item {
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(rgba(0x00000040))
            .flex()
            .justify_center()
            .items_center()
            .child(
                div()
                    .w(px(600.))
                    .h(px(400.))
                    .bg(rgb(0xffffff))
                    .rounded(px(8.))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(editing.editor.clone())
            )
    } else {
        div()
    }
}
```

Update `KanbanView::render` to replace the quick-add bar call with the modal:

Replace `.child(self.render_quick_add_bar(cx))` with `.child(self.render_editor_modal(cx))`.

Also wire `EditorEscape` and `EditorSave` actions on the root div:

```rust
.on_action(cx.listener(|this, _: &EditorEscape, _, cx| this.cancel_editor(cx)))
.on_action(cx.listener(|this, _: &EditorSave, _, cx| this.save_editor(cx)))
```

- [ ] **Step 5: Add keybindings for editor**

Add to `key_bindings()`:

```rust
KeyBinding::new("escape", EditorEscape, Some("ItemEditor")),
KeyBinding::new("cmd-s", EditorSave, Some("ItemEditor")),
KeyBinding::new("backspace", EditorBackspace, Some("ItemEditor")),
KeyBinding::new("delete", EditorDelete, Some("ItemEditor")),
KeyBinding::new("left", EditorLeft, Some("ItemEditor")),
KeyBinding::new("right", EditorRight, Some("ItemEditor")),
KeyBinding::new("shift-left", EditorSelectLeft, Some("ItemEditor")),
KeyBinding::new("shift-right", EditorSelectRight, Some("ItemEditor")),
KeyBinding::new("cmd-a", EditorSelectAll, Some("ItemEditor")),
KeyBinding::new("cmd-v", EditorPaste, Some("ItemEditor")),
KeyBinding::new("cmd-c", EditorCopy, Some("ItemEditor")),
KeyBinding::new("cmd-x", EditorCut, Some("ItemEditor")),
KeyBinding::new("cmd-z", EditorUndo, Some("ItemEditor")),
KeyBinding::new("cmd-shift-z", EditorRedo, Some("ItemEditor")),
```

- [ ] **Step 6: Add Save/Cancel/Tear-Off button click handlers**

The buttons need click handlers. Update the button divs in `ItemEditor::render` to call actions:

```rust
// Save button
div()
    .px(px(16.))
    .py(px(6.))
    .rounded(px(4.))
    .bg(rgb(0x4488ff))
    .text_color(rgb(0xffffff))
    .text_sm()
    .cursor_pointer()
    .on_mouse_down(MouseButton::Left, cx.listener(|_, _: &MouseDownEvent, _, cx| {
        cx.dispatch_action(&EditorSave, cx.entity());
    }))
    .child("Save")
```

Actually, GPUI's action dispatch from a child view to parent is complex. The simpler approach: have the buttons emit events via `cx.notify()` and let `KanbanView` observe. But the simplest correct approach is to handle clicks directly in the modal overlay rendering, not in `ItemEditor::render`.

Revised approach: Move the button bar out of `ItemEditor::render` into the modal overlay in `KanbanView::render_editor_modal`. This way the buttons can call `KanbanView` methods directly.

- [ ] **Step 7: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 8: Commit**

```bash
git add src/app.rs
git commit -m "feat: wire ItemEditor as modal overlay with save/cancel/tear-off"
```

---

### Task 7: Add tear-off to separate window

**Files:**
- Modify: `src/app.rs`

**Interfaces:**
- Produces: `tear_off_editor` method that opens a new GPUI window with the `ItemEditor` view

- [ ] **Step 1: Add tear_off_editor method**

Add to `KanbanView`:

```rust
fn tear_off_editor(&mut self, cx: &mut Context<Self>) {
    let Some(editing) = self.editing_item.take() else {
        return;
    };
    let content = editing.editor.read(cx).content().to_string();
    let item_id = editing.item_id;
    let is_new = editing.is_new;

    let window = cx.open_window(
        gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(600.), px(400.)),
                cx,
            ))),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Edit Item".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        },
        |_, cx| {
            cx.new(|cx| ItemEditor::new(cx, &content, item_id))
        },
    );

    if let Ok(window) = window {
        window.update(cx, |view, window, cx| {
            window.focus(&view.focus_handle(cx), cx);
        }).ok();
    }

    // Store the torn-off editor so we can retrieve its content when the window closes
    // For now, the torn-off editor is independent — when the user saves in the
    // separate window, it commits directly to storage.
    cx.notify();
}
```

- [ ] **Step 2: Add tear-off button to the modal overlay**

In `render_editor_modal`, add the tear-off button with the expand icon (⤢) and tooltip:

```rust
div()
    .px(px(12.))
    .py(px(6.))
    .rounded(px(4.))
    .bg(rgb(0xffffff))
    .border_1()
    .border_color(rgb(0xcccccc))
    .text_sm()
    .cursor_pointer()
    .child("⤢")
    .tooltip({
        let tooltip = gpui::Tooltip::text("tear off window");
        tooltip
    })
    .on_mouse_down(
        MouseButton::Left,
        cx.listener(|this, _: &MouseDownEvent, _, cx| {
            this.tear_off_editor(cx);
        }),
    )
```

- [ ] **Step 3: Verify it builds**

Run: `cargo build`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: add tear-off to separate window with expand icon"
```

---

### Task 8: Clean up and final verification

**Files:**
- Modify: `src/app.rs` (remove dead code, fix warnings)
- Verify: all tests pass

- [ ] **Step 1: Run clippy**

Run: `cargo clippy`
Expected: no warnings

- [ ] **Step 2: Run all tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 3: Verify the app launches**

Run: `cargo run`
Expected: window opens, cmd-n opens editor modal, typing works, save creates item

- [ ] **Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: clean up warnings from editor integration"
```
