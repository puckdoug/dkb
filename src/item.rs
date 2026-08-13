use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Backlog,
    Active,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Yesterday,
    Today,
    ThisWeek,
    NextWeek,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Item {
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            body: title.to_string(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn title(&self) -> String {
        Self::extract_title(&self.body)
    }

    pub fn extract_title(body: &str) -> String {
        body.lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
            .to_string()
    }

    pub fn serialize(&self) -> String {
        let frontmatter = ItemFrontmatter {
            created_at: Some(self.created_at),
            updated_at: Some(self.updated_at),
            completed_at: self.completed_at,
        };
        let yaml = serde_yaml::to_string(&frontmatter).unwrap_or_default();
        format!(
            "{}\n{}\n{}\n{}",
            FRONTMATTER_DELIMITER,
            yaml.trim_end(),
            FRONTMATTER_DELIMITER,
            self.body
        )
    }

    pub fn parse_frontmatter(content: &str) -> Option<(ItemFrontmatter, String)> {
        let content = content.trim_start_matches('\u{feff}');
        if !content.starts_with(FRONTMATTER_DELIMITER) {
            return Some((ItemFrontmatter::default(), content.to_string()));
        }
        let rest = &content[FRONTMATTER_DELIMITER.len()..];
        let close_marker = format!("\n{}\n", FRONTMATTER_DELIMITER);
        let (yaml_part, body) = match rest.find(&close_marker) {
            Some(idx) => (&rest[..idx], &rest[idx + close_marker.len()..]),
            None => {
                let close_eof = format!("\n{}", FRONTMATTER_DELIMITER);
                let idx = rest.find(&close_eof)?;
                if !rest[idx + close_eof.len()..].is_empty() {
                    return None;
                }
                (&rest[..idx], "")
            }
        };
        let yaml_str = yaml_part.trim();
        let frontmatter: ItemFrontmatter = if yaml_str.is_empty() {
            ItemFrontmatter::default()
        } else {
            serde_yaml::from_str(yaml_str).ok()?
        };
        Some((frontmatter, body.trim_start_matches('\n').to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ItemFrontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

const FRONTMATTER_DELIMITER: &str = "---";
