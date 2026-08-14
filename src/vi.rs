use std::ops::Range;

use crate::text_input::TextInputState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualKind {
    Character,
    Line,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViMode {
    Normal,
    Insert,
    Visual(VisualKind),
    Command,
    Search(SearchDirection),
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExCommand {
    Write,
    Quit {
        force: bool,
    },
    WriteQuit,
    GotoLine(usize),
    Substitute {
        pattern: String,
        replacement: String,
        global: bool,
        ignore_case: bool,
    },
    DeleteLine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViActionResult {
    None,
    Handled,
    ExecuteEx(ExCommand),
    Save,
    Close { force: bool },
    SaveAndClose,
}

#[derive(Debug, Clone)]
pub struct ViState {
    pub mode: ViMode,
    pub count: Option<usize>,
    pub pending_op: Option<char>,
    pub yank_buffer: Option<String>,
    pub is_linewise_yank: bool,
    pub visual_anchor: Option<usize>,
    pub command_buffer: String,
    pub search_buffer: String,
    pub last_search: Option<(String, SearchDirection)>,
    pub find_char_state: Option<(char, bool, bool)>, // (target_char, is_till, is_forward)
    pub replace_pending: bool,

    // Internal helper state
    pub pending_find: Option<(bool, bool)>, // (is_till, is_forward)
    pub pending_g: bool,
    pub pending_text_object: Option<(char, char)>, // (op, 'i' or 'a')
    pub op_count: Option<usize>,
}

impl Default for ViState {
    fn default() -> Self {
        Self::new()
    }
}

impl ViState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: ViMode::Normal,
            count: None,
            pending_op: None,
            yank_buffer: None,
            is_linewise_yank: false,
            visual_anchor: None,
            command_buffer: String::new(),
            search_buffer: String::new(),
            last_search: None,
            find_char_state: None,
            replace_pending: false,
            pending_find: None,
            pending_g: false,
            pending_text_object: None,
            op_count: None,
        }
    }

    fn take_count(&mut self) -> usize {
        let op_c = self.op_count.take().unwrap_or(1);
        let c = self.count.take().unwrap_or(1);
        op_c * c
    }

    pub fn handle_key(&mut self, key: &str, state: &mut TextInputState) -> ViActionResult {
        match self.mode {
            ViMode::Normal => self.handle_normal_key(key, state),
            ViMode::Insert => self.handle_insert_key(key, state),
            ViMode::Visual(kind) => self.handle_visual_key(key, kind, state),
            ViMode::Command => self.handle_command_key(key, state),
            ViMode::Search(dir) => self.handle_search_key(key, dir, state),
            ViMode::Replace => self.handle_replace_key(key, state),
        }
    }

    fn is_escape(key: &str) -> bool {
        matches!(key, "escape" | "Esc" | "Escape" | "\x1b")
    }

    fn is_enter(key: &str) -> bool {
        matches!(key, "enter" | "Enter" | "\n" | "\r")
    }

    fn is_backspace(key: &str) -> bool {
        matches!(key, "backspace" | "Backspace" | "\x08" | "\x7f")
    }

    fn handle_insert_key(&mut self, key: &str, state: &mut TextInputState) -> ViActionResult {
        if Self::is_escape(key) {
            self.mode = ViMode::Normal;
            if state.cursor_offset() > 0 && state.selected_range().is_empty() {
                let cur = state.cursor_offset();
                let line_start = find_line_start(state.content(), cur);
                if cur > line_start {
                    state.move_left();
                }
            }
            ViActionResult::Handled
        } else {
            ViActionResult::None
        }
    }

    fn handle_replace_key(&mut self, key: &str, state: &mut TextInputState) -> ViActionResult {
        if Self::is_escape(key) {
            self.mode = ViMode::Normal;
            return ViActionResult::Handled;
        }

        if let Some(ch) = key.chars().next()
            && !key.starts_with("ctrl")
            && !key.starts_with("Ctrl")
            && !key.starts_with("alt")
        {
            let cur = state.cursor_offset();
            let content = state.content();
            if cur < content.len() {
                let char_len = content[cur..]
                    .chars()
                    .next()
                    .map_or(1, char::len_utf8);
                state.replace_range(cur..(cur + char_len), &ch.to_string());
                state.move_to(cur + ch.len_utf8());
            }
            return ViActionResult::Handled;
        }

        ViActionResult::None
    }

    fn handle_command_key(&mut self, key: &str, state: &mut TextInputState) -> ViActionResult {
        if Self::is_escape(key) {
            self.mode = ViMode::Normal;
            self.command_buffer.clear();
            ViActionResult::Handled
        } else if Self::is_enter(key) {
            self.mode = ViMode::Normal;
            let cmd_str = std::mem::take(&mut self.command_buffer);
            if let Some(cmd) = Self::parse_ex_command(&cmd_str) {
                self.execute_ex_command(cmd, state)
            } else {
                ViActionResult::Handled
            }
        } else if Self::is_backspace(key) {
            if self.command_buffer.pop().is_none() {
                self.mode = ViMode::Normal;
            }
            ViActionResult::Handled
        } else {
            self.command_buffer.push_str(key);
            ViActionResult::Handled
        }
    }

    fn handle_search_key(
        &mut self,
        key: &str,
        dir: SearchDirection,
        state: &mut TextInputState,
    ) -> ViActionResult {
        if Self::is_escape(key) {
            self.mode = ViMode::Normal;
            self.search_buffer.clear();
            ViActionResult::Handled
        } else if Self::is_enter(key) {
            self.mode = ViMode::Normal;
            let query = std::mem::take(&mut self.search_buffer);
            if !query.is_empty() {
                self.last_search = Some((query, dir));
            }
            if let Some((pattern, d)) = self.last_search.clone() {
                self.jump_to_search_match(&pattern, d, state, 1);
            }
            ViActionResult::Handled
        } else if Self::is_backspace(key) {
            if self.search_buffer.pop().is_none() {
                self.mode = ViMode::Normal;
            }
            ViActionResult::Handled
        } else {
            self.search_buffer.push_str(key);
            ViActionResult::Handled
        }
    }

    fn handle_visual_key(
        &mut self,
        key: &str,
        kind: VisualKind,
        state: &mut TextInputState,
    ) -> ViActionResult {
        if Self::is_escape(key)
            || (key == "v" && kind == VisualKind::Character)
            || (key == "V" && kind == VisualKind::Line)
        {
            self.mode = ViMode::Normal;
            self.visual_anchor = None;
            let cur = state.cursor_offset();
            state.move_to(cur);
            return ViActionResult::Handled;
        }

        if key == "v" && kind == VisualKind::Line {
            self.mode = ViMode::Visual(VisualKind::Character);
            self.update_visual_selection(state, VisualKind::Character);
            return ViActionResult::Handled;
        }

        if key == "V" && kind == VisualKind::Character {
            self.mode = ViMode::Visual(VisualKind::Line);
            self.update_visual_selection(state, VisualKind::Line);
            return ViActionResult::Handled;
        }

        match key {
            "d" | "x" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    self.yank_buffer = Some(state.content()[range.clone()].to_string());
                    self.is_linewise_yank = matches!(kind, VisualKind::Line);
                    state.replace_range(range.clone(), "");
                    state.move_to(range.start.min(state.content().len()));
                }
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                ViActionResult::Handled
            }
            "c" | "s" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    self.yank_buffer = Some(state.content()[range.clone()].to_string());
                    self.is_linewise_yank = matches!(kind, VisualKind::Line);
                    state.replace_range(range.clone(), "");
                    state.move_to(range.start.min(state.content().len()));
                }
                self.mode = ViMode::Insert;
                self.visual_anchor = None;
                ViActionResult::Handled
            }
            "y" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    self.yank_buffer = Some(state.content()[range.clone()].to_string());
                    self.is_linewise_yank = matches!(kind, VisualKind::Line);
                }
                state.move_to(range.start.min(state.content().len()));
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                ViActionResult::Handled
            }
            "~" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    let toggled: String = state.content()[range.clone()]
                        .chars()
                        .map(toggle_char_case)
                        .collect();
                    state.replace_range(range.clone(), &toggled);
                    state.move_to(range.start);
                }
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                ViActionResult::Handled
            }
            ">" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    indent_range(state, range);
                }
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                ViActionResult::Handled
            }
            "<" => {
                let range = state.selected_range();
                if !range.is_empty() {
                    outdent_range(state, range);
                }
                self.mode = ViMode::Normal;
                self.visual_anchor = None;
                ViActionResult::Handled
            }
            _ => {
                if self.handle_motion_key(key, state, true) {
                    self.update_visual_selection(state, kind);
                    ViActionResult::Handled
                } else {
                    ViActionResult::None
                }
            }
        }
    }

    fn update_visual_selection(&self, state: &mut TextInputState, kind: VisualKind) {
        let anchor = self.visual_anchor.unwrap_or(state.cursor_offset());
        let cur = state.cursor_offset();
        let content = state.content();

        match kind {
            VisualKind::Character => {
                let start = anchor.min(cur);
                let end = anchor.max(cur);
                if cur < anchor {
                    state.move_to(end);
                    state.select_to(start);
                } else {
                    state.move_to(start);
                    state.select_to(end);
                }
            }
            VisualKind::Line => {
                let min_pos = anchor.min(cur);
                let max_pos = anchor.max(cur);
                let start = find_line_start(content, min_pos);
                let mut end = find_line_end(content, max_pos);
                if end < content.len() && content.as_bytes()[end] == b'\n' {
                    end += 1;
                }
                let same_line = find_line_start(content, anchor) == find_line_start(content, cur);
                if cur < anchor {
                    state.move_to(end);
                    state.select_to(start);
                } else {
                    state.move_to(start);
                    state.select_to(end);
                    if same_line {
                        state.move_to(end);
                        state.select_to(start);
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn handle_normal_key(&mut self, key: &str, state: &mut TextInputState) -> ViActionResult {
        if Self::is_escape(key) {
            self.pending_op = None;
            self.visual_anchor = None;
            self.count = None;
            self.op_count = None;
            self.pending_find = None;
            self.pending_g = false;
            self.pending_text_object = None;
            self.replace_pending = false;
            return ViActionResult::Handled;
        }

        // Replace pending (r<char>)
        if self.replace_pending {
            self.replace_pending = false;
            if let Some(ch) = key.chars().next()
                && !key.starts_with("ctrl")
                && !key.starts_with("Ctrl")
                && !key.starts_with("alt")
            {
                let count = self.take_count();
                let cur = state.cursor_offset();
                let content = state.content();
                if cur < content.len() {
                    let available = content[cur..].chars().take_while(|&c| c != '\n').count();
                    let n = count.min(available).max(1);
                    let replace_bytes: usize =
                        content[cur..].chars().take(n).map(char::len_utf8).sum();
                    let repl: String = std::iter::repeat_n(ch, n).collect();
                    state.replace_range(cur..(cur + replace_bytes), &repl);
                    state.move_to(cur);
                }
            }
            return ViActionResult::Handled;
        }

        // Pending find char (f, F, t, T)
        if let Some((is_till, is_forward)) = self.pending_find.take() {
            if let Some(ch) = key.chars().next()
                && !key.starts_with("ctrl")
                && !key.starts_with("Ctrl")
                && !key.starts_with("alt")
            {
                self.find_char_state = Some((ch, is_till, is_forward));
                let count = self.take_count();
                let cur = state.cursor_offset();
                if let Some(target) = find_char_inline(
                    state.content(),
                    cur,
                    ch,
                    is_till,
                    is_forward,
                    count,
                ) {
                    if let Some(op) = self.pending_op.take() {
                        self.apply_operator_range(
                            op,
                            cur,
                            target,
                            false,
                            state,
                        );
                    } else {
                        state.move_to(target);
                    }
                }
            }
            return ViActionResult::Handled;
        }

        // Pending text object (e.g. diw, ca", ya()
        if let Some((op, prefix)) = self.pending_text_object.take() {
            let inner = prefix == 'i';
            let cur = state.cursor_offset();
            let content = state.content();
            let range = match key {
                "w" => {
                    if inner {
                        find_inner_word_range(content, cur)
                    } else {
                        find_around_word_range(content, cur)
                    }
                }
                "\"" => find_quote_range(content, cur, '"', inner),
                "'" => find_quote_range(content, cur, '\'', inner),
                "`" => find_quote_range(content, cur, '`', inner),
                "(" | ")" => find_bracket_range(content, cur, '(', ')', inner),
                "[" | "]" => find_bracket_range(content, cur, '[', ']', inner),
                "{" | "}" => find_bracket_range(content, cur, '{', '}', inner),
                "p" => find_paragraph_range(content, cur, inner),
                _ => None,
            };

            if let Some(r) = range {
                self.apply_operator_exact_range(op, r, false, state);
            }
            return ViActionResult::Handled;
        }

        // Digits for counts
        if (key.len() == 1 && key.chars().all(|c| c.is_ascii_digit()))
            && (key != "0" || self.count.is_some())
        {
            let digit = key.parse::<usize>().unwrap();
            self.count = Some(self.count.unwrap_or(0) * 10 + digit);
            return ViActionResult::Handled;
        }

        // Pending 'g' prefix (gg, ge, gE)
        if self.pending_g {
            self.pending_g = false;
            match key {
                "g" => {
                    let cur = state.cursor_offset();
                    let target = if let Some(line_num) = self.count.take() {
                        find_line_by_number(state.content(), line_num)
                    } else {
                        0
                    };
                    if let Some(op) = self.pending_op.take() {
                        self.apply_operator_range(op, cur, target, true, state);
                    } else {
                        state.move_to(target);
                    }
                    return ViActionResult::Handled;
                }
                "e" => {
                    let count = self.take_count();
                    let cur = state.cursor_offset();
                    let target = find_word_end_backward(state.content(), cur, false, count);
                    if let Some(op) = self.pending_op.take() {
                        self.apply_operator_range(op, cur, target, false, state);
                    } else {
                        state.move_to(target);
                    }
                    return ViActionResult::Handled;
                }
                "E" => {
                    let count = self.take_count();
                    let cur = state.cursor_offset();
                    let target = find_word_end_backward(state.content(), cur, true, count);
                    if let Some(op) = self.pending_op.take() {
                        self.apply_operator_range(op, cur, target, false, state);
                    } else {
                        state.move_to(target);
                    }
                    return ViActionResult::Handled;
                }
                _ => {}
            }
        }

        // Operator pending (d, c, y, >, <)
        if let Some(op) = self.pending_op {
            if key == "i" || key == "a" {
                self.pending_text_object = Some((op, key.chars().next().unwrap()));
                self.pending_op = None;
                return ViActionResult::Handled;
            }

            if (op == 'd' && key == "d")
                || (op == 'c' && key == "c")
                || (op == 'y' && key == "y")
                || (op == '>' && key == ">")
                || (op == '<' && key == "<")
            {
                self.pending_op = None;
                let count = self.take_count();
                self.apply_linewise_operator(op, count, state);
                return ViActionResult::Handled;
            }

            if self.handle_motion_key(key, state, false) {
                return ViActionResult::Handled;
            }
        }

        // Normal mode key dispatch
        match key {
            "i" => {
                self.mode = ViMode::Insert;
                ViActionResult::Handled
            }
            "I" => {
                let cur = state.cursor_offset();
                let start = find_line_start(state.content(), cur);
                let end = find_line_end(state.content(), cur);
                let non_ws = find_first_non_whitespace(state.content(), start, end);
                state.move_to(non_ws);
                self.mode = ViMode::Insert;
                ViActionResult::Handled
            }
            "a" => {
                self.mode = ViMode::Insert;
                if state.cursor_offset() < state.content().len() {
                    state.move_right();
                }
                ViActionResult::Handled
            }
            "A" => {
                let cur = state.cursor_offset();
                let end = find_line_end(state.content(), cur);
                state.move_to(end);
                self.mode = ViMode::Insert;
                ViActionResult::Handled
            }
            "o" => {
                let cur = state.cursor_offset();
                let end = find_line_end(state.content(), cur);
                state.move_to(end);
                state.insert("\n");
                self.mode = ViMode::Insert;
                ViActionResult::Handled
            }
            "O" => {
                let cur = state.cursor_offset();
                let start = find_line_start(state.content(), cur);
                state.move_to(start);
                state.insert("\n");
                state.move_to(start);
                self.mode = ViMode::Insert;
                ViActionResult::Handled
            }
            "v" => {
                self.mode = ViMode::Visual(VisualKind::Character);
                self.visual_anchor = Some(state.cursor_offset());
                ViActionResult::Handled
            }
            "V" => {
                self.mode = ViMode::Visual(VisualKind::Line);
                self.visual_anchor = Some(state.cursor_offset());
                self.update_visual_selection(state, VisualKind::Line);
                ViActionResult::Handled
            }
            ":" => {
                self.mode = ViMode::Command;
                self.command_buffer.clear();
                ViActionResult::Handled
            }
            "/" => {
                self.mode = ViMode::Search(SearchDirection::Forward);
                self.search_buffer.clear();
                ViActionResult::Handled
            }
            "?" => {
                self.mode = ViMode::Search(SearchDirection::Backward);
                self.search_buffer.clear();
                ViActionResult::Handled
            }
            "d" => {
                self.pending_op = Some('d');
                if self.count.is_some() {
                    self.op_count = self.count.take();
                }
                ViActionResult::Handled
            }
            "c" => {
                self.pending_op = Some('c');
                if self.count.is_some() {
                    self.op_count = self.count.take();
                }
                ViActionResult::Handled
            }
            "y" => {
                self.pending_op = Some('y');
                if self.count.is_some() {
                    self.op_count = self.count.take();
                }
                ViActionResult::Handled
            }
            ">" => {
                self.pending_op = Some('>');
                if self.count.is_some() {
                    self.op_count = self.count.take();
                }
                ViActionResult::Handled
            }
            "<" => {
                self.pending_op = Some('<');
                if self.count.is_some() {
                    self.op_count = self.count.take();
                }
                ViActionResult::Handled
            }
            "D" => {
                let cur = state.cursor_offset();
                let end = find_line_end(state.content(), cur);
                self.yank_buffer = Some(state.content()[cur..end].to_string());
                self.is_linewise_yank = false;
                state.replace_range(cur..end, "");
                state.move_to(cur.min(state.content().len()));
                ViActionResult::Handled
            }
            "C" => {
                let cur = state.cursor_offset();
                let end = find_line_end(state.content(), cur);
                self.yank_buffer = Some(state.content()[cur..end].to_string());
                self.is_linewise_yank = false;
                state.replace_range(cur..end, "");
                state.move_to(cur.min(state.content().len()));
                self.mode = ViMode::Insert;
                ViActionResult::Handled
            }
            "s" => {
                let count = self.take_count();
                let cur = state.cursor_offset();
                let content = state.content();
                if cur < content.len() {
                    let delete_bytes: usize =
                        content[cur..].chars().take(count).map(char::len_utf8).sum();
                    self.yank_buffer = Some(content[cur..(cur + delete_bytes)].to_string());
                    self.is_linewise_yank = false;
                    state.replace_range(cur..(cur + delete_bytes), "");
                    state.move_to(cur);
                }
                self.mode = ViMode::Insert;
                ViActionResult::Handled
            }
            "S" => {
                let count = self.take_count();
                self.apply_linewise_operator('c', count, state);
                ViActionResult::Handled
            }
            "Y" => {
                let cur = state.cursor_offset();
                let end = find_line_end(state.content(), cur);
                self.yank_buffer = Some(state.content()[cur..end].to_string());
                self.is_linewise_yank = false;
                ViActionResult::Handled
            }
            "x" => {
                let count = self.take_count();
                let cur = state.cursor_offset();
                let content = state.content();
                if cur < content.len() {
                    let available = content[cur..].chars().take_while(|&c| c != '\n').count();
                    let n = count.min(available).max(1);
                    let delete_bytes: usize =
                        content[cur..].chars().take(n).map(char::len_utf8).sum();
                    self.yank_buffer = Some(content[cur..(cur + delete_bytes)].to_string());
                    self.is_linewise_yank = false;
                    state.replace_range(cur..(cur + delete_bytes), "");
                    state.move_to(cur.min(state.content().len()));
                }
                ViActionResult::Handled
            }
            "X" => {
                let count = self.take_count();
                let cur = state.cursor_offset();
                let content = state.content();
                let line_start = find_line_start(content, cur);
                if cur > line_start {
                    let available = content[line_start..cur].chars().count();
                    let n = count.min(available).max(1);
                    let delete_bytes: usize = content[line_start..cur]
                        .chars()
                        .rev()
                        .take(n)
                        .map(char::len_utf8)
                        .sum();
                    let start_pos = cur - delete_bytes;
                    self.yank_buffer = Some(content[start_pos..cur].to_string());
                    self.is_linewise_yank = false;
                    state.replace_range(start_pos..cur, "");
                    state.move_to(start_pos);
                }
                ViActionResult::Handled
            }
            "r" => {
                self.replace_pending = true;
                ViActionResult::Handled
            }
            "~" => {
                let count = self.take_count();
                let cur = state.cursor_offset();
                let content = state.content();
                if cur < content.len() {
                    let chars: Vec<char> = content[cur..].chars().take(count).collect();
                    let toggled: String = chars.iter().copied().map(toggle_char_case).collect();
                    let byte_len: usize = chars.iter().map(|c| c.len_utf8()).sum();
                    state.replace_range(cur..(cur + byte_len), &toggled);
                    state.move_to((cur + byte_len).min(state.content().len()));
                }
                ViActionResult::Handled
            }
            "J" => {
                let count = self.take_count().max(1);
                for _ in 0..count {
                    let cur = state.cursor_offset();
                    let content = state.content();
                    let line_end = find_line_end(content, cur);
                    if line_end < content.len() {
                        let next_line_start = line_end + 1;
                        let next_non_ws = content[next_line_start..]
                            .find(|c: char| !c.is_whitespace() || c == '\n')
                            .unwrap_or(0);
                        let delete_end = next_line_start + next_non_ws;
                        state.replace_range(line_end..delete_end, " ");
                        state.move_to(line_end);
                    }
                }
                ViActionResult::Handled
            }
            "u" => {
                state.undo();
                ViActionResult::Handled
            }
            "Ctrl-r" | "ctrl-r" | "C-r" => {
                state.redo();
                ViActionResult::Handled
            }
            "p" => {
                let count = self.take_count();
                if let Some(buf) = &self.yank_buffer.clone() {
                    if self.is_linewise_yank {
                        let cur = state.cursor_offset();
                        let content = state.content();
                        let line_end = find_line_end(content, cur);
                        let insert_pos = if line_end < content.len() {
                            line_end + 1
                        } else {
                            state.move_to(content.len());
                            state.insert("\n");
                            state.content().len()
                        };
                        let repeated: String = std::iter::repeat_n(buf.as_str(), count).collect();
                        state.replace_range(insert_pos..insert_pos, &repeated);
                        let new_end = find_line_end(state.content(), insert_pos);
                        let non_ws = find_first_non_whitespace(state.content(), insert_pos, new_end);
                        state.move_to(non_ws);
                    } else {
                        let cur = state.cursor_offset();
                        let insert_pos = if state.content().is_empty() {
                            0
                        } else {
                            (cur + 1).min(state.content().len())
                        };
                        let repeated: String = std::iter::repeat_n(buf.as_str(), count).collect();
                        state.replace_range(insert_pos..insert_pos, &repeated);
                        state.move_to(insert_pos + repeated.len() - 1);
                    }
                }
                ViActionResult::Handled
            }
            "P" => {
                let count = self.take_count();
                if let Some(buf) = &self.yank_buffer.clone() {
                    if self.is_linewise_yank {
                        let cur = state.cursor_offset();
                        let line_start = find_line_start(state.content(), cur);
                        let repeated: String = std::iter::repeat_n(buf.as_str(), count).collect();
                        state.replace_range(line_start..line_start, &repeated);
                        let new_end = find_line_end(state.content(), line_start);
                        let non_ws = find_first_non_whitespace(state.content(), line_start, new_end);
                        state.move_to(non_ws);
                    } else {
                        let cur = state.cursor_offset();
                        let repeated: String = std::iter::repeat_n(buf.as_str(), count).collect();
                        state.replace_range(cur..cur, &repeated);
                        state.move_to(cur + repeated.len() - 1);
                    }
                }
                ViActionResult::Handled
            }
            "g" => {
                self.pending_g = true;
                ViActionResult::Handled
            }
            "f" => {
                self.pending_find = Some((false, true));
                ViActionResult::Handled
            }
            "F" => {
                self.pending_find = Some((false, false));
                ViActionResult::Handled
            }
            "t" => {
                self.pending_find = Some((true, true));
                ViActionResult::Handled
            }
            "T" => {
                self.pending_find = Some((true, false));
                ViActionResult::Handled
            }
            ";" => {
                if let Some((ch, is_till, is_forward)) = self.find_char_state {
                    let count = self.take_count();
                    let cur = state.cursor_offset();
                    if let Some(target) = find_char_inline(
                        state.content(),
                        cur,
                        ch,
                        is_till,
                        is_forward,
                        count,
                    ) {
                        state.move_to(target);
                    }
                }
                ViActionResult::Handled
            }
            "," => {
                if let Some((ch, is_till, is_forward)) = self.find_char_state {
                    let count = self.take_count();
                    let cur = state.cursor_offset();
                    if let Some(target) = find_char_inline(
                        state.content(),
                        cur,
                        ch,
                        is_till,
                        !is_forward,
                        count,
                    ) {
                        state.move_to(target);
                    }
                }
                ViActionResult::Handled
            }
            "n" => {
                let count = self.take_count();
                if let Some((pattern, dir)) = self.last_search.clone() {
                    self.jump_to_search_match(&pattern, dir, state, count);
                }
                ViActionResult::Handled
            }
            "N" => {
                let count = self.take_count();
                if let Some((pattern, dir)) = self.last_search.clone() {
                    let opp_dir = match dir {
                        SearchDirection::Forward => SearchDirection::Backward,
                        SearchDirection::Backward => SearchDirection::Forward,
                    };
                    self.jump_to_search_match(&pattern, opp_dir, state, count);
                }
                ViActionResult::Handled
            }
            "*" => {
                let cur = state.cursor_offset();
                if let Some(range) = find_inner_word_range(state.content(), cur) {
                    let word = state.content()[range].to_string();
                    self.last_search = Some((word.clone(), SearchDirection::Forward));
                    self.jump_to_search_match(&word, SearchDirection::Forward, state, 1);
                }
                ViActionResult::Handled
            }
            "#" => {
                let cur = state.cursor_offset();
                if let Some(range) = find_inner_word_range(state.content(), cur) {
                    let word = state.content()[range].to_string();
                    self.last_search = Some((word.clone(), SearchDirection::Backward));
                    self.jump_to_search_match(&word, SearchDirection::Backward, state, 1);
                }
                ViActionResult::Handled
            }
            "%" => {
                let cur = state.cursor_offset();
                if let Some(target) = find_matching_bracket(state.content(), cur) {
                    state.move_to(target);
                }
                ViActionResult::Handled
            }
            _ => {
                if self.handle_motion_key(key, state, false) {
                    ViActionResult::Handled
                } else {
                    ViActionResult::None
                }
            }
        }
    }

    fn handle_motion_key(
        &mut self,
        key: &str,
        state: &mut TextInputState,
        is_visual: bool,
    ) -> bool {
        let has_explicit_count = self.count.is_some();
        let count = self.take_count();
        let cur = state.cursor_offset();
        let content = state.content();

        let (target, is_linewise) = match key {
            "h" => {
                let line_start = find_line_start(content, cur);
                let target = cur.saturating_sub(count).max(line_start);
                (target, false)
            }
            "l" => {
                let line_end = find_line_end(content, cur);
                let target = (cur + count).min(line_end);
                (target, false)
            }
            "j" => {
                let target = find_line_down(content, cur, count);
                (target, true)
            }
            "k" => {
                let target = find_line_up(content, cur, count);
                (target, true)
            }
            "w" => {
                let target = find_word_forward(content, cur, false, count);
                (target, false)
            }
            "W" => {
                let target = find_word_forward(content, cur, true, count);
                (target, false)
            }
            "b" => {
                let target = find_word_backward(content, cur, false, count);
                (target, false)
            }
            "B" => {
                let target = find_word_backward(content, cur, true, count);
                (target, false)
            }
            "e" => {
                let target = find_word_end_forward(content, cur, false, count);
                (target, false)
            }
            "E" => {
                let target = find_word_end_forward(content, cur, true, count);
                (target, false)
            }
            "0" => {
                let target = find_line_start(content, cur);
                (target, false)
            }
            "^" | "_" => {
                let start = find_line_start(content, cur);
                let end = find_line_end(content, cur);
                let target = find_first_non_whitespace(content, start, end);
                (target, false)
            }
            "$" => {
                let target_line_offset = if count > 1 {
                    find_line_down(content, cur, count - 1)
                } else {
                    cur
                };
                let target = find_line_end(content, target_line_offset);
                (target, false)
            }
            "G" => {
                let target = if has_explicit_count {
                    find_line_by_number(content, count)
                } else {
                    let last_line = content.lines().count().max(1);
                    find_line_by_number(content, last_line)
                };
                (target, true)
            }
            "{" => {
                let target = find_paragraph_backward(content, cur, count);
                (target, false)
            }
            "}" => {
                let target = find_paragraph_forward(content, cur, count);
                (target, false)
            }
            _ => return false,
        };

        if !is_visual && let Some(op) = self.pending_op.take() {
            self.apply_operator_range(op, cur, target, is_linewise, state);
        } else {
            state.move_to(target);
        }
        true
    }

    fn apply_operator_range(
        &mut self,
        op: char,
        start_offset: usize,
        end_offset: usize,
        is_linewise: bool,
        state: &mut TextInputState,
    ) {
        let content = state.content();
        let (start, end) = if is_linewise {
            let min_off = start_offset.min(end_offset);
            let max_off = start_offset.max(end_offset);
            let start = find_line_start(content, min_off);
            let mut end = find_line_end(content, max_off);
            if end < content.len() && content.as_bytes()[end] == b'\n' {
                end += 1;
            }
            (start, end)
        } else {
            (start_offset.min(end_offset), start_offset.max(end_offset))
        };

        self.apply_operator_exact_range(op, start..end, is_linewise, state);
    }

    fn apply_operator_exact_range(
        &mut self,
        op: char,
        range: Range<usize>,
        is_linewise: bool,
        state: &mut TextInputState,
    ) {
        let content = state.content();
        let start = range.start.min(content.len());
        let end = range.end.min(content.len());
        let clamped_range = start..end;

        match op {
            'd' => {
                self.yank_buffer = Some(content[clamped_range.clone()].to_string());
                self.is_linewise_yank = is_linewise;
                state.replace_range(clamped_range, "");
                state.move_to(start.min(state.content().len()));
            }
            'c' => {
                self.yank_buffer = Some(content[clamped_range.clone()].to_string());
                self.is_linewise_yank = is_linewise;
                state.replace_range(clamped_range, "");
                state.move_to(start.min(state.content().len()));
                self.mode = ViMode::Insert;
            }
            'y' => {
                self.yank_buffer = Some(content[clamped_range].to_string());
                self.is_linewise_yank = is_linewise;
                state.move_to(start);
            }
            '>' => {
                indent_range(state, clamped_range);
            }
            '<' => {
                outdent_range(state, clamped_range);
            }
            _ => {}
        }
    }

    fn apply_linewise_operator(
        &mut self,
        op: char,
        count: usize,
        state: &mut TextInputState,
    ) {
        let cur = state.cursor_offset();
        let content = state.content();
        let start = find_line_start(content, cur);

        let mut current_offset = start;
        for _ in 0..count {
            let line_end = find_line_end(content, current_offset);
            if line_end < content.len() {
                current_offset = line_end + 1;
            } else {
                current_offset = content.len();
                break;
            }
        }
        let end = current_offset;

        match op {
            'd' => {
                self.yank_buffer = Some(content[start..end].to_string());
                self.is_linewise_yank = true;
                state.replace_range(start..end, "");
                state.move_to(start.min(state.content().len()));
            }
            'c' => {
                self.yank_buffer = Some(content[start..end].to_string());
                self.is_linewise_yank = true;
                let line_end = find_line_end(content, cur);
                state.replace_range(start..line_end, "");
                state.move_to(start);
                self.mode = ViMode::Insert;
            }
            'y' => {
                self.yank_buffer = Some(content[start..end].to_string());
                self.is_linewise_yank = true;
                state.move_to(start);
            }
            '>' => {
                indent_range(state, start..end);
            }
            '<' => {
                outdent_range(state, start..end);
            }
            _ => {}
        }
    }

    fn jump_to_search_match(
        &mut self,
        pattern: &str,
        dir: SearchDirection,
        state: &mut TextInputState,
        count: usize,
    ) {
        if pattern.is_empty() {
            return;
        }

        let content = state.content();
        let mut cur = state.cursor_offset();

        for _ in 0..count {
            match dir {
                SearchDirection::Forward => {
                    let search_from = (cur + 1).min(content.len());
                    if let Some(pos) = content[search_from..].find(pattern) {
                        cur = search_from + pos;
                    } else if let Some(pos) = content[..search_from].find(pattern) {
                        cur = pos;
                    }
                }
                SearchDirection::Backward => {
                    let search_until = cur.min(content.len());
                    if let Some(pos) = content[..search_until].rfind(pattern) {
                        cur = pos;
                    } else if let Some(pos) = content[search_until..].rfind(pattern) {
                        cur = search_until + pos;
                    }
                }
            }
        }

        state.move_to(cur);
    }

    #[must_use]
    pub fn parse_ex_command(cmd_str: &str) -> Option<ExCommand> {
        let trimmed = cmd_str.trim().trim_start_matches(':').trim();
        if trimmed.is_empty() {
            return None;
        }

        if trimmed == "w" || trimmed == "write" {
            return Some(ExCommand::Write);
        }
        if trimmed == "q!" || trimmed == "quit!" {
            return Some(ExCommand::Quit { force: true });
        }
        if trimmed == "q" || trimmed == "quit" {
            return Some(ExCommand::Quit { force: false });
        }
        if trimmed == "wq" || trimmed == "x" || trimmed == "xit" || trimmed == "wq!" {
            return Some(ExCommand::WriteQuit);
        }
        if trimmed == "d" || trimmed == "delete" {
            return Some(ExCommand::DeleteLine);
        }

        if let Ok(line_num) = trimmed.parse::<usize>() {
            return Some(ExCommand::GotoLine(line_num));
        }

        // Substitute commands: %s/foo/bar/g, s/foo/bar/gi, 1,5s/foo/bar/g
        if let Some(s_idx) = trimmed.find('s') {
            let prefix = &trimmed[..s_idx];
            if prefix.is_empty()
                || prefix == "%"
                || prefix
                    .split(',')
                    .all(|part| part.trim().parse::<usize>().is_ok())
            {
                let rest = &trimmed[(s_idx + 1)..];
                if let Some(delim) = rest.chars().next() {
                    let parts: Vec<&str> = rest.split(delim).collect();
                    if parts.len() >= 3 {
                        let pattern = parts[1].to_string();
                        let replacement = parts[2].to_string();
                        let flags = parts.get(3).copied().unwrap_or("");
                        let global = flags.contains('g') || prefix == "%";
                        let ignore_case = flags.contains('i');
                        return Some(ExCommand::Substitute {
                            pattern,
                            replacement,
                            global,
                            ignore_case,
                        });
                    }
                }
            }
        }

        None
    }

    pub fn execute_ex_command(
        &mut self,
        cmd: ExCommand,
        state: &mut TextInputState,
    ) -> ViActionResult {
        match cmd {
            ExCommand::Write => ViActionResult::Save,
            ExCommand::Quit { force } => ViActionResult::Close { force },
            ExCommand::WriteQuit => ViActionResult::SaveAndClose,
            ExCommand::GotoLine(line_num) => {
                let target = find_line_by_number(state.content(), line_num);
                state.move_to(target);
                ViActionResult::Handled
            }
            ExCommand::DeleteLine => {
                let cur = state.cursor_offset();
                let start = find_line_start(state.content(), cur);
                let mut end = find_line_end(state.content(), cur);
                if end < state.content().len() && state.content().as_bytes()[end] == b'\n' {
                    end += 1;
                }
                self.yank_buffer = Some(state.content()[start..end].to_string());
                self.is_linewise_yank = true;
                state.replace_range(start..end, "");
                state.move_to(start.min(state.content().len()));
                ViActionResult::Handled
            }
            ExCommand::Substitute {
                pattern,
                replacement,
                global,
                ignore_case,
            } => {
                let content = state.content().to_string();
                let new_content = if ignore_case {
                    if global {
                        let mut res = String::new();
                        let mut last_end = 0;
                        let lower_content = content.to_lowercase();
                        let lower_pattern = pattern.to_lowercase();
                        if !lower_pattern.is_empty() {
                            let mut search_from = 0;
                            while let Some(pos) = lower_content[search_from..].find(&lower_pattern)
                            {
                                let match_start = search_from + pos;
                                let match_end = match_start + lower_pattern.len();
                                res.push_str(&content[last_end..match_start]);
                                res.push_str(&replacement);
                                last_end = match_end;
                                search_from = match_end;
                            }
                        }
                        res.push_str(&content[last_end..]);
                        res
                    } else {
                        let lower_content = content.to_lowercase();
                        let lower_pattern = pattern.to_lowercase();
                        if let Some(pos) = lower_content.find(&lower_pattern) {
                            let mut res = String::new();
                            res.push_str(&content[..pos]);
                            res.push_str(&replacement);
                            res.push_str(&content[(pos + lower_pattern.len())..]);
                            res
                        } else {
                            content.clone()
                        }
                    }
                } else if global {
                    content.replace(&pattern, &replacement)
                } else if let Some(pos) = content.find(&pattern) {
                    let mut res = String::new();
                    res.push_str(&content[..pos]);
                    res.push_str(&replacement);
                    res.push_str(&content[(pos + pattern.len())..]);
                    res
                } else {
                    content.clone()
                };

                if new_content != content {
                    state.replace_range(0..state.content().len(), &new_content);
                    state.move_to(0);
                }
                ViActionResult::Handled
            }
        }
    }
}

fn toggle_char_case(c: char) -> char {
    if c.is_uppercase() {
        c.to_lowercase().next().unwrap_or(c)
    } else if c.is_lowercase() {
        c.to_uppercase().next().unwrap_or(c)
    } else {
        c
    }
}

fn indent_range(state: &mut TextInputState, range: Range<usize>) {
    let content = state.content();
    let start_line = find_line_start(content, range.start);
    let end_line = find_line_end(content, range.end.max(range.start));

    let mut result = String::new();
    let lines_str = &content[start_line..end_line];
    for (i, line) in lines_str.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if !line.is_empty() {
            result.push_str("    ");
        }
        result.push_str(line);
    }
    state.replace_range(start_line..end_line, &result);
    state.move_to(start_line);
}

fn outdent_range(state: &mut TextInputState, range: Range<usize>) {
    let content = state.content();
    let start_line = find_line_start(content, range.start);
    let end_line = find_line_end(content, range.end.max(range.start));

    let mut result = String::new();
    let lines_str = &content[start_line..end_line];
    for (i, line) in lines_str.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let spaces = line.chars().take_while(|&c| c == ' ').count().min(4);
        result.push_str(&line[spaces..]);
    }
    state.replace_range(start_line..end_line, &result);
    state.move_to(start_line);
}

fn find_line_start(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())]
        .rfind('\n')
        .map_or(0, |idx| idx + 1)
}

fn find_line_end(content: &str, offset: usize) -> usize {
    let offset = offset.min(content.len());
    content[offset..]
        .find('\n')
        .map_or(content.len(), |idx| offset + idx)
}

fn find_first_non_whitespace(content: &str, start: usize, end: usize) -> usize {
    let line = &content[start..end];
    line.find(|c: char| !c.is_whitespace())
        .map_or(start, |idx| start + idx)
}

fn find_line_down(content: &str, offset: usize, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        let line_start = find_line_start(content, cur);
        let col = cur - line_start;
        let line_end = find_line_end(content, cur);
        if line_end >= content.len() {
            return cur;
        }
        let next_line_start = line_end + 1;
        let next_line_end = find_line_end(content, next_line_start);
        let next_line_len = next_line_end - next_line_start;
        cur = next_line_start + col.min(next_line_len);
    }
    cur
}

fn find_line_up(content: &str, offset: usize, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        let line_start = find_line_start(content, cur);
        if line_start == 0 {
            return cur;
        }
        let col = cur - line_start;
        let prev_line_end = line_start - 1;
        let prev_line_start = find_line_start(content, prev_line_end);
        let prev_line_len = prev_line_end - prev_line_start;
        cur = prev_line_start + col.min(prev_line_len);
    }
    cur
}

fn find_line_by_number(content: &str, line_num: usize) -> usize {
    if line_num <= 1 {
        return 0;
    }
    let mut current_line = 1;
    for (idx, ch) in content.char_indices() {
        if ch == '\n' {
            current_line += 1;
            if current_line == line_num {
                return idx + 1;
            }
        }
    }
    find_line_start(content, content.len())
}

#[derive(PartialEq, Eq)]
enum CharClass {
    Word,
    Punct,
    Whitespace,
}

fn char_class(c: char) -> CharClass {
    if c.is_alphanumeric() || c == '_' {
        CharClass::Word
    } else if c.is_whitespace() {
        CharClass::Whitespace
    } else {
        CharClass::Punct
    }
}

fn find_word_forward(content: &str, offset: usize, is_big: bool, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        if cur >= content.len() {
            return content.len();
        }
        let mut chars = content[cur..].char_indices().map(|(i, c)| (cur + i, c));
        if let Some((_, first_ch)) = chars.next() {
            let first_class = char_class(first_ch);
            let mut past_token = false;

            for (idx, ch) in chars {
                let class = char_class(ch);
                if !past_token {
                    if is_big {
                        if class == CharClass::Whitespace {
                            past_token = true;
                        }
                    } else if class != first_class {
                        if class != CharClass::Whitespace {
                            cur = idx;
                            break;
                        }
                        past_token = true;
                    }
                } else if class != CharClass::Whitespace {
                    cur = idx;
                    break;
                }
            }
        }
    }
    cur
}

fn find_word_backward(content: &str, offset: usize, is_big: bool, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        if cur == 0 {
            return 0;
        }
        let chars: Vec<(usize, char)> = content[..cur].char_indices().collect();
        let mut iter = chars.iter().rev();
        let mut target_class = None;
        let mut start_idx = 0;

        for &(idx, ch) in iter.by_ref() {
            let class = char_class(ch);
            if target_class.is_none() {
                if class != CharClass::Whitespace {
                    target_class = Some(class);
                    start_idx = idx;
                }
            } else if is_big {
                if class == CharClass::Whitespace {
                    break;
                }
                start_idx = idx;
            } else if Some(class) == target_class {
                start_idx = idx;
            } else {
                break;
            }
        }
        cur = start_idx;
    }
    cur
}

fn find_word_end_forward(content: &str, offset: usize, is_big: bool, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        if cur + 1 >= content.len() {
            return content.len().saturating_sub(1);
        }
        let start_pos = cur + 1;
        let mut chars = content[start_pos..]
            .char_indices()
            .map(|(i, c)| (start_pos + i, c));

        let mut target_class = None;
        let mut end_idx = cur;

        for (idx, ch) in chars.by_ref() {
            let class = char_class(ch);
            if target_class.is_none() {
                if class != CharClass::Whitespace {
                    target_class = Some(class);
                    end_idx = idx;
                }
            } else if is_big {
                if class == CharClass::Whitespace {
                    break;
                }
                end_idx = idx;
            } else if Some(class) == target_class {
                end_idx = idx;
            } else {
                break;
            }
        }
        cur = end_idx;
    }
    cur
}

fn find_word_end_backward(content: &str, offset: usize, is_big: bool, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        if cur == 0 {
            return 0;
        }
        let chars: Vec<(usize, char)> = content[..cur].char_indices().collect();
        let mut iter = chars.iter().rev();
        for &(idx, ch) in iter.by_ref() {
            let class = char_class(ch);
            if is_big {
                if class != CharClass::Whitespace {
                    cur = idx;
                    break;
                }
            } else if class != CharClass::Whitespace {
                cur = idx;
                break;
            }
        }
    }
    cur
}

fn find_char_inline(
    content: &str,
    offset: usize,
    target: char,
    is_till: bool,
    is_forward: bool,
    count: usize,
) -> Option<usize> {
    let line_start = find_line_start(content, offset);
    let line_end = find_line_end(content, offset);
    let line_slice = &content[line_start..line_end];

    if is_forward {
        let rel_offset = offset.saturating_sub(line_start);
        let mut found_count = 0;
        for (idx, ch) in line_slice.char_indices() {
            if idx > rel_offset && ch == target {
                found_count += 1;
                if found_count == count {
                    let mut res = line_start + idx;
                    if is_till {
                        res = res.saturating_sub(1).max(offset);
                    }
                    return Some(res);
                }
            }
        }
    } else {
        let rel_offset = offset.saturating_sub(line_start);
        let mut found_count = 0;
        for (idx, ch) in line_slice.char_indices().rev() {
            if idx < rel_offset && ch == target {
                found_count += 1;
                if found_count == count {
                    let mut res = line_start + idx;
                    if is_till {
                        res = (res + 1).min(offset);
                    }
                    return Some(res);
                }
            }
        }
    }
    None
}

fn find_paragraph_forward(content: &str, offset: usize, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        let mut line_cur = find_line_end(content, cur);
        if line_cur < content.len() && content.as_bytes()[line_cur] == b'\n' {
            line_cur += 1;
        }
        let mut found_text = false;
        let mut target = content.len();

        while line_cur < content.len() {
            let line_end = find_line_end(content, line_cur);
            let line = &content[line_cur..line_end];
            if line.trim().is_empty() {
                if found_text {
                    target = line_cur;
                    break;
                }
            } else {
                found_text = true;
            }
            if line_end < content.len() && content.as_bytes()[line_end] == b'\n' {
                line_cur = line_end + 1;
            } else {
                break;
            }
        }
        cur = target;
    }
    cur
}

fn find_paragraph_backward(content: &str, offset: usize, count: usize) -> usize {
    let mut cur = offset;
    for _ in 0..count {
        let mut line_cur = find_line_start(content, cur);
        let mut found_text = false;
        let mut target = 0;

        while line_cur > 0 {
            let prev_line_end = line_cur - 1;
            let prev_line_start = find_line_start(content, prev_line_end);
            let line = &content[prev_line_start..prev_line_end];
            if line.trim().is_empty() {
                if found_text {
                    target = prev_line_start;
                    break;
                }
            } else {
                found_text = true;
            }
            line_cur = prev_line_start;
        }
        cur = target;
    }
    cur
}

fn find_matching_bracket(content: &str, offset: usize) -> Option<usize> {
    let line_end = find_line_end(content, offset);
    let line_slice = &content[offset.min(line_end)..line_end];

    // Find first bracket on line starting from offset
    let bracket_info = line_slice
        .char_indices()
        .find_map(|(idx, ch)| match ch {
            '(' => Some((offset + idx, '(', ')', true)),
            ')' => Some((offset + idx, ')', '(', false)),
            '[' => Some((offset + idx, '[', ']', true)),
            ']' => Some((offset + idx, ']', '[', false)),
            '{' => Some((offset + idx, '{', '}', true)),
            '}' => Some((offset + idx, '}', '{', false)),
            _ => None,
        });

    let (pos, open_ch, close_ch, forward) = bracket_info?;
    let mut depth = 1;

    if forward {
        for (idx, ch) in content[(pos + 1)..].char_indices() {
            if ch == open_ch {
                depth += 1;
            } else if ch == close_ch {
                depth -= 1;
                if depth == 0 {
                    return Some(pos + 1 + idx);
                }
            }
        }
    } else {
        let chars: Vec<(usize, char)> = content[..pos].char_indices().collect();
        for &(idx, ch) in chars.iter().rev() {
            if ch == open_ch {
                depth += 1;
            } else if ch == close_ch {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
        }
    }
    None
}

fn find_inner_word_range(content: &str, offset: usize) -> Option<Range<usize>> {
    if content.is_empty() {
        return None;
    }
    let cur = offset.min(content.len().saturating_sub(1));
    let target_char = content[cur..].chars().next()?;
    let target_class = char_class(target_char);

    let start = content[..cur]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| char_class(c) == target_class)
        .last()
        .map_or(cur, |(idx, _)| idx);

    let end = content[cur..]
        .char_indices()
        .take_while(|&(_, c)| char_class(c) == target_class)
        .last()
        .map_or(cur, |(idx, c)| cur + idx + c.len_utf8());

    Some(start..end)
}

fn find_around_word_range(content: &str, offset: usize) -> Option<Range<usize>> {
    let inner = find_inner_word_range(content, offset)?;
    let after_ws_end = content[inner.end..]
        .char_indices()
        .take_while(|&(_, c)| c.is_whitespace() && c != '\n')
        .last()
        .map_or(inner.end, |(idx, c)| inner.end + idx + c.len_utf8());

    if after_ws_end > inner.end {
        Some(inner.start..after_ws_end)
    } else {
        let before_ws_start = content[..inner.start]
            .char_indices()
            .rev()
            .take_while(|&(_, c)| c.is_whitespace() && c != '\n')
            .last()
            .map_or(inner.start, |(idx, _)| idx);
        Some(before_ws_start..inner.end)
    }
}

fn find_quote_range(
    content: &str,
    offset: usize,
    quote_char: char,
    inner: bool,
) -> Option<Range<usize>> {
    let line_start = find_line_start(content, offset);
    let line_end = find_line_end(content, offset);
    let line_str = &content[line_start..line_end];

    let quote_positions: Vec<usize> = line_str
        .char_indices()
        .filter_map(|(idx, c)| (c == quote_char).then_some(line_start + idx))
        .collect();

    if quote_positions.len() < 2 {
        return None;
    }

    for pair in quote_positions.chunks(2) {
        if pair.len() == 2 {
            let (q_start, q_end) = (pair[0], pair[1]);
            if offset <= q_end {
                return if inner {
                    Some((q_start + 1)..q_end)
                } else {
                    Some(q_start..(q_end + 1))
                };
            }
        }
    }
    None
}

fn find_bracket_range(
    content: &str,
    offset: usize,
    open_ch: char,
    close_ch: char,
    inner: bool,
) -> Option<Range<usize>> {
    let chars: Vec<(usize, char)> = content.char_indices().collect();
    let mut best_pair = None;

    // Scan backwards from offset for open bracket
    for i in (0..chars.len()).rev() {
        let (idx_open, ch_open) = chars[i];
        if idx_open <= offset && ch_open == open_ch {
            // Find matching close bracket
            let mut depth = 1;
            for &(idx_close, ch_close) in &chars[(i + 1)..] {
                if ch_close == open_ch {
                    depth += 1;
                } else if ch_close == close_ch {
                    depth -= 1;
                    if depth == 0 {
                        if idx_close >= offset {
                            best_pair = Some((idx_open, idx_close));
                        }
                        break;
                    }
                }
            }
            if best_pair.is_some() {
                break;
            }
        }
    }

    let (open_pos, close_pos) = best_pair?;
    if inner {
        Some((open_pos + 1)..close_pos)
    } else {
        Some(open_pos..(close_pos + 1))
    }
}

fn find_paragraph_range(content: &str, offset: usize, inner: bool) -> Option<Range<usize>> {
    if content.is_empty() {
        return None;
    }
    let start = find_paragraph_backward(content, offset, 1);
    let end = find_paragraph_forward(content, offset, 1);
    if inner {
        Some(start..end)
    } else {
        let mut extended_end = end;
        if extended_end < content.len() && content.as_bytes()[extended_end] == b'\n' {
            extended_end += 1;
        }
        Some(start..extended_end)
    }
}
