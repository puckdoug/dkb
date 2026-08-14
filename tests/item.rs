#![allow(clippy::pedantic)]

use chrono::Utc;
use dkb::item::{Category, Item, Status};

#[test]
fn test_item_new_sets_title_from_first_line() {
    let item = Item::new("Fix the login bug");
    assert_eq!(item.title(), "Fix the login bug");
    assert_eq!(item.body, "Fix the login bug");
    assert!(item.created_at <= Utc::now());
    assert_eq!(item.created_at, item.updated_at);
    assert!(item.completed_at.is_none());
}

#[test]
fn test_item_new_with_multiline_body() {
    let item = Item::new("Fix the login bug\n\nDetails about the bug here");
    assert_eq!(item.title(), "Fix the login bug");
    assert_eq!(item.body, "Fix the login bug\n\nDetails about the bug here");
}

#[test]
fn test_extract_title_from_body() {
    assert_eq!(Item::extract_title("Hello world"), "Hello world");
    assert_eq!(Item::extract_title("Hello world\nrest"), "Hello world");
    assert_eq!(Item::extract_title("\n\nHello world\nrest"), "Hello world");
    assert_eq!(Item::extract_title(""), "");
    assert_eq!(Item::extract_title("\n\n\n"), "");
}

#[test]
fn test_status_variants() {
    let _backlog = Status::Backlog;
    let _active = Status::Active;
    let _done = Status::Done;
}

#[test]
fn test_category_variants() {
    let _y = Category::Yesterday;
    let _t = Category::Today;
    let _tw = Category::ThisWeek;
    let _nw = Category::NextWeek;
}

#[test]
fn test_serialize_item_with_frontmatter() {
    let mut item = Item::new("My task title");
    item.body = "My task title\n\nSome **markdown** body.".to_string();
    let serialized = item.serialize();
    assert!(serialized.starts_with("---\n"));
    assert!(serialized.contains("created_at:"));
    assert!(serialized.contains("updated_at:"));
    assert!(!serialized.contains("completed_at:"));
    assert!(serialized.contains("---\nMy task title"));
}

#[test]
fn test_parse_frontmatter() {
    let content = "---\ncreated_at: 2026-08-13T10:30:00Z\nupdated_at: 2026-08-13T14:22:00Z\ncompleted_at: null\n---\nFix the login bug\n\nDetails here";
    let (frontmatter, body) = Item::parse_frontmatter(content).unwrap();
    assert_eq!(body, "Fix the login bug\n\nDetails here");
    assert!(frontmatter.completed_at.is_none());
}

#[test]
fn test_parse_frontmatter_with_completed_at() {
    let content = "---\ncreated_at: 2026-08-13T10:30:00Z\nupdated_at: 2026-08-13T14:22:00Z\ncompleted_at: 2026-08-13T16:00:00Z\n---\nDone task";
    let (frontmatter, body) = Item::parse_frontmatter(content).unwrap();
    assert!(frontmatter.completed_at.is_some());
    assert_eq!(body, "Done task");
}

#[test]
fn test_round_trip_serialize_parse() {
    let mut item = Item::new("Round trip test");
    item.body = "Round trip test\n\nBody text".to_string();
    let serialized = item.serialize();
    let (frontmatter, body) = Item::parse_frontmatter(&serialized).unwrap();
    assert_eq!(body, "Round trip test\n\nBody text");
    assert_eq!(frontmatter.created_at, Some(item.created_at));
    assert_eq!(frontmatter.updated_at, Some(item.updated_at));
    assert_eq!(frontmatter.completed_at, item.completed_at);
}

#[test]
fn test_parse_frontmatter_no_frontmatter() {
    let content = "Just a body\nwith text";
    let (frontmatter, body) = Item::parse_frontmatter(content).unwrap();
    assert_eq!(body, "Just a body\nwith text");
    assert!(frontmatter.created_at.is_none());
}

#[test]
fn test_clean_title_formatting() {
    assert_eq!(dkb::item::Item::clean_title("# Heading One"), "Heading One");
    assert_eq!(dkb::item::Item::clean_title("### Subheading"), "Subheading");
    assert_eq!(dkb::item::Item::clean_title("**Bold Title**"), "Bold Title");
    assert_eq!(dkb::item::Item::clean_title("*Italic Title*"), "Italic Title");
    assert_eq!(dkb::item::Item::clean_title("`Code Title`"), "Code Title");
    assert_eq!(dkb::item::Item::clean_title("[Link Title](https://example.com)"), "Link Title");
    assert_eq!(dkb::item::Item::clean_title("~~Strikethrough~~"), "Strikethrough");
    assert_eq!(dkb::item::Item::clean_title("## **Complex** `Title` with [Link](url)"), "Complex Title with Link");
}

