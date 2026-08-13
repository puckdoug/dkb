use crate::item::Item;

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
