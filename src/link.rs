use std::collections::HashSet;
use std::ops::Range;
use std::path::Path;
use uuid::Uuid;

use crate::item::Item;
use crate::storage::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSpan {
    pub range: Range<usize>,
    pub target_id: Uuid,
    pub text: String,
}

#[must_use]
pub fn format_markdown_link(text: &str, id: Uuid) -> String {
    format!("[{text}]({id}.md)")
}

#[must_use]
pub fn extract_link_spans(content: &str) -> Vec<LinkSpan> {
    let mut spans = Vec::new();
    let len = content.len();
    let mut i = 0;

    while i < len {
        if content[i..].starts_with("[[") {
            if let Some(close_idx) = content[i + 2..].find("]]") {
                let close_pos = i + 2 + close_idx;
                let end_pos = close_pos + 2;
                let inner = &content[i + 2..close_pos];
                let (target_part, text_part) = match inner.split_once('|') {
                    Some((target, text)) => (target.trim(), text.trim()),
                    None => (inner.trim(), inner.trim()),
                };
                let target_cleaned = target_part.strip_suffix(".md").unwrap_or(target_part);
                if let Ok(id) = Uuid::parse_str(target_cleaned) {
                    spans.push(LinkSpan {
                        range: i..end_pos,
                        target_id: id,
                        text: text_part.to_string(),
                    });
                    i = end_pos;
                    continue;
                }
            }
        } else if content[i..].starts_with('[')
            && let Some(close_bracket_idx) = content[i + 1..].find(']')
        {
            let close_bracket_pos = i + 1 + close_bracket_idx;
            if content[close_bracket_pos..].starts_with("](") {
                let open_paren_pos = close_bracket_pos + 1;
                if let Some(close_paren_idx) = content[open_paren_pos + 1..].find(')') {
                    let close_paren_pos = open_paren_pos + 1 + close_paren_idx;
                    let end_pos = close_paren_pos + 1;
                    let text = &content[i + 1..close_bracket_pos];
                    let url = content[open_paren_pos + 1..close_paren_pos].trim();
                    let target_cleaned = url.strip_suffix(".md").unwrap_or(url);
                    if let Ok(id) = Uuid::parse_str(target_cleaned) {
                        spans.push(LinkSpan {
                            range: i..end_pos,
                            target_id: id,
                            text: text.to_string(),
                        });
                        i = end_pos;
                        continue;
                    }
                }
            }
        }

        if let Some(ch) = content[i..].chars().next() {
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    spans
}

#[must_use]
pub fn find_link_at_offset(content: &str, offset: usize) -> Option<LinkSpan> {
    extract_link_spans(content)
        .into_iter()
        .find(|span| offset >= span.range.start && offset <= span.range.end)
}

#[must_use]
pub fn extract_links(body: &str) -> Vec<Uuid> {
    extract_link_spans(body)
        .into_iter()
        .map(|span| span.target_id)
        .collect()
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
