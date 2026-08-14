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
use crate::storage::{Location, Storage};
use crate::text_input::TextInputState;
use crate::theme::Theme;
use crate::vi::{ViMode, ViState};

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
}

impl ItemEditor {
    pub fn new(
        cx: &mut Context<Self>,
        initial: &str,
        editing_item_id: Option<Uuid>,
        is_new: bool,
        config: Config,
        is_torn_off: bool,
    ) -> Self {
        Self {
            state: TextInputState::new(initial),
            vi_state: ViState::new(),
            focus_handle: cx.focus_handle().tab_stop(true),
            editing_item_id,
            is_new,
            config,
            is_torn_off,
        }
    }

    pub fn content(&self) -> &str {
        self.state.content()
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
            }
        } else if let Some(id) = self.editing_item_id {
            let item = Item {
                id,
                body: content,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                completed_at: None,
            };
            let location = Location::Active(Category::Today);
            let _ = Storage::write_item(&self.config.data_dir, &item, &location);
        }
        cx.notify();
    }

    pub fn on_close(&mut self, _: &CloseWindow, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }

    pub fn on_backspace(&mut self, _: &EditorBackspace, _: &mut Window, cx: &mut Context<Self>) {
        self.state.backspace();
        cx.notify();
    }

    pub fn on_delete(&mut self, _: &EditorDelete, _: &mut Window, cx: &mut Context<Self>) {
        self.state.delete();
        cx.notify();
    }

    pub fn on_up(&mut self, _: &EditorUp, _: &mut Window, cx: &mut Context<Self>) {
        self.state.move_up();
        cx.notify();
    }

    pub fn on_down(&mut self, _: &EditorDown, _: &mut Window, cx: &mut Context<Self>) {
        self.state.move_down();
        cx.notify();
    }

    pub fn on_left(&mut self, _: &EditorLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.state.move_left();
        cx.notify();
    }

    pub fn on_right(&mut self, _: &EditorRight, _: &mut Window, cx: &mut Context<Self>) {
        self.state.move_right();
        cx.notify();
    }

    pub fn on_select_up(&mut self, _: &EditorSelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_up();
        cx.notify();
    }

    pub fn on_select_down(&mut self, _: &EditorSelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_down();
        cx.notify();
    }

    pub fn on_select_left(&mut self, _: &EditorSelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_left();
        cx.notify();
    }

    pub fn on_select_right(&mut self, _: &EditorSelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_right();
        cx.notify();
    }

    pub fn on_enter(&mut self, _: &EditorEnter, _: &mut Window, cx: &mut Context<Self>) {
        if self.config.vi_mode && self.vi_state.mode == ViMode::Normal {
            self.state.move_down();
        } else {
            self.state.insert("\n");
        }
        cx.notify();
    }

    pub fn on_escape(&mut self, _: &EditorEscape, _: &mut Window, cx: &mut Context<Self>) {
        if self.config.vi_mode {
            self.vi_state.mode = ViMode::Normal;
            self.vi_state.visual_anchor = None;
            self.vi_state.pending_op = None;
            if self.state.cursor_offset() > 0 && self.state.selected_range().is_empty() {
                let cur = self.state.cursor_offset();
                let line_start = self.state.find_line_start(cur);
                if cur > line_start {
                    self.state.move_left();
                }
            }
            cx.notify();
        }
    }

    pub fn on_select_all(&mut self, _: &EditorSelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_all();
        cx.notify();
    }

    pub fn on_undo(&mut self, _: &EditorUndo, _: &mut Window, cx: &mut Context<Self>) {
        self.state.undo();
        cx.notify();
    }

    pub fn on_redo(&mut self, _: &EditorRedo, _: &mut Window, cx: &mut Context<Self>) {
        self.state.redo();
        cx.notify();
    }

    pub fn on_paste(&mut self, _: &EditorPaste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.state.insert(&text);
            cx.notify();
        }
    }

    pub fn on_copy(&mut self, _: &EditorCopy, _: &mut Window, cx: &mut Context<Self>) {
        let range = self.state.selected_range();
        if !range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.state.content()[range].to_string(),
            ));
        }
    }

    pub fn on_cut(&mut self, _: &EditorCut, _: &mut Window, cx: &mut Context<Self>) {
        let range = self.state.selected_range();
        if !range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.state.content()[range].to_string(),
            ));
            self.state.insert("");
            cx.notify();
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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.config.vi_mode {
            let handled = self.vi_state.handle_key(new_text, &mut self.state);
            if !handled && self.vi_state.mode == ViMode::Insert {
                let range = range_utf16
                    .as_ref()
                    .map(|r| self.state.range_from_utf16(r))
                    .unwrap_or_else(|| self.state.selected_range());
                self.state.replace_range(range, new_text);
            }
        } else {
            let range = range_utf16
                .as_ref()
                .map(|r| self.state.range_from_utf16(r))
                .unwrap_or_else(|| self.state.selected_range());
            self.state.replace_range(range, new_text);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.config.vi_mode {
            let handled = self.vi_state.handle_key(new_text, &mut self.state);
            if !handled && self.vi_state.mode == ViMode::Insert {
                let range = range_utf16
                    .as_ref()
                    .map(|r| self.state.range_from_utf16(r))
                    .unwrap_or_else(|| self.state.selected_range());
                self.state.replace_range(range, new_text);
            }
        } else {
            let range = range_utf16
                .as_ref()
                .map(|r| self.state.range_from_utf16(r))
                .unwrap_or_else(|| self.state.selected_range());
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
    selection: Option<gpui::PaintQuad>,
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
        let cursor_offset = editor.state.cursor_offset();
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let wrap_width = bounds.size.width;

        if content.is_empty() {
            return EditorPrepaintState {
                lines: Vec::new(),
                cursor: Some(gpui::fill(
                    gpui::Bounds::new(
                        Point::new(bounds.left(), bounds.top()),
                        gpui::size(px(2.), line_height),
                    ),
                    gpui::blue(),
                )),
                selection: None,
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
                cursor: None,
                selection: None,
            };
        };

        let cursor = if let Some(first_line) = lines.first() {
            let cursor_pos = first_line.position_for_index(cursor_offset, line_height);
            cursor_pos.map(|p| {
                gpui::fill(
                    gpui::Bounds::new(
                        Point::new(bounds.left() + p.x, bounds.top() + p.y),
                        gpui::size(px(2.), line_height),
                    ),
                    gpui::blue(),
                )
            })
        } else {
            None
        };

        let selection = if !selected_range.is_empty() {
            if let Some(first_line) = lines.first() {
                let start_x = first_line
                    .position_for_index(selected_range.start, line_height)
                    .map(|p| p.x)
                    .unwrap_or(px(0.));
                let end_x = first_line
                    .position_for_index(selected_range.end, line_height)
                    .map(|p| p.x)
                    .unwrap_or(bounds.size.width);
                Some(gpui::fill(
                    gpui::Bounds::from_corners(
                        Point::new(bounds.left() + start_x, bounds.top()),
                        Point::new(bounds.left() + end_x, bounds.top() + line_height),
                    ),
                    rgba(0x3311ff30),
                ))
            } else {
                None
            }
        } else {
            None
        };

        EditorPrepaintState {
            lines: lines.into_iter().collect(),
            cursor,
            selection,
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

        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }

        let line_height = window.line_height();
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

        let content = self.state.content();
        let line_count = content.lines().count().max(1);

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
                .text_sm()
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
            ViMode::Normal => "-- NORMAL --",
            ViMode::Insert => "-- INSERT --",
            ViMode::Visual => "-- VISUAL --",
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_surface)
            .track_focus(&self.focus_handle)
            .key_context("ItemEditor")
            .cursor(gpui::CursorStyle::IBeam)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.as_str();
                if (key == "escape" || key == "Esc" || key == "Escape" || key == "\x1b")
                    && this.config.vi_mode
                {
                    this.vi_state.mode = ViMode::Normal;
                    this.vi_state.visual_anchor = None;
                    this.vi_state.pending_op = None;
                    if this.state.cursor_offset() > 0 && this.state.selected_range().is_empty() {
                        let cur = this.state.cursor_offset();
                        let line_start = this.state.find_line_start(cur);
                        if cur > line_start {
                            this.state.move_left();
                        }
                    }
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
            // Editor main area (gutter + text)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .children(gutter)
                    .child(
                        div()
                            .flex_1()
                            .p(px(16.))
                            .text_sm()
                            .text_color(theme.text_primary)
                            .child(EditorElement {
                                editor: cx.entity(),
                            }),
                    ),
            )
            // Vi Mode Status Bar
            .when(show_vi_status, |this| {
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
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.accent)
                                .child(vi_status_text),
                        ),
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
                                .child("Save"),
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
                                .child("Cancel"),
                        ),
                )
            })
    }
}
