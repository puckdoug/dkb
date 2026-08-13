use chrono::{DateTime, Utc};
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
}
