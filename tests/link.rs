use dkb::item::{Category, Item};
use dkb::link::{count_recursive_subitems, extract_links};
use dkb::storage::{Location, Storage};
use tempfile::TempDir;

#[test]
fn test_link_extraction() {
    let body = "Parent item\n- [Sub 1](00000000-0000-0000-0000-000000000001.md)\n- [[00000000-0000-0000-0000-000000000002]]";
    let links = extract_links(body);
    assert_eq!(links.len(), 2);
}

#[test]
fn test_recursive_subitem_count() {
    let temp = TempDir::new().unwrap();
    Storage::init(temp.path()).unwrap();

    let child_leaf = Item::new("Leaf child");
    let mut child_middle = Item::new("Middle child");
    child_middle
        .body
        .push_str(&format!("\n- [Leaf]({}.md)", child_leaf.id));

    let mut root = Item::new("Root item");
    root.body
        .push_str(&format!("\n- [Middle]({}.md)", child_middle.id));

    Storage::write_item(
        temp.path(),
        &child_leaf,
        &Location::Active(Category::Today),
    )
    .unwrap();
    Storage::write_item(
        temp.path(),
        &child_middle,
        &Location::Active(Category::Today),
    )
    .unwrap();
    Storage::write_item(temp.path(), &root, &Location::Active(Category::Today)).unwrap();

    let count = count_recursive_subitems(root.id, temp.path());
    assert_eq!(count, 2);
}

#[test]
fn test_subitem_cycle_detection() {
    let temp = TempDir::new().unwrap();
    Storage::init(temp.path()).unwrap();

    let mut item1 = Item::new("Item 1");
    let mut item2 = Item::new("Item 2");

    item1.body.push_str(&format!("\n- [[{}]]", item2.id));
    item2.body.push_str(&format!("\n- [[{}]]", item1.id));

    Storage::write_item(temp.path(), &item1, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(temp.path(), &item2, &Location::Active(Category::Today)).unwrap();

    let count = count_recursive_subitems(item1.id, temp.path());
    assert_eq!(count, 1);
}

#[test]
fn test_iwe_init_workspace() {
    let temp = TempDir::new().unwrap();
    dkb::iwe::init_workspace(temp.path()).unwrap();

    let config_file = temp.path().join(".iwe").join("config.toml");
    assert!(config_file.exists());
    let content = std::fs::read_to_string(config_file).unwrap();
    assert!(content.contains("name = \"dkb\""));
}

