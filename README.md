# Daily Kanban (`dkb`)

A native macOS Kanban application built with the GPUI framework from Zed. `dkb` organizes tasks across time horizons (Yesterday, Today, This Week, Next Week, Backlog, Done) stored as plain Markdown files on disk with full compatibility with the [iwe](https://iwe.md) markdown knowledge graph.

---

## Features

- **Time Horizon Kanban**: Active board with `Yesterday`, `Today`, `This Week`, and `Next Week` columns, alongside dedicated `Backlog` and `Done` tabs.
- **Plain Markdown Persistence**: Every item is a Markdown file with YAML frontmatter timestamps, stored directly in the filesystem.
- **iwe Integration & Sub-Items**: Recursive sub-item hierarchy (`↪ <count>`) using standard Markdown links (`[Title](uuid.md)` or `[[uuid]]`), auto-initializing `.iwe` workspace configuration.
- **Clean Titles**: Automatically strips Markdown headers (`#`), bold (`**`), italics, and links from the first line for display on kanban cards.
- **Persistent Absolute Sorting**: Card drag-and-drop within and between columns with order persisted in `board_state.json`.
- **Integrated Markdown Editor**:
  - Optional line numbers gutter.
  - Vi-mode modal editing (Normal, Insert, Visual) supporting motions (`h`, `j`, `k`, `l`, `w`, `b`, `0`, `$`), operations (`x`, `dd`, `yy`, `p`, `u`, `Ctrl-r`), and status indicator.
  - Modal editor with tear-off window support.
- **Theming**: Light, Dark, and Follow System modes.
- **Settings Screen**: Pinned `Settings ⌘,` tab and application menu item for configuring storage paths, Vi mode, Line numbers, and Themes.

---

## Requirements

- **macOS**: 13.0+ (Ventura, Sonoma, Sequoia or newer)
- **Rust**: 1.85+ stable or recent nightly toolchain (`edition = "2024"`)
- **Xcode Command Line Tools**: `xcode-select --install`

---

## Development

### Run in Debug Mode
```bash
cargo run
```

### Run Tests & Linting
```bash
cargo test
cargo clippy --all-targets
```

---

## Building and Packaging for Local Installation

To build a standalone macOS application bundle (`Daily Kanban.app`):

1. **Run the bundle script**:
   ```bash
   ./scripts/bundle_macos.sh
   ```

2. **Output Location**:
   ```
   target/release/bundle/Daily Kanban.app
   ```

3. **Install on Mac**:
   Drag or copy `Daily Kanban.app` to your `/Applications` directory:
   ```bash
   cp -R "target/release/bundle/Daily Kanban.app" /Applications/
   ```

---

## Mac App Store Distribution

For full instructions on entitlements, provisioning profiles, code signing, building the installer package (`.pkg`), and submitting via `altool` / Transporter, see **[AppStore.md](AppStore.md)**.

---

## Keyboard Shortcuts

| Shortcut | Action | Context |
| :--- | :--- | :--- |
| `⌘ N` | Create new item | Kanban |
| `⇧ ⌘ N` | Create sub-item under selected card | Kanban |
| `⌘ ,` | Open Settings tab | Global |
| `⌘ ]` / `⌘ [` | Next / Previous column (focuses top item) | Kanban |
| `↑` / `↓` | Move selection up / down in column | Kanban |
| `←` / `→` | Move selection to nearest item in left / right column | Kanban |
| `h` / `j` / `k` / `l` | Spatial navigation (Vi mode enabled) | Kanban |
| `Enter` | Open selected item in editor | Kanban |
| `⌘ →` | Drill down into sub-items | Kanban |
| `⌘ ←` | Drill up breadcrumb | Kanban |
| `⌘ 1` - `⌘ 4` | Move selected item to Yesterday / Today / This Week / Next Week | Kanban |
| `⌘ B` | Move selected item to Backlog | Kanban |
| `⌘ D` | Toggle Done / Reopen item | Kanban |
| `Delete` / `Backspace` | Delete selected item | Kanban |
| `⌘ S` | Save editor content | Editor |
| `Esc` | Return to Normal mode (Vi mode) | Editor |
| `⌘ W` | Close window / editor | Global |
| `⌘ Q` | Quit application | Global |

---

## License

This project is dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or [LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

This product also includes software developed by Zed Industries, Inc. (GPUI framework), licensed under the Apache License, Version 2.0 ([NOTICE](NOTICE)).
