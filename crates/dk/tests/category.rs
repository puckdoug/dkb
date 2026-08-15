#![allow(clippy::pedantic)]

use dkb_core::item::Category;
use dkb_core::storage::Location;
use dk::category::parse_category;

#[test]
fn test_short_aliases() {
    assert_eq!(parse_category("b"), Some(Location::Backlog));
    assert_eq!(parse_category("y"), Some(Location::Active(Category::Yesterday)));
    assert_eq!(parse_category("t"), Some(Location::Active(Category::Today)));
    assert_eq!(parse_category("tw"), Some(Location::Active(Category::ThisWeek)));
    assert_eq!(parse_category("nw"), Some(Location::Active(Category::NextWeek)));
    assert_eq!(parse_category("d"), Some(Location::Done));
}

#[test]
fn test_long_aliases() {
    assert_eq!(parse_category("backlog"), Some(Location::Backlog));
    assert_eq!(parse_category("yesterday"), Some(Location::Active(Category::Yesterday)));
    assert_eq!(parse_category("today"), Some(Location::Active(Category::Today)));
    assert_eq!(parse_category("thisweek"), Some(Location::Active(Category::ThisWeek)));
    assert_eq!(parse_category("nextweek"), Some(Location::Active(Category::NextWeek)));
    assert_eq!(parse_category("done"), Some(Location::Done));
}

#[test]
fn test_underscored_forms() {
    assert_eq!(parse_category("this_week"), Some(Location::Active(Category::ThisWeek)));
    assert_eq!(parse_category("next_week"), Some(Location::Active(Category::NextWeek)));
}

#[test]
fn test_case_insensitive() {
    assert_eq!(parse_category("TODAY"), Some(Location::Active(Category::Today)));
    assert_eq!(parse_category("Done"), Some(Location::Done));
}

#[test]
fn test_unknown() {
    assert_eq!(parse_category("tomorrow"), None);
    assert_eq!(parse_category(""), None);
}
