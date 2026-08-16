# Editor Robustness Improvement Plan

**Goal:** Fix four editor issues: (1) sluggish editing from filesystem reads on every frame, (2) cursor rendering below line numbers due to off-by-one line count, (3) backspace not continuing to delete from the last line in vi mode, (4) Esc incorrectly dismissing the subitem prompt — it must always route to vi mode.

**Architecture:** The editor lives in `crates/dkb/src/editor.rs` (GPUI element + `ItemEditor`), the text model in `crates/dkb-core/src/text_input.rs`, and the vi engine in `crates/dkb-core/src/vi.rs`. Keybindings are in `crates/dkb/src/app.rs`.

**Tech Stack:** Rust 2024, GPUI (Zed framework).

## Global Constraints

- Must compile under Rust 2024 edition.
- Zero clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic`).
- Maintain existing tests; write new tests for each task (red → green).
- Commit after each step.

---

### Task 1: Cache board data to eliminate per-frame filesystem reads

**Problem:** `render()` calls `Storage::load_board()` twice per frame — once in `is_done()` ([editor.rs:510](L510)) and once in the breadcrumb bar ([editor.rs:1089](L1089)). Each call reads and parses every `.md` file from disk. `render()` runs on every keystroke.

**Files:**
- Modify: `crates/dkb/src/editor.rs`
- Test: `crates/dkb/tests/app.rs`

**Plan:**
- [ ] Step 1: Write test asserting `is_done()` does not re-read disk when board is cached
- [ ] Step 2: Verify test fails
- [ ] Step 3: Add `cached_done: Option<bool>` field to `ItemEditor`; set it when the item is saved or loaded; have `is_done()` return the cached value (refresh on save)
- [ ] Step 4: Cache breadcrumb titles — store `history_stack` as `Vec<(Uuid, String)>` with the title captured at push time, so the breadcrumb bar doesn't call `load_board`
- [ ] Step 5: Verify tests pass

---

### Task 2: Fix line count to include trailing empty line

**Problem:** `content.lines().count()` ([editor.rs:962](L962)) doesn't count the final empty line after a trailing `\n`. The cursor can be on that line (offset after `\n`), but there's no gutter number for it.

**Files:**
- Modify: `crates/dkb-core/src/text_input.rs`
- Test: `crates/dkb-core/tests/text_input.rs`

**Plan:**
- [ ] Step 1: Write test for `line_count()` method returning correct count including trailing empty line
- [ ] Step 2: Verify test fails
- [ ] Step 3: Add `pub fn line_count(&self) -> usize` to `TextInputState` that counts `\n` + 1 (so `"a\nb"` = 2, `"a\nb\n"` = 3, `""` = 1)
- [ ] Step 4: Use `line_count()` in `render()` gutter generation
- [ ] Step 5: Verify tests pass

---

### Task 3: Fix vi-mode backspace to delete across line boundary

**Problem:** In vi Normal mode, backspace is handled by `vi_state.handle_key("backspace", ...)` which falls through to `handle_normal_key`, which has no backspace handler — so it returns `ViActionResult::None` and nothing happens. Backspace should move the cursor left (like `h`) or, when at column 0, join to the previous line.

**Files:**
- Modify: `crates/dkb-core/src/vi.rs`
- Test: `crates/dkb-core/tests/vi.rs`

**Plan:**
- [ ] Step 1: Write test: in Normal mode, backspace at column > 0 moves cursor left
- [ ] Step 2: Write test: in Normal mode, backspace at column 0 on a non-first line joins with previous line
- [ ] Step 3: Verify tests fail
- [ ] Step 4: Add backspace handler to `handle_normal_key` that moves left (same as `h`) — in vi, backspace in Normal mode is a motion equivalent to `h`
- [ ] Step 5: Verify tests pass

---

### Task 4: Esc must never dismiss the subitem prompt

**Problem:** Esc dismisses the subitem prompt ([editor.rs:1018](L1018) and [editor.rs:339](L339)) instead of routing to vi mode. Esc is a critical vi key (Insert→Normal) and must always reach the editor's vi handler.

**Files:**
- Modify: `crates/dkb/src/editor.rs`
- Test: `crates/dkb/tests/app.rs`

**Plan:**
- [ ] Step 1: Write test: with subitem prompt open in vi mode, Esc enters Normal mode and does NOT close the prompt
- [ ] Step 2: Verify test fails
- [ ] Step 3: Remove the Esc → `cancel_subitem_prompt` path from both `on_key_down` and `on_escape`; let Esc always flow to the vi handler
- [ ] Step 4: Verify tests pass

---

### Task 5: Full verification

- [ ] Run `cargo test`
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic`
- [ ] Update `UserGuide.md` if needed
- [ ] Move plan to `docs/done/plans/`
