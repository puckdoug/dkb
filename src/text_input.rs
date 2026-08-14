use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
struct UndoEntry {
    content: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
}

pub struct TextInputState {
    content: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
}

impl TextInputState {
    pub fn new(initial: &str) -> Self {
        Self {
            content: initial.to_string(),
            selected_range: 0..0,
            selection_reversed: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn selected_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    pub fn move_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
    }

    pub fn move_right(&mut self) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            self.move_to(next);
        } else {
            self.move_to(self.selected_range.end);
        }
    }

    pub fn move_left(&mut self) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            self.move_to(prev);
        } else {
            self.move_to(self.selected_range.start);
        }
    }

    pub fn find_line_start(&self, offset: usize) -> usize {
        let offset = offset.min(self.content.len());
        self.content[..offset]
            .rfind('\n')
            .map(|idx| idx + 1)
            .unwrap_or(0)
    }

    pub fn find_line_end(&self, offset: usize) -> usize {
        let offset = offset.min(self.content.len());
        self.content[offset..]
            .find('\n')
            .map(|idx| offset + idx)
            .unwrap_or(self.content.len())
    }

    pub fn find_line_up(&self, offset: usize) -> usize {
        let line_start = self.find_line_start(offset);
        if line_start == 0 {
            return 0;
        }
        let col = offset - line_start;
        let prev_line_end = line_start - 1;
        let prev_line_start = self.find_line_start(prev_line_end);
        let prev_line_len = prev_line_end - prev_line_start;
        prev_line_start + col.min(prev_line_len)
    }

    pub fn find_line_down(&self, offset: usize) -> usize {
        let line_start = self.find_line_start(offset);
        let col = offset - line_start;
        let line_end = self.find_line_end(offset);
        if line_end >= self.content.len() {
            return self.content.len();
        }
        let next_line_start = line_end + 1;
        let next_line_end = self.find_line_end(next_line_start);
        let next_line_len = next_line_end - next_line_start;
        next_line_start + col.min(next_line_len)
    }

    pub fn move_up(&mut self) {
        let cur = self.cursor_offset();
        let target = self.find_line_up(cur);
        self.move_to(target);
    }

    pub fn move_down(&mut self) {
        let cur = self.cursor_offset();
        let target = self.find_line_down(cur);
        self.move_to(target);
    }

    pub fn select_up(&mut self) {
        let cur = self.cursor_offset();
        let target = self.find_line_up(cur);
        self.select_to(target);
    }

    pub fn select_down(&mut self) {
        let cur = self.cursor_offset();
        let target = self.find_line_down(cur);
        self.select_to(target);
    }

    pub fn move_to_home(&mut self) {
        self.move_to(0);
    }

    pub fn move_to_end(&mut self) {
        self.move_to(self.content.len());
    }

    pub fn select_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
    }

    pub fn select_right(&mut self) {
        let next = self.next_boundary(self.cursor_offset());
        self.select_to(next);
    }

    pub fn select_left(&mut self) {
        let prev = self.previous_boundary(self.cursor_offset());
        self.select_to(prev);
    }

    pub fn select_all(&mut self) {
        self.selected_range = 0..self.content.len();
        self.selection_reversed = false;
    }

    fn save_undo(&mut self) {
        self.undo_stack.push(UndoEntry {
            content: self.content.clone(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        });
        self.redo_stack.clear();
    }

    pub fn insert(&mut self, text: &str) {
        self.save_undo();
        let range = self.selected_range.clone();
        self.content = self.content[..range.start].to_owned() + text + &self.content[range.end..];
        let new_pos = range.start + text.len();
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }

    pub fn backspace(&mut self) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if prev == self.cursor_offset() {
                return;
            }
            self.save_undo();
            self.select_to(prev);
        } else {
            self.save_undo();
        }
        let range = self.selected_range.clone();
        self.content = self.content[..range.start].to_owned() + &self.content[range.end..];
        let new_pos = range.start;
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }

    pub fn delete(&mut self) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if next == self.cursor_offset() {
                return;
            }
            self.save_undo();
            self.select_to(next);
        } else {
            self.save_undo();
        }
        let range = self.selected_range.clone();
        self.content = self.content[..range.start].to_owned() + &self.content[range.end..];
        let new_pos = range.start;
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }

    pub fn replace_range(&mut self, range: Range<usize>, text: &str) {
        self.save_undo();
        self.content = self.content[..range.start].to_owned() + text + &self.content[range.end..];
        let new_pos = range.start + text.len();
        self.selected_range = new_pos..new_pos;
        self.selection_reversed = false;
    }

    pub fn undo(&mut self) {
        if let Some(entry) = self.undo_stack.pop() {
            self.redo_stack.push(UndoEntry {
                content: self.content.clone(),
                selected_range: self.selected_range.clone(),
                selection_reversed: self.selection_reversed,
            });
            self.content = entry.content;
            self.selected_range = entry.selected_range;
            self.selection_reversed = entry.selection_reversed;
        }
    }

    pub fn redo(&mut self) {
        if let Some(entry) = self.redo_stack.pop() {
            self.undo_stack.push(UndoEntry {
                content: self.content.clone(),
                selected_range: self.selected_range.clone(),
                selection_reversed: self.selection_reversed,
            });
            self.content = entry.content;
            self.selected_range = entry.selected_range;
            self.selection_reversed = entry.selection_reversed;
        }
    }

    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    pub fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    pub fn word_start(&self, offset: usize) -> usize {
        for (idx, segment) in self.content.split_word_bound_indices() {
            let end = idx + segment.len();
            if offset >= idx && offset < end {
                return idx;
            }
        }
        if let Some((idx, _)) = self.content.split_word_bound_indices().next_back() {
            return idx;
        }
        0
    }

    pub fn word_end(&self, offset: usize) -> usize {
        for (idx, segment) in self.content.split_word_bound_indices() {
            let end = idx + segment.len();
            if offset >= idx && offset < end {
                return end;
            }
        }
        self.content.len()
    }

    pub fn select_word_at(&mut self, offset: usize) {
        let start = self.word_start(offset);
        let end = self.word_end(offset);
        self.selected_range = start..end;
        self.selection_reversed = false;
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }
}
