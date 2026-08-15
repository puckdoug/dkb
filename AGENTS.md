# AGENTS.md

Rust project `dkb`. The project is a Cargo workspace with three crates:

- `crates/dkb-core` — shared library (`dkb_core`) containing the data model, storage, board, config, and CLI state logic.
- `crates/dkb` — the GUI binary (macOS GPUI Kanban app), entrypoint `crates/dkb/src/main.rs`.
- `crates/dk` — the CLI binary, entrypoint `crates/dk/src/main.rs`.

- `Cargo.toml` (workspace) uses `edition = "2024"` — requires recent nightly (or 1.85+ stable) toolchain.
- Commands: `cargo build` (builds both `dkb` and `dk` binaries), `cargo run -p dkb` (GUI) / `cargo run -p dk` (CLI), `cargo test` (runs tests across all three crates).
- use ripgrep (`rg`) in preference to grep
- Always confirm `cargo clippy` warnings are addressed
- Always review `cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic` and address items which make sense or mark with #[allow(clippy::lint_name)]
- when implementation is done move specs/plans into done/{specs,plans}
- when building multiple items, create a separate branch, commit after each step (e.g. test created/red, code implemetned/green)
- ensure UserGuide.md is updated whenever updates to the application are made which affect the user interface
- Always maintain the checklist in the plan file as work progresses.
