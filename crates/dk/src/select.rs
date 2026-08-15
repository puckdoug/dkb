use std::path::{Path, PathBuf};

use dkb_core::board::Board;
use dkb_core::cli_state::CliState;
use dkb_core::item::{Category, Item};
use dkb_core::storage::Location;
use uuid::Uuid;

use crate::category::parse_category;

pub enum Selection {
    Current,
    Index(usize),
    CategoryIndex(Location, usize),
    File(Uuid),
    Path(PathBuf),
}

#[must_use]
pub fn parse_selection(arg: &str) -> Selection {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return Selection::Current;
    }
    if let Some(idx) = trimmed.find('/')
        && let Ok(n) = trimmed[idx + 1..].parse::<usize>()
        && let Some(loc) = parse_category(&trimmed[..idx])
    {
        return Selection::CategoryIndex(loc, n);
    }
    if let Ok(n) = trimmed.parse::<usize>()
        && trimmed.chars().all(|c| c.is_ascii_digit())
    {
        return Selection::Index(n);
    }
    let stripped = trimmed.strip_suffix(".md").unwrap_or(trimmed);
    if let Ok(id) = Uuid::parse_str(stripped) {
        return Selection::File(id);
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() || path.exists() {
        return Selection::Path(path);
    }
    Selection::Path(path)
}

#[allow(clippy::missing_errors_doc)]
pub fn resolve_selection(
    sel: &Selection,
    board: &Board,
    state: &CliState,
    _data_dir: &Path,
) -> Result<Uuid, String> {
    match sel {
        Selection::Current => state.current.ok_or_else(|| "no current item set".to_string()),
        Selection::Index(n) => state.resolve_index(*n).ok_or_else(|| {
            format!("index {n} out of range (last list had {} items)", state.last_list.len())
        }),
        Selection::CategoryIndex(loc, n) => {
            let ids: Vec<Uuid> = items_for_location(board, *loc).into_iter().map(|i| i.id).collect();
            ids.get(*n).copied().ok_or_else(|| {
                format!("index {n} out of range for {} ({})", loc_display(*loc), ids.len())
            })
        }
        Selection::File(id) => {
            if board.find_item(id).is_some() {
                Ok(*id)
            } else {
                Err(format!("item {id} not found on board"))
            }
        }
        Selection::Path(p) => {
            let stem = p.file_stem().and_then(|s| s.to_str()).ok_or_else(|| "invalid path".to_string())?;
            let id = Uuid::parse_str(stem).map_err(|e| format!("invalid uuid in path: {e}"))?;
            if board.find_item(&id).is_some() {
                Ok(id)
            } else {
                Err(format!("item {id} not found on board"))
            }
        }
    }
}

fn items_for_location(board: &Board, loc: Location) -> Vec<&Item> {
    match loc {
        Location::Backlog => board.backlog.iter().collect(),
        Location::Active(Category::Yesterday) => board.active.yesterday.iter().collect(),
        Location::Active(Category::Today) => board.active.today.iter().collect(),
        Location::Active(Category::ThisWeek) => board.active.this_week.iter().collect(),
        Location::Active(Category::NextWeek) => board.active.next_week.iter().collect(),
        Location::Done => board.done.iter().collect(),
    }
}

fn loc_display(loc: Location) -> &'static str {
    match loc {
        Location::Backlog => "backlog",
        Location::Active(Category::Yesterday) => "yesterday",
        Location::Active(Category::Today) => "today",
        Location::Active(Category::ThisWeek) => "this_week",
        Location::Active(Category::NextWeek) => "next_week",
        Location::Done => "done",
    }
}
