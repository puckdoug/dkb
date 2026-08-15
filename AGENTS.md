# AGENTS.md

Rust project `dkb`. Minimal scaffold, no commits yet.

- `Cargo.toml` uses `edition = "2024"` — requires recent nightly (or 1.85+ stable) toolchain.
- Commands: `cargo build`, `cargo run`, `cargo test`.
- Single binary, entrypoint `src/main.rs`.
- use ripgrep (`rg`) in preference to grep
- Always confirm `cargo clippy` warnings are addressed
- Always review `cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic` and address items which make sense or mark with #[allow(clippy::lint_name)]
- when implementation is done move specs/plans into done/{specs,plans}
- when building multiple items, create a separate branch, commit after each step (e.g. test created/red, code implemetned/green)
- ensure UserGuide.md is updated whenever updates to the application are made which affect the user interface
