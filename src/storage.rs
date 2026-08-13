use std::path::{Path, PathBuf};

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
}
