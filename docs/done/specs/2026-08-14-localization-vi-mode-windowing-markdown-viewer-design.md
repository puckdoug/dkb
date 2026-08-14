# Daily Kanban (dkb) — Localization, Vi-Mode, Windowing & Markdown Viewer Design Spec

## 1. Overview & Scope

This specification addresses all findings and feature requirements identified in `docs/2026-08-14-findings2.md`:
1. **Full Localization**: Support for all 44 macOS-supported languages, defaulting to the OS language on startup, with a configurable language selector in Settings and dynamic runtime language switching for all UI strings and macOS menu items.
2. **Comprehensive Vi-Mode Editing & Command-Line (`:`)**: Complete modal editing engine supporting operators, counts, text objects, motions, registers, search (`/`, `?`), and Ex command line (`:`) supporting buffer/file commands (`:w`, `:q`, `:wq`, `:x`, `:q!`), line jumps (`:<line>`), substitutions (`:%s/old/new/g`), and line deletions.
3. **Editor Close Scoping (`cmd-w`)**: Proper scoping for `cmd-w` so that closing an attached/modal editor dismisses only the editor without closing the main Kanban window.
4. **Main Window Management (`cmd-option-n`)**: Ability to reopen or open multiple independent main Kanban windows via shortcut (`cmd-option-n`) and menu item.
5. **External Markdown Viewer Integration (`cmd-shift-m`)**: Configurable markdown viewer executable/app path (via Settings file picker) with item opening via shortcut (`cmd-shift-m`), Item menu, and right-click context menu.

---

## 2. Architecture & Module Structure

```
src/
├── app.rs             # Multi-window support, context menus, action handlers, localized menus & tabs
├── board.rs           # Kanban board state and card structures
├── config.rs          # Config model with language, markdown_viewer settings
├── editor.rs          # Editor view, modal cmd-w isolation, vi command line (:) & search bar UI
├── i18n/
│   ├── mod.rs         # Localization engine, OS language detection, t!() macro / lookup
│   └── locales.rs     # Bundled translation dictionary across 44 macOS languages
├── item.rs            # Item model, title formatting
├── link.rs            # Sub-item link handling
├── storage.rs         # File I/O and board persistence
├── text_input.rs      # Low-level text buffer, cursor/selection tracking
├── theme.rs           # Theme engine (Light/Dark/System)
├── viewer.rs          # External viewer launcher (Marked, MD-Viewer, custom app)
└── vi.rs              # Comprehensive Vi state machine, counts, motions, operators, Ex parser
```

---

## 3. Subsystem Specifications

### 3.1 Localization System (`i18n/`)

#### 1. Supported Languages (44 macOS Locales + Auto-Detection)
- `SystemAuto`: Detects OS locale via macOS `CFLocaleCopyCurrent` / `defaults read -g AppleLanguages` / environment with fallback to `en-US`.
- 44 Explicit Locales:
  - Arabic (`ar`)
  - Catalan (`ca`)
  - Chinese (Simplified) (`zh-Hans`)
  - Chinese (Traditional) (`zh-Hant`)
  - Croatian (`hr`)
  - Czech (`cs`)
  - Danish (`da`)
  - Dutch (`nl`)
  - English (Australia) (`en-AU`)
  - English (Canada) (`en-CA`)
  - English (India) (`en-IN`)
  - English (Japan) (`en-JP`)
  - English (UK) (`en-GB`)
  - English (US) (`en-US`)
  - Finnish (`fi`)
  - French (Canada) (`fr-CA`)
  - French (France) (`fr-FR`)
  - German (`de`)
  - Greek (`el`)
  - Hebrew (`he`)
  - Hindi (`hi`)
  - Hungarian (`hu`)
  - Indonesian (`id`)
  - Italian (`it`)
  - Japanese (`ja`)
  - Korean (`ko`)
  - Malay (`ms`)
  - Norwegian (`nb`)
  - Polish (`pl`)
  - Portuguese (Brazil) (`pt-BR`)
  - Portuguese (Portugal) (`pt-PT`)
  - Romanian (`ro`)
  - Russian (`ru`)
  - Slovak (`sk`)
  - Spanish (Chile) (`es-CL`)
  - Spanish (Latin America) (`es-419`)
  - Spanish (Mexico) (`es-MX`)
  - Spanish (Spain) (`es-ES`)
  - Spanish (United States) (`es-US`)
  - Swedish (`sv`)
  - Thai (`th`)
  - Turkish (`tr`)
  - Ukrainian (`uk`)
  - Vietnamese (`vi`)

#### 2. Key Structure & String Catalog
- All UI strings, menu labels, tabs, column headers, dialogs, button titles, tooltips, and settings options use localized keys:
  - Tabs: `tab.backlog`, `tab.active`, `tab.done`, `tab.settings`
  - Columns: `col.yesterday`, `col.today`, `col.this_week`, `col.next_week`, `col.backlog`, `col.done`, `col.sub_items`
  - Menus: `menu.file`, `menu.file.new_item`, `menu.file.new_sub_item`, `menu.file.new_window`, `menu.file.close_window`, `menu.view`, `menu.item`, `menu.item.open_markdown_viewer`, `menu.item.mark_done`, `menu.item.delete`, `menu.app.settings`, `menu.app.quit`
  - Settings: `settings.title`, `settings.appearance`, `settings.vi_mode`, `settings.line_numbers`, `settings.language`, `settings.markdown_viewer`, `settings.storage_dir`, `settings.browse`
  - Editor: `editor.save`, `editor.cancel`, `editor.tear_off`, `editor.status.normal`, `editor.status.insert`, `editor.status.visual`, `editor.status.command`
- Fallback resolution order: `Selected Region Locale -> Base Language (e.g. es-MX -> es) -> en-US`.

#### 3. Configuration & Runtime Switch
- Stored in `config.toml` as `language = "auto"` (or specific locale code e.g. `language = "fr-FR"`).
- When changed in the Settings UI, updates `Config`, refreshes the active view, and re-registers macOS application menus (`KanbanView::setup_menus(cx)`) with localized text.

---

### 3.2 Comprehensive Vi-Mode Editing Engine & Command Line (`vi.rs`, `editor.rs`)

#### 1. Modal State Machine
- `ViMode::Normal`: Navigation, counts, operators, text objects.
- `ViMode::Insert`: Direct character insertion; `Esc` / `Ctrl-[` returns to Normal mode.
- `ViMode::Visual(VisualKind)`: Character-wise (`v`) or Line-wise (`V`) selection.
- `ViMode::Command`: Command-line Ex mode triggered by `:` in Normal mode.
- `ViMode::Search(SearchDirection)`: Incremental search triggered by `/` (forward) or `?` (backward).
- `ViMode::Replace`: Single character replace triggered by `r<char>`.
- `ViMode::OperatorPending(Op)`: Awaiting motion or text object (e.g. `d`, `c`, `y`, `gu`, `gU`, `>`, `<`).

#### 2. Command Grammar & Coverage
- **Counts**: Number prefixes on motions and operators (e.g. `3j`, `5w`, `2dd`, `4x`, `10gg`).
- **Motions**:
  - Basic: `h`, `j`, `k`, `l`
  - Word: `w`, `W` (next word start), `b`, `B` (prev word start), `e`, `E` (word end), `ge`, `gE` (prev word end)
  - Line: `0` (column 0), `^` (first non-blank), `$` (line end), `_` (current line non-blank)
  - Buffer: `gg` (first line), `G` (last line / `[count]G`), `:<num>`
  - Inline Find: `f<char>`, `F<char>`, `t<char>`, `T<char>`, `;` (repeat), `,` (reverse repeat)
  - Structure: `%` (matching pair `()`, `[]`, `{}`), `{`, `}` (paragraph motions)
- **Operators & Shortcuts**:
  - `d{motion}`, `dd`, `D` (`d$`)
  - `c{motion}`, `cc`, `C` (`c$`), `s` (`cl`), `S` (`cc`)
  - `y{motion}`, `yy`, `Y` (`y$`)
  - `x`, `X` (delete char back), `r<char>`, `~` (toggle case), `J` (join lines)
  - `>`, `<` (indent/outdent)
  - `u` (undo), `Ctrl-r` (redo)
  - `o` (open below), `O` (open above), `i`, `I`, `a`, `A`
  - `p`, `P` (paste after/before with line-wise / inline awareness)
- **Text Objects**:
  - `iw`, `aw` (inner/around word)
  - `i"`, `a"`, `i'`, `a'`, `i```, `a``` (inner/around quotes)
  - `i(`, `a(`, `i)`, `a)`, `i[`, `a[`, `i]`, `a]`, `i{`, `a{`, `i}`, `a}` (brackets)
  - `ip`, `ap` (paragraphs)
- **Search (`/`, `?`)**:
  - Interactive bottom search input with `/` or `?`.
  - `n` for next match, `N` for previous match.
  - `*` (search current word forward), `#` (search current word backward).

#### 3. Ex Command Line (`:`) UI & Execution
- In Normal mode, pressing `:` opens the command bar at the bottom of the editor.
- **Commands**:
  - `:w` / `:write`: Save item.
  - `:q` / `:quit`: Dismiss / close editor without saving if unchanged.
  - `:q!`: Force close editor without saving.
  - `:wq` / `:x` / `:xit`: Save item and close editor.
  - `:<number>`: Jump cursor to line number.
  - `:%s/pattern/replacement/[flags]`: Buffer-wide regex substitution (`g` = all occurrences per line, `i` = case-insensitive).
  - `:<range>s/pattern/replacement/[flags]`: Range substitution.
  - `:d` / `:delete`: Delete current line or range.
- `Escape` cancels command-line mode without executing.

---

### 3.3 Window Management & Scoped `cmd-w` (`app.rs`, `editor.rs`)

#### 1. Modal Editor `cmd-w` Scoping
- In `KanbanView`:
  - When modal editor is open (`self.editing.is_some()`), `cmd-w` triggers `CancelEditor`, closing only the editor modal and restoring board focus.
  - When no modal editor is open, `cmd-w` closes the window via `window.remove_window()`.
- In torn-off editor window:
  - `cmd-w` closes the torn-off editor window.

#### 2. Reopening & Multi-Window Support
- **Keybinding**: `cmd-option-n` bound to `OpenNewMainWindow`.
- **Menu Bar**: "File -> New Window" (`cmd-option-n`).
- **Behavior**:
  - Opens a new GPUI window hosting an independent `KanbanView`.
  - Multiple main windows operate concurrently, reading and persisting to the shared `data_dir`.
  - Closing a window only removes that window instance; the application remains active while at least one window exists or until `cmd-q` is pressed.

---

### 3.4 External Markdown Viewer Launch & Settings (`viewer.rs`, `config.rs`, `app.rs`)

#### 1. Configuration & Auto-Detection Priority
- Pre-selection priority order on initialization / auto-detection:
  1. **Marked** (`/Applications/Marked.app` or `Marked.app` in `~/Applications` / Spotlight)
  2. **Marked 2** (`/Applications/Marked 2.app` or `Marked 2.app` in `~/Applications` / Spotlight)
  3. **MD-Viewer** (`/Applications/MD-Viewer.app` or `MD-Viewer.app` in `~/Applications` / Spotlight)
  4. Any other application configured by user via file dialog or system default markdown viewer.
- Stored in `config.toml`:
  ```toml
  markdown_viewer = "/Applications/Marked.app" # or "auto" / custom path
  ```
- In Settings Screen:
  - "Markdown Viewer" section displaying current application path.
  - "Browse..." button using native macOS file dialog (`rfd::FileDialog`) allowing `.app` or binary selection.
  - "Reset to Auto-Detect" button resetting to the highest-priority detected viewer from the list above.

#### 2. Opening Items in Viewer
- **Actions & Shortcuts**:
  - Shortcut: `cmd-shift-m` on selected item.
  - Menu: "Item -> Open in Markdown Viewer".
  - Context Menu: Right-clicking any item card opens a context menu with "Open in Markdown Viewer", "Open / Edit", "Mark Done", "Delete", etc.
- **Execution**:
  - Resolves item file path: `<data_dir>/<location>/<item_id>.md`.
  - Uses `std::process::Command` to invoke `open -a "<viewer_path>" "<item_file_path>"` (or `open "<item_file_path>"` if default).

---

## 4. Testing & Verification Plan

1. **Localization Tests** (`tests/i18n.rs`):
   - Verify string key resolution across all 44 language codes.
   - Verify fallback hierarchy (e.g. `es-CL` -> `es` -> `en-US`).
   - Verify system auto-detection and config serialization.
2. **Vi-Mode Tests** (`tests/vi.rs`):
   - Test operators (`d`, `c`, `y`) with counts and motions (`3dw`, `2c$`, `5j`).
   - Test text objects (`ciw`, `di"`, `da(`, `yap`).
   - Test Ex commands (`:w`, `:q`, `:wq`, `:q!`, `:12`, `:%s/foo/bar/g`).
   - Test search motions (`/pattern`, `?pattern`, `n`, `N`, `*`, `#`).
3. **Windowing & Action Tests** (`tests/app.rs`):
   - Test `cmd-w` dismisses editor when `editing.is_some()`.
   - Test `OpenNewMainWindow` creates new window instance.
4. **Viewer Tests** (`tests/viewer.rs`):
   - Test viewer command building and path resolution.
5. **Quality Verification**:
   - `cargo test`
   - `cargo clippy --all-targets`
   - Test run of binary and GUI interactions.
