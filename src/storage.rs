use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::board::Board;
use crate::item::{Category, Item, Status};
use uuid::Uuid;

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

    pub fn write_item(
        data_dir: &Path,
        item: &Item,
        location: &Location,
    ) -> std::io::Result<()> {
        let dir = data_dir.join(location.to_path());
        std::fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("{}.md", item.id));
        std::fs::write(file_path, item.serialize())?;
        Ok(())
    }

    pub fn read_item(
        data_dir: &Path,
        id: &Uuid,
        location: &Location,
    ) -> std::io::Result<Item> {
        let file_path = data_dir
            .join(location.to_path())
            .join(format!("{}.md", id));
        let content = std::fs::read_to_string(file_path)?;
        Self::parse_item_from_content(id, content)
    }

    pub fn move_item(
        data_dir: &Path,
        id: &Uuid,
        from: &Location,
        to: &Location,
    ) -> std::io::Result<Item> {
        let from_path = data_dir
            .join(from.to_path())
            .join(format!("{}.md", id));
        let to_dir = data_dir.join(to.to_path());
        std::fs::create_dir_all(&to_dir)?;
        let to_path = to_dir.join(format!("{}.md", id));

        let content = std::fs::read_to_string(&from_path)?;
        let (frontmatter, body) = Item::parse_frontmatter(&content)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to parse frontmatter"))?;

        let now = Utc::now();
        let completed_at = match to.status() {
            Status::Done => Some(now),
            _ => {
                if from.status() == Status::Done {
                    None
                } else {
                    frontmatter.completed_at
                }
            }
        };

        let updated_item = Item {
            id: *id,
            body,
            created_at: frontmatter.created_at.unwrap_or_default(),
            updated_at: now,
            completed_at,
        };

        std::fs::write(&to_path, updated_item.serialize())?;
        std::fs::remove_file(from_path)?;

        Ok(updated_item)
    }

    pub fn delete_item(
        data_dir: &Path,
        id: &Uuid,
        location: &Location,
    ) -> std::io::Result<()> {
        let path = data_dir
            .join(location.to_path())
            .join(format!("{}.md", id));
        std::fs::remove_file(path)?;
        Ok(())
    }

    pub fn parse_item_from_content(id: &Uuid, content: String) -> std::io::Result<Item> {
        let (frontmatter, body) = Item::parse_frontmatter(&content)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to parse frontmatter"))?;
        Ok(Item {
            id: *id,
            body,
            created_at: frontmatter.created_at.unwrap_or_default(),
            updated_at: frontmatter.updated_at.unwrap_or_default(),
            completed_at: frontmatter.completed_at,
        })
    }

    pub fn load_board(data_dir: &Path) -> std::io::Result<Board> {
        let mut board = Board::default();

        let locations = [
            Location::Backlog,
            Location::Active(Category::Yesterday),
            Location::Active(Category::Today),
            Location::Active(Category::ThisWeek),
            Location::Active(Category::NextWeek),
            Location::Done,
        ];

        for location in locations {
            let dir = data_dir.join(location.to_path());
            if !dir.exists() {
                continue;
            }
            let mut items: Vec<Item> = Vec::new();
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let Ok(id) = Uuid::parse_str(file_stem) else {
                    continue;
                };
                let content = std::fs::read_to_string(&path)?;
                match Self::parse_item_from_content(&id, content) {
                    Ok(item) => items.push(item),
                    Err(_) => continue,
                }
            }
            match location {
                Location::Backlog => board.backlog = items,
                Location::Active(Category::Yesterday) => board.active.yesterday = items,
                Location::Active(Category::Today) => board.active.today = items,
                Location::Active(Category::ThisWeek) => board.active.this_week = items,
                Location::Active(Category::NextWeek) => board.active.next_week = items,
                Location::Done => board.done = items,
            }
        }

        board.done.sort_by(|a, b| {
            b.completed_at
                .unwrap_or_default()
                .cmp(&a.completed_at.unwrap_or_default())
        });

        Ok(board)
    }
}
