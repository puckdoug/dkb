use dkb_core::item::Category;
use dkb_core::storage::Location;

#[must_use]
pub fn parse_category(s: &str) -> Option<Location> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "b" | "backlog" => Some(Location::Backlog),
        "y" | "yesterday" => Some(Location::Active(Category::Yesterday)),
        "t" | "today" => Some(Location::Active(Category::Today)),
        "tw" | "thisweek" | "this_week" => Some(Location::Active(Category::ThisWeek)),
        "nw" | "nextweek" | "next_week" => Some(Location::Active(Category::NextWeek)),
        "d" | "done" => Some(Location::Done),
        _ => None,
    }
}
