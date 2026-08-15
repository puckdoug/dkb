use crate::item::{Category, Item, Status};
use crate::storage::Location;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BoardState {
    pub version: u32,
    pub order: HashMap<String, Vec<Uuid>>,
    #[serde(default)]
    pub last_active_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Default)]
pub struct ActiveColumns {
    pub yesterday: Vec<Item>,
    pub today: Vec<Item>,
    pub this_week: Vec<Item>,
    pub next_week: Vec<Item>,
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    pub backlog: Vec<Item>,
    pub active: ActiveColumns,
    pub done: Vec<Item>,
}

impl Board {
    pub fn set_column_order(&mut self, location: &Location, ordered_ids: Vec<Uuid>) {
        let items_vec = match location {
            Location::Backlog => &mut self.backlog,
            Location::Active(Category::Yesterday) => &mut self.active.yesterday,
            Location::Active(Category::Today) => &mut self.active.today,
            Location::Active(Category::ThisWeek) => &mut self.active.this_week,
            Location::Active(Category::NextWeek) => &mut self.active.next_week,
            Location::Done => &mut self.done,
        };

        let mut map: HashMap<Uuid, Item> = items_vec.drain(..).map(|i| (i.id, i)).collect();
        for id in &ordered_ids {
            if let Some(item) = map.remove(id) {
                items_vec.push(item);
            }
        }
        for (_, remaining_item) in map {
            items_vec.push(remaining_item);
        }
    }

    pub fn to_board_state(&self) -> BoardState {
        let mut order = HashMap::new();
        order.insert(
            Location::Backlog.to_path().to_string_lossy().to_string(),
            self.backlog.iter().map(|i| i.id).collect(),
        );
        order.insert(
            Location::Active(Category::Yesterday).to_path().to_string_lossy().to_string(),
            self.active.yesterday.iter().map(|i| i.id).collect(),
        );
        order.insert(
            Location::Active(Category::Today).to_path().to_string_lossy().to_string(),
            self.active.today.iter().map(|i| i.id).collect(),
        );
        order.insert(
            Location::Active(Category::ThisWeek).to_path().to_string_lossy().to_string(),
            self.active.this_week.iter().map(|i| i.id).collect(),
        );
        order.insert(
            Location::Active(Category::NextWeek).to_path().to_string_lossy().to_string(),
            self.active.next_week.iter().map(|i| i.id).collect(),
        );
        order.insert(
            Location::Done.to_path().to_string_lossy().to_string(),
            self.done.iter().map(|i| i.id).collect(),
        );

        BoardState {
            version: 1,
            order,
            last_active_date: None,
        }
    }

    pub fn find_item(&self, id: &Uuid) -> Option<&Item> {
        self.backlog
            .iter()
            .chain(self.active.yesterday.iter())
            .chain(self.active.today.iter())
            .chain(self.active.this_week.iter())
            .chain(self.active.next_week.iter())
            .chain(self.done.iter())
            .find(|item| item.id == *id)
    }

    pub fn find_item_mut(&mut self, id: &Uuid) -> Option<&mut Item> {
        self.backlog
            .iter_mut()
            .chain(self.active.yesterday.iter_mut())
            .chain(self.active.today.iter_mut())
            .chain(self.active.this_week.iter_mut())
            .chain(self.active.next_week.iter_mut())
            .chain(self.done.iter_mut())
            .find(|item| item.id == *id)
    }

    pub fn find_item_location(&self, id: &Uuid) -> Option<Location> {
        if self.backlog.iter().any(|i| i.id == *id) {
            Some(Location::Backlog)
        } else if self.active.yesterday.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::Yesterday))
        } else if self.active.today.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::Today))
        } else if self.active.this_week.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::ThisWeek))
        } else if self.active.next_week.iter().any(|i| i.id == *id) {
            Some(Location::Active(Category::NextWeek))
        } else if self.done.iter().any(|i| i.id == *id) {
            Some(Location::Done)
        } else {
            None
        }
    }

    pub fn can_move(&self, id: &Uuid, to: &Location) -> bool {
        let Some(from) = self.find_item_location(id) else {
            return false;
        };
        if from.status() == Status::Backlog && to.status() == Status::Done {
            return false;
        }
        if from == *to {
            return false;
        }
        true
    }

    pub fn move_item(
        &mut self,
        id: &Uuid,
        to: &Location,
    ) -> Option<(Location, Location)> {
        if !self.can_move(id, to) {
            return None;
        }
        let from = self.find_item_location(id)?;
        let now = Utc::now();

        let mut item = self.remove_item(id, &from)?;
        item.updated_at = now;
        item.completed_at = match to.status() {
            Status::Done => Some(now),
            _ => {
                if from.status() == Status::Done {
                    None
                } else {
                    item.completed_at
                }
            }
        };

        self.insert_item(item, to);
        Some((from, *to))
    }

    pub fn remove_item(&mut self, id: &Uuid, location: &Location) -> Option<Item> {
        let vec = match location {
            Location::Backlog => &mut self.backlog,
            Location::Active(Category::Yesterday) => &mut self.active.yesterday,
            Location::Active(Category::Today) => &mut self.active.today,
            Location::Active(Category::ThisWeek) => &mut self.active.this_week,
            Location::Active(Category::NextWeek) => &mut self.active.next_week,
            Location::Done => &mut self.done,
        };
        let pos = vec.iter().position(|i| i.id == *id)?;
        Some(vec.remove(pos))
    }

    pub fn insert_item(&mut self, item: Item, location: &Location) {
        match location {
            Location::Backlog => self.backlog.push(item),
            Location::Active(Category::Yesterday) => self.active.yesterday.push(item),
            Location::Active(Category::Today) => self.active.today.push(item),
            Location::Active(Category::ThisWeek) => self.active.this_week.push(item),
            Location::Active(Category::NextWeek) => self.active.next_week.push(item),
            Location::Done => self.done.push(item),
        }
    }
}
