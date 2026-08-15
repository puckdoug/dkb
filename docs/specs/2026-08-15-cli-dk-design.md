# CLI (`dk`) Design Spec

Date: 2026-08-15

## Purpose

A command-line tool `dk` for quick interaction with the same daily-kanban
files managed by the GUI. Built and distributed alongside the GUI binary as
part of the macOS package. Both binaries are first-class and always built
together via a bare `cargo build`.

## Constraints

- `dk` must not include any GUI code (no gpui/gpui_platform dependency).
- Bare `cargo build` produces both `dkb` (gui) and `dk` binaries.
- `dk` reuses the existing core modules (item, board, storage, config, i18n,
  viewer, link, text_input, vi, iwe) — no logic duplication.

## Workspace Structure

Split the gpui-free modules into a `dkb-core` library crate so that `dk` can
depend on core without pulling in gpui.

```
dkb/                          (workspace root)
  Cargo.toml                  (workspace manifest)
  crates/
    dkb-core/                 (new lib crate)
      Cargo.toml
      src/
        lib.rs
        item.rs
        board.rs
        storage.rs
        config.rs
        i18n/
          mod.rs
          locales.rs
        viewer.rs
        link.rs
        text_input.rs
        vi.rs
        iwe.rs
    dkb/                      (existing gui crate, moved)
      Cargo.toml
      src/
        lib.rs                (re-exports dkb-core + gui modules)
        app.rs
        editor.rs
        theme.rs
        bin/
          dkb.rs
  crates/dk/                  (new bin crate)
    Cargo.toml
    src/
      main.rs
```

- The root `Cargo.toml` becomes a workspace `[workspace]` manifest with
  `members = ["crates/dkb-core", "crates/dkb", "crates/dk"]`.
- `dkb-core` holds every module that has no gpui dependency.
- `dkb` (gui) depends on `dkb-core` + `gpui` + `gpui_platform`. Its `lib.rs`
  re-exports `dkb_core::*` so existing `use dkb::item::...` references in the
  GUI code continue to work, plus declares `app`, `editor`, `theme`.
- `dk` depends on `dkb-core` only.
- File moves use `git mv` to preserve history.

## CLI State

A `cli_state.json` file in the data dir (alongside `board_state.json`)
persists state across invocations:

```json
{
  "last_list": ["uuid1", "uuid2", "uuid3"],
  "current": "uuid2"
}
```

- `dk ls` writes `last_list` with the UUIDs in display order.
- `dk pick N` reads `last_list[N]` and writes that UUID to `current`.
- `dk edit` (no args) reads `current`.
- The GUI may honor `current` in the future but is not required to for this
  feature.

## Selection Model

All commands that operate on items (`edit`, `move`, `delete`) accept a
selection specifier. Four modes:

1. **No argument** — operate on the `current` item from `cli_state.json`.
   Error if no current item is set.
2. **Plain number** (e.g. `3`) — resolve via `last_list[3]` from
   `cli_state.json`. Error if index is out of range or `last_list` is empty.
3. **`category/number`** (e.g. `yesterday/1`, `backlog/5`) — list that
   category fresh from disk and pick the item at that index. Does NOT update
   `last_list`.
4. **Filename or full path** — `uuid.md` or `/path/to/uuid.md`. Resolve
   directly.

`move` accepts a second argument (the destination category) after the
selection.

## Category Aliases

Both short and long (underscored) forms are accepted everywhere a category is
expected:

| Alias | Category |
|-------|----------|
| `b`, `backlog` | Backlog |
| `y`, `yesterday` | Yesterday |
| `t`, `today` | Today |
| `tw`, `thisweek`, `this_week` | ThisWeek |
| `nw`, `nextweek`, `next_week` | NextWeek |
| `d`, `done` | Done |

## Commands

### `dk new` / `dk n`

Create a new item. Default location is backlog unless a category argument is
given.

```
dk n
dk new
dk new yesterday
dk n y
dk n t
dk n tw
dk new thisweek
dk n nw
dk new nextweek
dk n d
dk new done
```

**Flow:**

1. Generate a UUID.
2. Write a temp file containing `# ` (with trailing cursor position intent).
3. Invoke `$VISUAL` or `$EDITOR` on the temp file with `+1:3` argument for
   cursor positioning (best-effort; graceful fallback to plain invocation
   for unknown editors).
4. Wait for the editor to exit.
5. Read the temp file back. Extract the first line as the title (strip `# `
   prefix if present). The body is the full edited content.
6. Construct an `Item` with the generated UUID, the body, current timestamps.
7. Save via `Storage::write_item` to the target location.
8. Set the new item as `current` in `cli_state.json`.
9. Clean up the temp file.

If the editor exits and the file is empty or contains only `#`, abort without
creating an item.

### `dk list` / `dk ls`

With no arguments, lists all active items (yesterday, today, this_week,
next_week). Backlog and done are excluded unless explicitly requested.

```
dk ls
dk ls backlog
dk ls done
dk ls yesterday
```

**Output format:**

- Each row: right-justified index number (column width = digits of max index
  + 1) + space + header/subject text (markdown formatting stripped, via
  `Item::clean_title`).
- The row of the `current` item is prefixed with `* ` instead of its number.
- Text is truncated to fit the terminal width without wrapping (using
  terminal width detection; fallback to 80 columns).
- `last_list` in `cli_state.json` is updated with the UUIDs in display order.
- When listing a specific category, `last_list` is also updated to that
  category's items (so `dk pick N` works after a filtered list).

### `dk pick` / `dk p`

Sets the current marker. Accepts:

- A number from the last `dk ls` output.
- A filename (`uuid.md`).
- A full path.

```
dk p 3
dk pick 3
dk pick long-uuid-here.md
dk pick /some/path/to/filename-uuid.md
```

Resolves the selection to a UUID and writes it to `current` in
`cli_state.json`. Prints a confirmation.

### `dk edit` / `dk ed`

Edits one or more items. Selection uses the standard model (no arg = current;
number, category/number, filename, path all accepted). Multiple selections
can be passed to edit several items sequentially.

```
dk ed
dk edit 3
dk edit 3 5 9
dk edit long-uuid-here.md
dk edit /some/path/to/filename-uuid.md
dk edit backlog/5
dk edit today
```

`dk edit today` — a bare category name (no `/number`) is treated as a
no-op alias; `edit` operates on the `current` item. This is a minor
ambiguity to resolve through testing.

**Flow per item:**

1. Resolve selection to an `Item` and its `Location`.
2. Write a temp file containing the item's body (no frontmatter).
3. Invoke `$VISUAL` or `$EDITOR`.
4. Read back. Update the item's body and `updated_at`.
5. Save via `Storage::write_item` to the same location.
6. Clean up temp file.

### `dk move` / `dk mv`

Moves an item between boards/categories. First argument is the selection;
second is the destination category.

```
dk mv 3 done
dk mv yesterday/1 today
dk mv backlog nextweek
```

Uses `Storage::move_item` (which handles frontmatter timestamp updates and
file relocation). Updates `current` if the moved item was current.

### `dk delete` / `dk rm`

Deletes an item. Selection uses the standard model.

```
dk rm
dk rm 3
dk rm done
dk rm backlog/5
```

Uses `Storage::delete_item`. Clears `current` if the deleted item was
current. Always prompts for confirmation unless `--force`/`-f` is passed
(safety measure for irreversible deletion). To be confirmed through testing.

## Install Symlink

A configuration button in the GUI (separate task, not part of the CLI binary
itself) creates a symlink:

- Target: `~/.local/bin/dk`
- Points to: the `dk` binary embedded in the app bundle
  (`dkb.app/Contents/MacOS/dk`) in production, or the cargo build output in
  development.
- Creates `~/.local/bin` if it does not exist.

The CLI binary itself does not perform installation; that is a GUI-side
concern. This spec covers only the `dk` binary behavior.

## Error Handling

- Missing `current` item when one is needed: print error, exit non-zero.
- Out-of-range list index: print error with the valid range, exit non-zero.
- File not found for path/filename selection: print error, exit non-zero.
- Editor not set (`$VISUAL` and `$EDITOR` both unset): fall back to `vi` on
  macOS (platform default), print a notice.
- Data dir not initialized: run `Storage::init` automatically on first use.

## Testing

- Unit tests in `dkb-core` for the existing modules (already present where
  applicable).
- Unit tests in `dk` for:
  - Category alias parsing.
  - Selection resolution (number, category/number, filename, path).
  - CLI state read/write.
  - List formatting (width truncation, current marker).
- Integration tests using a temp data dir to verify end-to-end flows:
  create → list → pick → edit → move → delete.

## Out of Scope

- GUI changes for the install-symlink button (separate task).
- GUI honoring the `current` marker (future enhancement).
- Non-macOS platform support for the symlink (macOS is the distribution
  target).
