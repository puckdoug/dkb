#![allow(
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::uninlined_format_args,
    clippy::if_not_else,
    clippy::map_unwrap_or
)]

use std::ops::Range;

use gpui::{
    App, Context, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler,
    FocusHandle, Focusable, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    MouseButton, Point, Pixels, Render, SharedString, Style, TextAlign, TextRun, Window,
    actions, div, prelude::*, px, rgb, rgba, relative,
};
use uuid::Uuid;

use crate::config::Config;
use crate::item::{Category, Item};
use crate::link::{find_link_at_offset, format_markdown_link};
use crate::storage::{Location, Storage};
use crate::text_input::TextInputState;
use crate::theme::Theme;
use crate::vi::{SearchDirection, ViActionResult, ViMode, ViState, VisualKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorEvent {
    Save,
    Close,
}

actions!(
    dkb_editor,
    [
        EditorBackspace,
        EditorDelete,
        EditorUp,
        EditorDown,
        EditorLeft,
        EditorRight,
        EditorSelectUp,
        EditorSelectDown,
        EditorSelectLeft,
        EditorSelectRight,
        EditorSelectAll,
        EditorUndo,
        EditorRedo,
        EditorPaste,
        EditorCut,
        EditorCopy,
        EditorEnter,
        EditorEscape,
        SaveEditor,
        CloseWindow,
        EditorCreateSubItem,
        EditorFollowLink,
        EditorNavigateBack,
    ]
);

pub struct ItemEditor {
    pub state: TextInputState,
    pub vi_state: ViState,
    pub focus_handle: FocusHandle,
    pub editing_item_id: Option<Uuid>,
    pub is_new: bool,
    pub config: Config,
    pub is_torn_off: bool,
    pub subitem_prompt_open: bool,
    pub subitem_prompt_text: String,
    pub context_menu_pos: Option<Point<Pixels>>,
    pub history_stack: Vec<(Uuid, String)>,
    pub cached_is_done: bool,
}

impl gpui::EventEmitter<EditorEvent> for ItemEditor {}

impl ItemEditor {
    pub fn new(
        cx: &mut Context<Self>,
        initial: &str,
        editing_item_id: Option<Uuid>,
        is_new: bool,
        config: Config,
        is_torn_off: bool,
    ) -> Self {
        let (initial_content, initial_cursor) = if is_new && initial.is_empty() {
            ("# ", 2)
        } else {
            (initial, 0)
        };
        let mut state = TextInputState::new(initial_content);
        if initial_cursor > 0 {
            state.move_to(initial_cursor);
        }

        Self {
            state,
            vi_state: ViState::new(),
            focus_handle: cx.focus_handle().tab_stop(true),
            editing_item_id,
            is_new,
            config,
            is_torn_off,
            subitem_prompt_open: false,
            subitem_prompt_text: String::new(),
            context_menu_pos: None,
            history_stack: Vec::new(),
            cached_is_done: false,
        }
    }

    #[must_use]
    pub fn content(&self) -> &str {
        self.state.content()
    }

    pub fn process_vi_action(
        &mut self,
        action: &ViActionResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            ViActionResult::Save => {
                self.on_save(&SaveEditor, window, cx);
            }
            ViActionResult::Close { .. } => {
                self.on_close(&CloseWindow, window, cx);
            }
            ViActionResult::SaveAndClose => {
                self.on_save(&SaveEditor, window, cx);
                self.on_close(&CloseWindow, window, cx);
            }
            ViActionResult::ExecuteEx(cmd) => {
                let action = self.vi_state.execute_ex_command(cmd.clone(), &mut self.state);
                self.process_vi_action(&action, window, cx);
            }
            ViActionResult::Handled | ViActionResult::None => {}
        }
    }

    pub fn on_save(&mut self, _: &SaveEditor, _window: &mut Window, cx: &mut Context<Self>) {
        let content = self.state.content().to_string();
        let title = content
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .to_string();
        if title.is_empty() {
            return;
        }

        if self.is_new {
            let item = Item::new(&content);
            let location = Location::Active(Category::Today);
            if Storage::write_item(&self.config.data_dir, &item, &location).is_ok() {
                self.editing_item_id = Some(item.id);
                self.is_new = false;
                self.cached_is_done = false;
            }
        } else if let Some(id) = self.editing_item_id {
            let locations = [
                Location::Backlog,
                Location::Active(Category::Yesterday),
                Location::Active(Category::Today),
                Location::Active(Category::ThisWeek),
                Location::Active(Category::NextWeek),
                Location::Done,
            ];
            let location = locations
                .iter()
                .find(|loc| {
                    self.config
                        .data_dir
                        .join(loc.to_path())
                        .join(format!("{id}.md"))
                        .exists()
                })
                .copied()
                .unwrap_or(Location::Active(Category::Today));

            let item = Item {
                id,
                body: content,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                completed_at: if matches!(location, Location::Done) {
                    Some(chrono::Utc::now())
                } else {
                    None
                },
            };
            let _ = Storage::write_item(&self.config.data_dir, &item, &location);
            self.cached_is_done = matches!(location, Location::Done);
        }
        cx.emit(EditorEvent::Save);
        cx.notify();
    }

    pub fn on_close(&mut self, _: &CloseWindow, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_torn_off {
            window.remove_window();
        } else {
            cx.emit(EditorEvent::Close);
        }
    }

    pub fn on_backspace(&mut self, _: &EditorBackspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            self.subitem_prompt_text.pop();
            cx.notify();
            return;
        }
        if self.config.vi_mode && matches!(self.vi_state.mode, ViMode::Command | ViMode::Search(_)) {
            let action = self.vi_state.handle_key("backspace", &mut self.state);
            self.process_vi_action(&action, window, cx);
        } else {
            self.state.backspace();
        }
        cx.notify();
    }

    pub fn on_delete(&mut self, _: &EditorDelete, window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        if self.config.vi_mode && matches!(self.vi_state.mode, ViMode::Command | ViMode::Search(_)) {
            let action = self.vi_state.handle_key("backspace", &mut self.state);
            self.process_vi_action(&action, window, cx);
        } else {
            self.state.delete();
        }
        cx.notify();
    }

    pub fn on_up(&mut self, _: &EditorUp, window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        if self.config.vi_mode && self.vi_state.mode != ViMode::Insert {
            let action = self.vi_state.handle_key("k", &mut self.state);
            self.process_vi_action(&action, window, cx);
        } else {
            self.state.move_up();
        }
        cx.notify();
    }

    pub fn on_down(&mut self, _: &EditorDown, window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        if self.config.vi_mode && self.vi_state.mode != ViMode::Insert {
            let action = self.vi_state.handle_key("j", &mut self.state);
            self.process_vi_action(&action, window, cx);
        } else {
            self.state.move_down();
        }
        cx.notify();
    }

    pub fn on_left(&mut self, _: &EditorLeft, window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        if self.config.vi_mode && self.vi_state.mode != ViMode::Insert {
            let action = self.vi_state.handle_key("h", &mut self.state);
            self.process_vi_action(&action, window, cx);
        } else {
            self.state.move_left();
        }
        cx.notify();
    }

    pub fn on_right(&mut self, _: &EditorRight, window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        if self.config.vi_mode && self.vi_state.mode != ViMode::Insert {
            let action = self.vi_state.handle_key("l", &mut self.state);
            self.process_vi_action(&action, window, cx);
        } else {
            self.state.move_right();
        }
        cx.notify();
    }

    pub fn on_select_up(&mut self, _: &EditorSelectUp, _window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        self.state.select_up();
        cx.notify();
    }

    pub fn on_select_down(&mut self, _: &EditorSelectDown, _window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        self.state.select_down();
        cx.notify();
    }

    pub fn on_select_left(&mut self, _: &EditorSelectLeft, _window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        self.state.select_left();
        cx.notify();
    }

    pub fn on_select_right(&mut self, _: &EditorSelectRight, _window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        self.state.select_right();
        cx.notify();
    }

    pub fn on_enter(&mut self, _: &EditorEnter, window: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            self.confirm_subitem_prompt(window, cx);
            return;
        }
        if self.config.vi_mode {
            if matches!(self.vi_state.mode, ViMode::Command | ViMode::Search(_)) {
                let action = self.vi_state.handle_key("enter", &mut self.state);
                self.process_vi_action(&action, window, cx);
            } else if self.vi_state.mode == ViMode::Normal {
                self.state.move_down();
            } else {
                self.state.insert("\n");
            }
        } else {
            self.state.insert("\n");
        }
        cx.notify();
    }

    pub fn on_escape(&mut self, _: &EditorEscape, window: &mut Window, cx: &mut Context<Self>) {
        if self.config.vi_mode {
            let action = self.vi_state.handle_key("escape", &mut self.state);
            self.process_vi_action(&action, window, cx);
            cx.notify();
        }
    }

    pub fn on_select_all(&mut self, _: &EditorSelectAll, _: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        self.state.select_all();
        cx.notify();
    }

    pub fn on_undo(&mut self, _: &EditorUndo, _: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        self.state.undo();
        cx.notify();
    }

    pub fn on_redo(&mut self, _: &EditorRedo, _: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        self.state.redo();
        cx.notify();
    }

    pub fn on_paste(&mut self, _: &EditorPaste, _: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.state.insert(&text);
            cx.notify();
        }
    }

    pub fn on_copy(&mut self, _: &EditorCopy, _: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        let range = self.state.selected_range();
        if !range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.state.content()[range].to_string(),
            ));
        }
    }

    pub fn on_cut(&mut self, _: &EditorCut, _: &mut Window, cx: &mut Context<Self>) {
        if self.subitem_prompt_open {
            return;
        }
        let range = self.state.selected_range();
        if !range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.state.content()[range].to_string(),
            ));
            self.state.insert("");
            cx.notify();
        }
    }

    pub fn on_create_sub_item(&mut self, _: &EditorCreateSubItem, window: &mut Window, cx: &mut Context<Self>) {
        let sel_range = self.state.selected_range();
        if !sel_range.is_empty() {
            let selected_text = self.state.content()[sel_range.clone()].to_string();
            let sub_item = Item::new(&selected_text);
            let location = Location::Active(Category::Today);
            if Storage::write_item(&self.config.data_dir, &sub_item, &location).is_ok() {
                let link_text = format_markdown_link(&selected_text, sub_item.id);
                self.state.replace_range(sel_range, &link_text);
                self.on_save(&SaveEditor, window, cx);
            }
        } else {
            self.subitem_prompt_open = true;
            self.subitem_prompt_text.clear();
        }
        self.context_menu_pos = None;
        cx.notify();
    }

    pub fn confirm_subitem_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let title = self.subitem_prompt_text.trim().to_string();
        if !title.is_empty() {
            let sub_item = Item::new(&title);
            let location = Location::Active(Category::Today);
            if Storage::write_item(&self.config.data_dir, &sub_item, &location).is_ok() {
                let link_text = format_markdown_link(&title, sub_item.id);
                self.state.insert(&link_text);
                self.on_save(&SaveEditor, window, cx);
            }
        }
        self.subitem_prompt_open = false;
        self.subitem_prompt_text.clear();
        cx.notify();
    }

    pub fn cancel_subitem_prompt(&mut self, cx: &mut Context<Self>) {
        self.subitem_prompt_open = false;
        self.subitem_prompt_text.clear();
        cx.notify();
    }

    pub fn follow_link_at_offset(&mut self, offset: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(span) = find_link_at_offset(self.state.content(), offset) {
            self.on_save(&SaveEditor, window, cx);
            if let Some(curr_id) = self.editing_item_id {
                let title = Item::extract_title(self.state.content());
                self.history_stack.push((curr_id, title));
            }
            if let Ok(board) = Storage::load_board(&self.config.data_dir)
                && let Some(target_item) = board.find_item(&span.target_id)
            {
                self.state = TextInputState::new(&target_item.body);
                self.editing_item_id = Some(span.target_id);
                self.is_new = false;
                self.cached_is_done = matches!(board.find_item_location(&span.target_id), Some(Location::Done));
                self.context_menu_pos = None;
                cx.notify();
            }
        }
    }

    pub fn on_follow_link(&mut self, _: &EditorFollowLink, window: &mut Window, cx: &mut Context<Self>) {
        let cur = self.state.cursor_offset();
        self.follow_link_at_offset(cur, window, cx);
    }

    pub fn navigate_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((parent_id, _title)) = self.history_stack.pop() {
            self.on_save(&SaveEditor, window, cx);
            if let Ok(board) = Storage::load_board(&self.config.data_dir)
                && let Some(item) = board.find_item(&parent_id)
            {
                self.state = TextInputState::new(&item.body);
                self.editing_item_id = Some(parent_id);
                self.is_new = false;
                self.cached_is_done = matches!(board.find_item_location(&parent_id), Some(Location::Done));
                self.context_menu_pos = None;
                cx.notify();
            }
        }
    }

    pub fn on_navigate_back(&mut self, _: &EditorNavigateBack, window: &mut Window, cx: &mut Context<Self>) {
        self.navigate_back(window, cx);
    }

    pub fn navigate_to_history_index(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index < self.history_stack.len() {
            self.on_save(&SaveEditor, window, cx);
            let (target_id, _title) = self.history_stack[index].clone();
            self.history_stack.truncate(index);
            if let Ok(board) = Storage::load_board(&self.config.data_dir)
                && let Some(item) = board.find_item(&target_id)
            {
                self.state = TextInputState::new(&item.body);
                self.editing_item_id = Some(target_id);
                self.is_new = false;
                self.cached_is_done = matches!(board.find_item_location(&target_id), Some(Location::Done));
                self.context_menu_pos = None;
                cx.notify();
            }
        }
    }

    pub fn is_done(&self) -> bool {
        self.cached_is_done
    }

    pub fn refresh_done_status(&mut self) {
        if let Some(id) = self.editing_item_id
            && let Ok(board) = Storage::load_board(&self.config.data_dir)
            && let Some(loc) = board.find_item_location(&id)
        {
            self.cached_is_done = matches!(loc, Location::Done);
        } else {
            self.cached_is_done = false;
        }
    }
}

impl Focusable for ItemEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for ItemEditor {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.state.range_from_utf16(&range_utf16);
        actual_range.replace(self.state.range_to_utf16(&range));
        Some(self.state.content()[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<gpui::UTF16Selection> {
        let range = self.state.selected_range();
        Some(gpui::UTF16Selection {
            range: self.state.range_to_utf16(&range),
            reversed: range.start > range.end,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.config.vi_mode {
            let action = self.vi_state.handle_key(new_text, &mut self.state);
            self.process_vi_action(&action, window, cx);
            if action == ViActionResult::None && self.vi_state.mode == ViMode::Insert {
                let range = range_utf16
                    .as_ref()
                    .map_or_else(|| self.state.selected_range(), |r| self.state.range_from_utf16(r));
                self.state.replace_range(range, new_text);
            }
        } else {
            let range = range_utf16
                .as_ref()
                .map_or_else(|| self.state.selected_range(), |r| self.state.range_from_utf16(r));
            self.state.replace_range(range, new_text);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.config.vi_mode {
            let action = self.vi_state.handle_key(new_text, &mut self.state);
            self.process_vi_action(&action, window, cx);
            if action == ViActionResult::None && self.vi_state.mode == ViMode::Insert {
                let range = range_utf16
                    .as_ref()
                    .map_or_else(|| self.state.selected_range(), |r| self.state.range_from_utf16(r));
                self.state.replace_range(range, new_text);
            }
        } else {
            let range = range_utf16
                .as_ref()
                .map_or_else(|| self.state.selected_range(), |r| self.state.range_from_utf16(r));
            self.state.replace_range(range, new_text);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _bounds: gpui::Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<gpui::Bounds<Pixels>> {
        None
    }

    fn character_index_for_point(
        &mut self,
        _point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

pub struct EditorElement {
    pub editor: Entity<ItemEditor>,
}

pub struct EditorPrepaintState {
    lines: Vec<gpui::WrappedLine>,
    cursor: Option<gpui::PaintQuad>,
    selections: Vec<gpui::PaintQuad>,
}

impl IntoElement for EditorElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = EditorPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: gpui::Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let editor = self.editor.read(cx);
        let content: SharedString = editor.state.content().to_string().into();
        let selected_range = editor.state.selected_range();
        let mut style = window.text_style();
        style.font_family = editor.config.font_family.clone().into();
        style.font_size = px(13.).into();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = px(20.);
        let wrap_width = bounds.size.width;

        let is_visual = editor.config.vi_mode
            && matches!(editor.vi_state.mode, ViMode::Visual(_));
        let is_block_cursor = editor.config.vi_mode
            && matches!(editor.vi_state.mode, ViMode::Normal | ViMode::Command | ViMode::Replace);

        let cursor_offset = if is_visual {
            editor.vi_state.visual_head.unwrap_or_else(|| editor.state.cursor_offset())
        } else {
            editor.state.cursor_offset()
        };

        let raw_content = editor.state.content();
        let prefix = &raw_content[..cursor_offset.min(raw_content.len())];
        let line_idx = prefix.matches('\n').count();
        let line_start = prefix.rfind('\n').map_or(0, |idx| idx + 1);
        let col_text = &prefix[line_start..];

        let cursor_y = bounds.top() + (line_idx as f32) * line_height;
        let cursor_x = if col_text.is_empty() {
            bounds.left()
        } else {
            let col_run = TextRun {
                len: col_text.len(),
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let shaped = window
                .text_system()
                .shape_line(col_text.to_string().into(), font_size, &[col_run], None);
            bounds.left() + shaped.width
        };

        let (cursor_bounds, cursor_color) = if is_visual {
            (
                gpui::Bounds::new(
                    Point::new(cursor_x, cursor_y + line_height - px(2.)),
                    gpui::size(px(8.), px(2.)),
                ),
                rgb(0x0a84ff),
            )
        } else if is_block_cursor {
            (
                gpui::Bounds::new(
                    Point::new(cursor_x, cursor_y),
                    gpui::size(px(8.), line_height),
                ),
                rgba(0x0a84ff99),
            )
        } else {
            (
                gpui::Bounds::new(
                    Point::new(cursor_x, cursor_y),
                    gpui::size(px(2.), line_height),
                ),
                rgb(0x0a84ff),
            )
        };

        let cursor = Some(gpui::fill(cursor_bounds, cursor_color));

        let mut selections = Vec::new();
        if !selected_range.is_empty() {
            let start_off = selected_range.start.min(raw_content.len());
            let end_off = selected_range.end.min(raw_content.len());
            let start_prefix = &raw_content[..start_off];
            let end_prefix = &raw_content[..end_off];
            let start_line = start_prefix.matches('\n').count();
            let end_line = end_prefix.matches('\n').count();

            for line_idx in start_line..=end_line {
                let line_y = bounds.top() + (line_idx as f32) * line_height;
                let (line_start_x, line_end_x) = if line_idx == start_line && line_idx == end_line {
                    let col_start = start_prefix.rfind('\n').map_or(0, |idx| idx + 1);
                    let col_text_start = &start_prefix[col_start..];
                    let x1 = if col_text_start.is_empty() {
                        bounds.left()
                    } else {
                        let r = TextRun {
                            len: col_text_start.len(),
                            font: style.font(),
                            color: style.color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        bounds.left()
                            + window
                                .text_system()
                                .shape_line(col_text_start.to_string().into(), font_size, &[r], None)
                                .width
                    };

                    let col_end = end_prefix.rfind('\n').map_or(0, |idx| idx + 1);
                    let col_text_end = &end_prefix[col_end..];
                    let x2 = if col_text_end.is_empty() {
                        bounds.left()
                    } else {
                        let r = TextRun {
                            len: col_text_end.len(),
                            font: style.font(),
                            color: style.color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        bounds.left()
                            + window
                                .text_system()
                                .shape_line(col_text_end.to_string().into(), font_size, &[r], None)
                                .width
                    };
                    (x1, x2)
                } else if line_idx == start_line {
                    let col_start = start_prefix.rfind('\n').map_or(0, |idx| idx + 1);
                    let col_text_start = &start_prefix[col_start..];
                    let x1 = if col_text_start.is_empty() {
                        bounds.left()
                    } else {
                        let r = TextRun {
                            len: col_text_start.len(),
                            font: style.font(),
                            color: style.color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        bounds.left()
                            + window
                                .text_system()
                                .shape_line(col_text_start.to_string().into(), font_size, &[r], None)
                                .width
                    };
                    (x1, bounds.left() + bounds.size.width)
                } else if line_idx == end_line {
                    let col_end = end_prefix.rfind('\n').map_or(0, |idx| idx + 1);
                    let col_text_end = &end_prefix[col_end..];
                    let x2 = if col_text_end.is_empty() {
                        bounds.left()
                    } else {
                        let r = TextRun {
                            len: col_text_end.len(),
                            font: style.font(),
                            color: style.color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        bounds.left()
                            + window
                                .text_system()
                                .shape_line(col_text_end.to_string().into(), font_size, &[r], None)
                                .width
                    };
                    (bounds.left(), x2)
                } else {
                    (bounds.left(), bounds.left() + bounds.size.width)
                };

                if line_end_x > line_start_x {
                    selections.push(gpui::fill(
                        gpui::Bounds::from_corners(
                            Point::new(line_start_x, line_y),
                            Point::new(line_end_x, line_y + line_height),
                        ),
                        rgba(0x0a84ff40),
                    ));
                }
            }
        }

        if content.is_empty() {
            return EditorPrepaintState {
                lines: Vec::new(),
                cursor,
                selections,
            };
        }

        let run = TextRun {
            len: content.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let Ok(lines) = window.text_system().shape_text(
            content,
            font_size,
            &[run],
            Some(wrap_width),
            None,
        ) else {
            return EditorPrepaintState {
                lines: Vec::new(),
                cursor,
                selections,
            };
        };

        EditorPrepaintState {
            lines: lines.into_iter().collect(),
            cursor,
            selections,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: gpui::Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.editor.read(cx).focus_handle.clone();

        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.editor.clone()),
            cx,
        );

        for selection in prepaint.selections.drain(..) {
            window.paint_quad(selection);
        }

        let line_height = px(20.);
        let mut y = bounds.top();
        for line in &prepaint.lines {
            line.paint(
                Point::new(bounds.left(), y),
                line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
            y += line.size(line_height).height;
        }

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }
    }
}

impl Render for ItemEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = matches!(
            window.appearance(),
            gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
        );
        let theme = Theme::resolve(self.config.theme_mode, is_dark);
        let show_line_numbers = self.config.line_numbers;
        let show_vi_status = self.config.vi_mode;

        let line_count = self.state.line_count();

        let gutter = if show_line_numbers {
            let mut numbers_col = div()
                .flex()
                .flex_col()
                .items_end()
                .w(px(36.))
                .py(px(16.))
                .px(px(8.))
                .bg(theme.bg_column)
                .border_r_1()
                .border_color(theme.border)
                .font_family(self.config.font_family.clone())
                .text_size(px(13.))
                .line_height(px(20.))
                .text_color(theme.text_secondary);

            for i in 1..=line_count {
                numbers_col = numbers_col.child(
                    div()
                        .h(px(20.))
                        .flex()
                        .items_center()
                        .justify_end()
                        .child(format!("{}", i)),
                );
            }
            Some(numbers_col)
        } else {
            None
        };

        let vi_status_text = match self.vi_state.mode {
            ViMode::Normal => format!("-- {} --", crate::i18n::t("editor.status.normal", self.config.language)),
            ViMode::Insert => format!("-- {} --", crate::i18n::t("editor.status.insert", self.config.language)),
            ViMode::Visual(VisualKind::Character) => format!("-- {} --", crate::i18n::t("editor.status.visual", self.config.language)),
            ViMode::Visual(VisualKind::Line) => format!("-- {} LINE --", crate::i18n::t("editor.status.visual", self.config.language)),
            ViMode::Command => format!("-- {} --", crate::i18n::t("editor.status.command", self.config.language)),
            ViMode::Search(_) => "-- SEARCH --".to_string(),
            ViMode::Replace => "-- REPLACE --".to_string(),
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_surface)
            .track_focus(&self.focus_handle)
            .key_context("ItemEditor")
            .cursor(gpui::CursorStyle::IBeam)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if this.subitem_prompt_open {
                    if key == "enter" || key == "Enter" || key == "\n" || key == "\r" {
                        this.confirm_subitem_prompt(window, cx);
                        return;
                    } else if key == "escape" || key == "Esc" || key == "Escape" || key == "\x1b" {
                        // Esc must NOT close the prompt — fall through to the vi
                        // handler below so the user can switch modes.
                    } else if key == "backspace" || key == "Backspace" || key == "\x08" || key == "\x7f" {
                        this.subitem_prompt_text.pop();
                        cx.notify();
                        return;
                    } else if !event.keystroke.modifiers.control && !event.keystroke.modifiers.alt && key.chars().count() == 1 {
                        this.subitem_prompt_text.push_str(key);
                        cx.notify();
                        return;
                    } else {
                        return;
                    }
                    // For Esc, fall through to the vi handler below.
                }

                if (key == "escape" || key == "Esc" || key == "Escape" || key == "\x1b")
                    && this.config.vi_mode
                {
                    let action = this.vi_state.handle_key("escape", &mut this.state);
                    this.process_vi_action(&action, window, cx);
                    cx.notify();
                }
            }))
            .on_action(cx.listener(Self::on_backspace))
            .on_action(cx.listener(Self::on_delete))
            .on_action(cx.listener(Self::on_up))
            .on_action(cx.listener(Self::on_down))
            .on_action(cx.listener(Self::on_left))
            .on_action(cx.listener(Self::on_right))
            .on_action(cx.listener(Self::on_select_up))
            .on_action(cx.listener(Self::on_select_down))
            .on_action(cx.listener(Self::on_select_left))
            .on_action(cx.listener(Self::on_select_right))
            .on_action(cx.listener(Self::on_select_all))
            .on_action(cx.listener(Self::on_enter))
            .on_action(cx.listener(Self::on_escape))
            .on_action(cx.listener(Self::on_undo))
            .on_action(cx.listener(Self::on_redo))
            .on_action(cx.listener(Self::on_paste))
            .on_action(cx.listener(Self::on_copy))
            .on_action(cx.listener(Self::on_cut))
            .on_action(cx.listener(Self::on_save))
            .on_action(cx.listener(Self::on_close))
            .on_action(cx.listener(Self::on_create_sub_item))
            .on_action(cx.listener(Self::on_follow_link))
            .on_action(cx.listener(Self::on_navigate_back))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &gpui::MouseDownEvent, _window, cx| {
                    this.context_menu_pos = Some(event.position);
                    cx.notify();
                }),
            )
            // Top breadcrumb & done badge bar
            .children(if !self.history_stack.is_empty() || self.is_done() {
                let mut row = div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .px(px(12.))
                    .py(px(6.))
                    .bg(theme.bg_column)
                    .border_b_1()
                    .border_color(theme.border)
                    .text_sm();

                let mut crumbs = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.));

                if !self.history_stack.is_empty() {
                    for (idx, (item_id, title)) in self.history_stack.iter().enumerate() {
                        let _ = item_id;
                        crumbs = crumbs
                            .child(
                                div()
                                    .cursor_pointer()
                                    .text_color(theme.accent)
                                    .child(title.clone())
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, window, cx| {
                                            this.navigate_to_history_index(idx, window, cx);
                                        }),
                                    ),
                            )
                            .child(div().text_color(theme.text_secondary).child(">"));
                    }
                    crumbs = crumbs.child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child(Item::extract_title(self.state.content())),
                    );
                }

                row = row.child(crumbs);

                if self.is_done() {
                    row = row.child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .rounded(px(4.))
                            .bg(rgb(0x2e7d32))
                            .text_color(rgb(0xffffff))
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("✅ Done"),
                    );
                }

                Some(row)
            } else {
                None
            })
            // Editor main area (gutter + text)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                            this.context_menu_pos = None;
                            if event.modifiers.control || event.modifiers.platform {
                                this.follow_link_at_offset(this.state.cursor_offset(), window, cx);
                            }
                        }),
                    )
                    .children(gutter)
                    .child(
                        div()
                            .flex_1()
                            .p(px(16.))
                            .font_family(self.config.font_family.clone())
                            .text_size(px(13.))
                            .line_height(px(20.))
                            .text_color(theme.text_primary)
                            .child(EditorElement {
                                editor: cx.entity(),
                            }),
                    ),
            )
            // Vi Mode Status Bar / Command Line / Search Bar
            .when(show_vi_status, |this| {
                let status_content = match &self.vi_state.mode {
                    ViMode::Command => {
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme.accent)
                                    .child(":"),
                            )
                            .child(
                                div()
                                    .text_color(theme.text_primary)
                                    .child(self.vi_state.command_buffer.clone()),
                            )
                            .child(
                                div()
                                    .w(px(2.))
                                    .h(px(14.))
                                    .ml(px(1.))
                                    .bg(theme.accent),
                            )
                    }
                    ViMode::Search(dir) => {
                        let prefix = match dir {
                            SearchDirection::Forward => "/",
                            SearchDirection::Backward => "?",
                        };
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .child(
                                div()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme.accent)
                                    .child(prefix),
                            )
                            .child(
                                div()
                                    .text_color(theme.text_primary)
                                    .child(self.vi_state.search_buffer.clone()),
                            )
                            .child(
                                div()
                                    .w(px(2.))
                                    .h(px(14.))
                                    .ml(px(1.))
                                    .bg(theme.accent),
                            )
                    }
                    _ => {
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.accent)
                            .child(vi_status_text)
                    }
                };

                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .px(px(12.))
                        .py(px(4.))
                        .bg(theme.bg_column)
                        .border_t_1()
                        .border_color(theme.border)
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .child(status_content),
                )
            })
            // Bottom button bar — only shown in torn-off window (modal has its own)
            .when(self.is_torn_off, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .p(px(12.))
                        .border_t_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .px(px(16.))
                                .py(px(6.))
                                .rounded(px(4.))
                                .bg(theme.accent)
                                .text_color(rgb(0xffffff))
                                .text_sm()
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, window, cx| {
                                        this.on_save(&SaveEditor, window, cx);
                                    }),
                                )
                                .child(crate::i18n::t("editor.save", self.config.language)),
                        )
                        .child(
                            div()
                                .px(px(16.))
                                .py(px(6.))
                                .rounded(px(4.))
                                .bg(theme.bg_surface)
                                .border_1()
                                .border_color(theme.border)
                                .text_color(theme.text_primary)
                                .text_sm()
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|_, _, window, _cx| {
                                        window.remove_window();
                                    }),
                                )
                                .child(crate::i18n::t("editor.cancel", self.config.language)),
                        ),
                )
            })
            // Sub-item creation prompt dialog modal
            .children(if self.subitem_prompt_open {
                let prompt_text = self.subitem_prompt_text.clone();
                Some(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(rgba(0x00000066))
                        .flex()
                        .justify_center()
                        .items_center()
                        .child(
                            div()
                                .w(px(360.))
                                .p(px(16.))
                                .rounded(px(8.))
                                .bg(theme.bg_surface)
                                .border_1()
                                .border_color(theme.border)
                                .shadow_lg()
                                .flex()
                                .flex_col()
                                .gap(px(12.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.text_primary)
                                        .child("Create Sub-item"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_secondary)
                                        .child("Enter title or short phrase for the new sub-item:"),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .p(px(8.))
                                        .rounded(px(4.))
                                        .bg(theme.bg_column)
                                        .border_1()
                                        .border_color(theme.selection)
                                        .text_sm()
                                        .text_color(theme.text_primary)
                                        .child(if prompt_text.is_empty() {
                                            "Type sub-item title...".to_string()
                                        } else {
                                            prompt_text
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .justify_end()
                                        .gap(px(8.))
                                        .child(
                                            div()
                                                .px(px(12.))
                                                .py(px(6.))
                                                .rounded(px(4.))
                                                .bg(theme.accent)
                                                .text_color(rgb(0xffffff))
                                                .text_xs()
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(|this, _, window, cx| {
                                                        this.confirm_subitem_prompt(window, cx);
                                                    }),
                                                )
                                                .child("Create Link"),
                                        )
                                        .child(
                                            div()
                                                .px(px(12.))
                                                .py(px(6.))
                                                .rounded(px(4.))
                                                .bg(theme.bg_surface)
                                                .border_1()
                                                .border_color(theme.border)
                                                .text_color(theme.text_primary)
                                                .text_xs()
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.cancel_subitem_prompt(cx);
                                                    }),
                                                )
                                                .child("Cancel"),
                                        ),
                                ),
                        ),
                )
            } else {
                None
            })
            // Context menu overlay
            .children(if let Some(pos) = self.context_menu_pos {
                let has_selection = !self.state.selected_range().is_empty();
                let cursor_on_link = find_link_at_offset(self.state.content(), self.state.cursor_offset()).is_some();
                Some(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .size_full()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.context_menu_pos = None;
                                        cx.notify();
                                    }),
                                )
                                .on_mouse_down(
                                    MouseButton::Right,
                                    cx.listener(|this, _, _, cx| {
                                        this.context_menu_pos = None;
                                        cx.notify();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .absolute()
                                .left(pos.x)
                                .top(pos.y)
                                .w(px(240.))
                                .bg(theme.bg_surface)
                                .border_1()
                                .border_color(theme.border)
                                .rounded(px(6.))
                                .shadow_lg()
                                .p(px(4.))
                                .flex()
                                .flex_col()
                                .gap(px(2.))
                                .child(
                                    div()
                                        .px(px(8.))
                                        .py(px(6.))
                                        .rounded(px(4.))
                                        .text_sm()
                                        .text_color(theme.text_primary)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(theme.bg_column))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.on_create_sub_item(&EditorCreateSubItem, window, cx);
                                            }),
                                        )
                                        .child(if has_selection {
                                            "Create Sub-item from Selection (Cmd-K)"
                                        } else {
                                            "Create Sub-item... (Cmd-K)"
                                        }),
                                )
                                .children(if cursor_on_link {
                                    Some(
                                        div()
                                            .px(px(8.))
                                            .py(px(6.))
                                            .rounded(px(4.))
                                            .text_sm()
                                            .text_color(theme.text_primary)
                                            .cursor_pointer()
                                            .hover(|s| s.bg(theme.bg_column))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, window, cx| {
                                                    this.on_follow_link(&EditorFollowLink, window, cx);
                                                }),
                                            )
                                            .child("Follow Link (Cmd-Enter)"),
                                    )
                                } else {
                                    None
                                })
                                .child(
                                    div()
                                        .px(px(8.))
                                        .py(px(6.))
                                        .rounded(px(4.))
                                        .text_sm()
                                        .text_color(theme.text_primary)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(theme.bg_column))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.on_copy(&EditorCopy, window, cx);
                                                this.context_menu_pos = None;
                                            }),
                                        )
                                        .child("Copy"),
                                )
                                .child(
                                    div()
                                        .px(px(8.))
                                        .py(px(6.))
                                        .rounded(px(4.))
                                        .text_sm()
                                        .text_color(theme.text_primary)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(theme.bg_column))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.on_paste(&EditorPaste, window, cx);
                                                this.context_menu_pos = None;
                                            }),
                                        )
                                        .child("Paste"),
                                ),
                        ),
                )
            } else {
                None
            })
    }
}
