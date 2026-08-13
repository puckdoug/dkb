use std::path::{Path, PathBuf};

use crate::item::{Category, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Backlog,
    Active(Category),
    Done,
}

impl Location {
    pub fn to_path(&self) -> PathBuf {
        match self {
            Location::Backlog => PathBuf::from("backlog"),
            Location::Active(cat) => PathBuf::from("active").join(match cat {
                Category::Yesterday => "yesterday",
                Category::Today => "today",
                Category::ThisWeek => "this_week",
                Category::NextWeek => "next_week",
            }),
            Location::Done => PathBuf::from("done"),
        }
    }

    pub fn from_path(path: &str) -> Self {
        let path = path.trim_end_matches('/');
        match path {
            "backlog" => Location::Backlog,
            "done" => Location::Done,
            "active/yesterday" => Location::Active(Category::Yesterday),
            "active/today" => Location::Active(Category::Today),
            "active/this_week" => Location::Active(Category::ThisWeek),
            "active/next_week" => Location::Active(Category::NextWeek),
            _ => Location::Backlog,
        }
    }

    pub fn status(&self) -> Status {
        match self {
            Location::Backlog => Status::Backlog,
            Location::Active(_) => Status::Active,
            Location::Done => Status::Done,
        }
    }

    pub fn category(&self) -> Option<Category> {
        match self {
            Location::Active(cat) => Some(*cat),
            _ => None,
        }
    }
}

pub struct Storage;

impl Storage {
    pub fn init(data_dir: &Path) -> std::io::Result<()> {
        let dirs = [
            "backlog",
            "active/yesterday",
            "active/today",
            "active/this_week",
            "active/next_week",
            "done",
        ];
        for dir in &dirs {
            std::fs::create_dir_all(data_dir.join(dir))?;
        }
        Ok(())
    }
}
