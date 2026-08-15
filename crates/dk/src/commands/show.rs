use std::io::{IsTerminal, Write};
use std::path::Path;

use dkb_core::cli_state::CliState;
use dkb_core::storage::Storage;

use crate::select::{parse_selection, resolve_selection};

#[allow(clippy::missing_errors_doc)]
pub fn run_show(data_dir: &Path, args: &[String]) -> std::io::Result<()> {
    let board = Storage::load_board(data_dir)?;
    let state = CliState::load(data_dir);

    let id = if args.is_empty() {
        state
            .current
            .ok_or_else(|| std::io::Error::other("no current item set"))?
    } else {
        let sel = parse_selection(&args[0]);
        resolve_selection(&sel, &board, &state, data_dir).map_err(std::io::Error::other)?
    };

    let item = board
        .find_item(&id)
        .ok_or_else(|| std::io::Error::other(format!("item {id} not found")))?
        .clone();

    let output = item.body.as_bytes();

    if std::io::stdout().is_terminal() {
        let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
        let mut parts = pager.split_whitespace();
        let binary = parts.next().unwrap_or("less");
        let mut cmd = std::process::Command::new(binary);
        for extra in parts {
            cmd.arg(extra);
        }
        cmd.stdin(std::process::Stdio::piped());
        let mut child = cmd.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(output)?;
        }
        let status = child.wait()?;
        if !status.success() {
            return Err(std::io::Error::other(format!(
                "pager exited with code {:?}",
                status.code()
            )));
        }
    } else {
        std::io::stdout().write_all(output)?;
    }

    Ok(())
}
