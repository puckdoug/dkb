use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

use crate::item::Item;
use crate::storage::Location;

pub fn extract_links(body: &str) -> Vec<Uuid> {
    let mut links = Vec::new();
    let chars: Vec<char> = body.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Check for markdown link: `](` + uuid + (`.md` optionally) + `)`
        if chars[i] == ']' && i + 1 < len && chars[i + 1] == '(' {
            let start = i + 2;
            if let Some(close_paren) = body[start..].find(')') {
                let target = &body[start..start + close_paren].trim();
                let target_cleaned = target.strip_suffix(".md").unwrap_or(target);
                if let Ok(id) = Uuid::parse_str(target_cleaned) {
                    links.push(id);
                }
            }
        }
        // Check for wikilink: `[[` + uuid + (`.md` optionally) + `]]`
        else if chars[i] == '[' && i + 1 < len && chars[i + 1] == '[' {
            let start = i + 2;
            if let Some(close_wiki) = body[start..].find("]]") {
                let target = &body[start..start + close_wiki].trim();
                let target_cleaned = target.strip_suffix(".md").unwrap_or(target);
                if let Ok(id) = Uuid::parse_str(target_cleaned) {
                    links.push(id);
                }
            }
        }
        i += 1;
    }

    links
}

fn find_and_read_item(data_dir: &Path, id: &Uuid) -> Option<Item> {
    use crate::item::Category;
    let locations = [
        Location::Backlog,
        Location::Active(Category::Yesterday),
        Location::Active(Category::Today),
        Location::Active(Category::ThisWeek),
        Location::Active(Category::NextWeek),
        Location::Done,
    ];

    for loc in locations {
        let file_path = data_dir.join(loc.to_path()).join(format!("{}.md", id));
        if let Ok(content) = std::fs::read_to_string(file_path)
            && let Ok(item) = crate::storage::Storage::parse_item_from_content(id, content)
        {
            return Some(item);
        }
    }

    None
}

pub fn count_recursive_subitems(root_id: Uuid, data_dir: &Path) -> usize {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    visited.insert(root_id);

    if let Some(root_item) = find_and_read_item(data_dir, &root_id) {
        for child_id in extract_links(&root_item.body) {
            if visited.insert(child_id) {
                stack.push(child_id);
            }
        }
    }

    while let Some(current_id) = stack.pop() {
        if let Some(item) = find_and_read_item(data_dir, &current_id) {
            for child_id in extract_links(&item.body) {
                if visited.insert(child_id) {
                    stack.push(child_id);
                }
            }
        }
    }

    // Number of reachable subitems excluding root itself
    visited.len().saturating_sub(1)
}
