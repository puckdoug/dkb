use crate::text_input::TextInputState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViMode {
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, Clone)]
pub struct ViState {
    pub mode: ViMode,
    pub pending_op: Option<char>,
    pub yank_buffer: Option<String>,
    pub visual_anchor: Option<usize>,
}

impl Default for ViState {
    fn default() -> Self {
        Self::new()
    }
}

impl ViState {
    pub fn new() -> Self {
        Self {
            mode: ViMode::Normal,
            pending_op: None,
            yank_buffer: None,
            visual_anchor: None,
        }
    }

    pub fn handle_key(&mut self, key: &str, state: &mut TextInputState) -> bool {
        match self.mode {
            ViMode::Normal => self.handle_normal_key(key, state),
            ViMode::Insert => self.handle_insert_key(key, state),
            ViMode::Visual => self.handle_visual_key(key, state),
        }
    }

    fn handle_insert_key(&mut self, key: &str, state: &mut TextInputState) -> bool {
        match key {
            "escape" | "Esc" | "Escape" | "\x1b" => {
                self.mode = ViMode::Normal;
                if state.cursor_offset() > 0 && state.selected_range().is_empty() {
                    let cur = state.cursor_offset();
                    let line_start = state.find_line_start(cur);
                    if cur > line_start {
                        state.move_left();
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn handle_normal_key(&mut self, key: &str, state: &mut TextInputState) -> bool {
        match key {
            "escape" | "Esc" | "Escape" | "\x1b" => {
                self.pending_op = None;
                self.visual_anchor = None;
                true
            }
            "i" => {
                self.mode = ViMode::Insert;
                true
            }
            "a" => {
                self.mode = ViMode::Insert;
                if state.cursor_offset() < state.content().len() {
                    state.move_right();
                }
                true
            }
            "v" => {
                self.mode = ViMode::Visual;
                self.visual_anchor = Some(state.cursor_offset());
                true
            }
            "h" => {
                state.move_left();
                true
            }
            "l" => {
                state.move_right();
                true
            }
            "j" => {
                let cur = state.cursor_offset();
                let target = find_line_down(state.content(), cur);
                state.move_to(target);
                true
            }
            "k" => {
                let cur = state.cursor_offset();
                let target = find_line_up(state.content(), cur);
                state.move_to(target);
                true
            }
            "0" => {
                let cur = state.cursor_offset();
                let target = find_line_start(state.content(), cur);
                state.move_to(target);
                true
            }
            "$" => {
                let cur = state.cursor_offset();
                let target = find_line_end(state.content(), cur);
                state.move_to(target);
                true
            }
            "w" => {
                let cur = state.cursor_offset();
                let target = find_next_word_start(state.content(), cur);
                state.move_to(target);
                true
            }
            "b" => {
                let cur = state.cursor_offset();
                let target = find_prev_word_start(state.content(), cur);
                state.move_to(target);
                true
            }
            "x" => {
                state.delete();
                true
            }
            "u" => {
                state.undo();
                true
            }
            "Ctrl-r" | "ctrl-r" | "C-r" => {
                state.redo();
                true
            }
            "d" => {
                if self.pending_op == Some('d') {
                    // dd -> delete current line
                    self.pending_op = None;
                    let cur = state.cursor_offset();
                    let start = find_line_start(state.content(), cur);
                    let mut end = find_line_end(state.content(), cur);
                    if end < state.content().len() && state.content().as_bytes()[end] == b'\n' {
                        end += 1;
                    }
                    self.yank_buffer = Some(state.content()[start..end].to_string());
                    state.replace_range(start..end, "");
                    state.move_to(start.min(state.content().len()));
                    true
                } else {
                    self.pending_op = Some('d');
                    true
                }
            }
            "y" => {
                if self.pending_op == Some('y') {
                    // yy -> yank line
                    self.pending_op = None;
                    let cur = state.cursor_offset();
                    let start = find_line_start(state.content(), cur);
                    let mut end = find_line_end(state.content(), cur);
                    if end < state.content().len() && state.content().as_bytes()[end] == b'\n' {
                        end += 1;
                    }
                    self.yank_buffer = Some(state.content()[start..end].to_string());
                    true
                } else {
                    self.pending_op = Some('y');
                    true
                }
            }
            "p" => {
                if let Some(buf) = &self.yank_buffer.clone() {
                    state.insert(buf);
                }
                true
            }
            "o" => {
                let cur = state.cursor_offset();
                let end = find_line_end(state.content(), cur);
                state.move_to(end);
                state.insert("\n");
                self.mode = ViMode::Insert;
                true
            }
            "O" => {
                let cur = state.cursor_offset();
                let start = find_line_start(state.content(), cur);
                state.move_to(start);
                state.insert("\n");
                state.move_to(start);
                self.mode = ViMode::Insert;
                true
            }
            _ => {
                self.pending_op = None;
                false
            }
        }
    }

    fn handle_visual_key(&mut self, key: &str, state: &mut TextInputState) -> bool {
        match key {
            "escape" | "Esc" | "Escape" | "\x1b" | "v" => {
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                let cur = state.cursor_offset();
                state.move_to(cur);
                true
            }
            "h" => {
                state.select_left();
                true
            }
            "l" => {
                state.select_right();
                true
            }
            "j" => {
                let cur = state.cursor_offset();
                let target = find_line_down(state.content(), cur);
                state.select_to(target);
                true
            }
            "k" => {
                let cur = state.cursor_offset();
                let target = find_line_up(state.content(), cur);
                state.select_to(target);
                true
            }
            "w" => {
                let cur = state.cursor_offset();
                let target = find_next_word_start(state.content(), cur);
                state.select_to(target);
                true
            }
            "b" => {
                let cur = state.cursor_offset();
                let target = find_prev_word_start(state.content(), cur);
                state.select_to(target);
                true
            }
            "0" => {
                let cur = state.cursor_offset();
                let target = find_line_start(state.content(), cur);
                state.select_to(target);
                true
            }
            "$" => {
                let cur = state.cursor_offset();
                let target = find_line_end(state.content(), cur);
                state.select_to(target);
                true
            }
            "d" | "x" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    let yanked = state.content()[range.clone()].to_string();
                    self.yank_buffer = Some(yanked);
                    state.replace_range(range.clone(), "");
                    state.move_to(range.start);
                }
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                true
            }
            "y" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    self.yank_buffer = Some(state.content()[range.clone()].to_string());
                }
                let cur = state.cursor_offset();
                state.move_to(cur);
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                true
            }
            _ => false,
        }
    }
}

fn find_line_start(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(0)
}

fn find_line_end(content: &str, offset: usize) -> usize {
    let offset = offset.min(content.len());
    content[offset..]
        .find('\n')
        .map(|idx| offset + idx)
        .unwrap_or(content.len())
}

fn find_line_down(content: &str, offset: usize) -> usize {
    let line_start = find_line_start(content, offset);
    let col = offset - line_start;
    let line_end = find_line_end(content, offset);
    if line_end >= content.len() {
        return offset;
    }
    let next_line_start = line_end + 1;
    let next_line_end = find_line_end(content, next_line_start);
    let next_line_len = next_line_end - next_line_start;
    next_line_start + col.min(next_line_len)
}

fn find_line_up(content: &str, offset: usize) -> usize {
    let line_start = find_line_start(content, offset);
    if line_start == 0 {
        return offset;
    }
    let col = offset - line_start;
    let prev_line_end = line_start - 1;
    let prev_line_start = find_line_start(content, prev_line_end);
    let prev_line_len = prev_line_end - prev_line_start;
    prev_line_start + col.min(prev_line_len)
}

fn find_next_word_start(content: &str, offset: usize) -> usize {
    let mut chars = content.char_indices().filter(|(i, _)| *i >= offset);
    if let Some((_, first_ch)) = chars.next() {
        let first_is_ws = first_ch.is_whitespace();
        let first_is_word = first_ch.is_alphanumeric() || first_ch == '_';

        let mut past_first_group = false;
        for (idx, ch) in content.char_indices().filter(|(i, _)| *i > offset) {
            if !past_first_group {
                if first_is_ws {
                    if !ch.is_whitespace() {
                        return idx;
                    }
                } else if first_is_word {
                    if !(ch.is_alphanumeric() || ch == '_') {
                        if !ch.is_whitespace() {
                            return idx;
                        }
                        past_first_group = true;
                    }
                } else {
                    if ch.is_alphanumeric() || ch == '_' {
                        return idx;
                    }
                    if ch.is_whitespace() {
                        past_first_group = true;
                    }
                }
            } else if !ch.is_whitespace() {
                return idx;
            }
        }
    }
    content.len()
}

fn find_prev_word_start(content: &str, offset: usize) -> usize {
    if offset == 0 {
        return 0;
    }
    let chars: Vec<(usize, char)> = content.char_indices().filter(|(i, _)| *i < offset).collect();
    if chars.is_empty() {
        return 0;
    }

    let iter = chars.iter().rev();
    let mut non_ws_found = false;
    let mut target_type = None;
    let mut last_idx = 0;

    for &(idx, ch) in iter {
        if !non_ws_found {
            if ch.is_whitespace() {
                continue;
            }
            non_ws_found = true;
            let is_word = ch.is_alphanumeric() || ch == '_';
            target_type = Some(is_word);
            last_idx = idx;
        } else {
            let is_word = ch.is_alphanumeric() || ch == '_';
            if Some(is_word) == target_type && !ch.is_whitespace() {
                last_idx = idx;
            } else {
                break;
            }
        }
    }

    last_idx
}
