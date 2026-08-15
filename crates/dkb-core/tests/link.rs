#![allow(clippy::pedantic)]

use dkb_core::item::{Category, Item};
use dkb_core::link::{
    count_recursive_subitems, extract_link_spans, extract_links, find_link_at_offset,
    format_markdown_link, LinkSpan,
};
use dkb_core::storage::{Location, Storage};
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_link_extraction() {
    let body = "Parent item\n- [Sub 1](00000000-0000-0000-0000-000000000001.md)\n- [[00000000-0000-0000-0000-000000000002]]";
    let links = extract_links(body);
    assert_eq!(links.len(), 2);
}

#[test]
fn test_recursive_subitem_count() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path().join("data");
    Storage::init(&data_dir).unwrap();

    let child_leaf = Item::new("Leaf child");
    let mut child_middle = Item::new("Middle child");
    child_middle
        .body
        .push_str(&format!("\n- [Leaf]({}.md)", child_leaf.id));

    let mut root = Item::new("Root item");
    root.body
        .push_str(&format!("\n- [Middle]({}.md)", child_middle.id));

    Storage::write_item(
        &data_dir,
        &child_leaf,
        &Location::Active(Category::Today),
    )
    .unwrap();
    Storage::write_item(
        &data_dir,
        &child_middle,
        &Location::Active(Category::Today),
    )
    .unwrap();
    Storage::write_item(&data_dir, &root, &Location::Active(Category::Today)).unwrap();

    let count = count_recursive_subitems(root.id, &data_dir);
    assert_eq!(count, 2);
}

#[test]
fn test_subitem_cycle_detection() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path().join("data");
    Storage::init(&data_dir).unwrap();

    let mut item1 = Item::new("Item 1");
    let mut item2 = Item::new("Item 2");

    item1.body.push_str(&format!("\n- [[{}]]", item2.id));
    item2.body.push_str(&format!("\n- [[{}]]", item1.id));

    Storage::write_item(&data_dir, &item1, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(&data_dir, &item2, &Location::Active(Category::Today)).unwrap();

    let count = count_recursive_subitems(item1.id, &data_dir);
    assert_eq!(count, 1);
}

#[test]
fn test_iwe_init_workspace() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path().join("data");
    dkb_core::iwe::init_workspace(&data_dir).unwrap();

    let config_file = temp.path().join(".iwe").join("config.toml");
    assert!(config_file.exists());
    let content = std::fs::read_to_string(config_file).unwrap();
    assert!(content.contains("name = \"dkb\""));
}

#[test]
fn test_find_link_at_offset_markdown() {
    let id = Uuid::new_v4();
    let body = format!("Check this item: [My Sub Task]({id}.md) for details");
    let link_start = body.find('[').unwrap();
    let link_end = body.find(')').unwrap() + 1;

    // Offset before link
    assert!(find_link_at_offset(&body, 2).is_none());

    // Offset exactly at start of link
    let span_start = find_link_at_offset(&body, link_start).expect("should find link at start");
    assert_eq!(span_start.target_id, id);
    assert_eq!(span_start.text, "My Sub Task");
    assert_eq!(span_start.range, link_start..link_end);

    // Offset within link text
    let span_mid = find_link_at_offset(&body, link_start + 5).expect("should find link in text");
    assert_eq!(span_mid.target_id, id);
    assert_eq!(span_mid.text, "My Sub Task");
    assert_eq!(span_mid.range, link_start..link_end);

    // Offset within URL target
    let span_url = find_link_at_offset(&body, link_end - 5).expect("should find link in url");
    assert_eq!(span_url.target_id, id);
    assert_eq!(span_url.text, "My Sub Task");
    assert_eq!(span_url.range, link_start..link_end);

    // Offset exactly at end of link (offset <= end)
    let span_end = find_link_at_offset(&body, link_end).expect("should find link at end boundary");
    assert_eq!(span_end.target_id, id);
    assert_eq!(span_end.text, "My Sub Task");
    assert_eq!(span_end.range, link_start..link_end);

    // Offset after link
    assert!(find_link_at_offset(&body, link_end + 1).is_none());
}

#[test]
fn test_find_link_at_offset_wikilink() {
    let id = Uuid::new_v4();
    let body = format!("Prefix [[{id}.md]] suffix");
    let link_start = body.find("[[").unwrap();
    let link_end = body.find("]]").unwrap() + 2;

    // Offset inside wikilink
    let span = find_link_at_offset(&body, link_start + 4).expect("should find wikilink");
    assert_eq!(span.target_id, id);
    assert_eq!(span.range, link_start..link_end);

    // Wikilink without .md
    let body_no_ext = format!("Note: [[{id}]] done");
    let link_start2 = body_no_ext.find("[[").unwrap();
    let link_end2 = body_no_ext.find("]]").unwrap() + 2;
    let span2 = find_link_at_offset(&body_no_ext, link_start2 + 3).expect("should find wikilink without .md");
    assert_eq!(span2.target_id, id);
    assert_eq!(span2.range, link_start2..link_end2);
}

#[test]
fn test_find_link_at_offset_multiple_links() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let body = format!("First [Item One]({id1}.md) and second [[{id2}]] end");

    let offset1 = body.find("Item One").unwrap();
    let span1 = find_link_at_offset(&body, offset1).expect("should find first link");
    assert_eq!(span1.target_id, id1);
    assert_eq!(span1.text, "Item One");

    let offset2 = body.find(&id2.to_string()).unwrap();
    let span2 = find_link_at_offset(&body, offset2).expect("should find second link");
    assert_eq!(span2.target_id, id2);

    let offset_between = body.find(" and ").unwrap() + 2;
    assert!(find_link_at_offset(&body, offset_between).is_none());
}

#[test]
fn test_format_markdown_link() {
    let id = Uuid::new_v4();
    let formatted = format_markdown_link("Design Document", id);
    assert_eq!(formatted, format!("[Design Document]({id}.md)"));
}

#[test]
fn test_extract_link_spans() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let content = format!("Intro [Link One]({id1}.md) and [[{id2}|Custom Text]] outro");

    let spans = extract_link_spans(&content);
    assert_eq!(spans.len(), 2);

    let expected_span1 = LinkSpan {
        range: content.find('[').unwrap()..(content.find(')').unwrap() + 1),
        target_id: id1,
        text: "Link One".to_string(),
    };
    assert_eq!(spans[0], expected_span1);
    assert_eq!(spans[1].target_id, id2);
    assert_eq!(spans[1].text, "Custom Text");
}

#[test]
fn test_find_link_at_offset_invalid_links() {
    let body = "This is [not a valid link](not-a-uuid.md) and [[invalid-uuid]]";
    assert!(find_link_at_offset(body, 15).is_none());
    assert!(find_link_at_offset(body, 50).is_none());
    assert!(find_link_at_offset(body, 999).is_none());
}

