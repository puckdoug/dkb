use std::ops::Range;

use gpui::{
    App, Context, FocusHandle, Focusable, GlobalElementId, KeyBinding, Menu, MenuItem, Render,
    Window, actions, div, prelude::*, px, rgb,
    ClipboardItem, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler,
    InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent, Pixels, Point,
    SharedString, Style, TextRun, TextAlign, UTF16Selection,
    relative, rgba,
};

use crate::board::Board;
use crate::config::Config;
use crate::item::{Category, Item, Status};
use crate::storage::{Location, Storage};
use crate::text_input::TextInputState;
use uuid::Uuid;

actions!(
    dkb,
    [
        NewItem,
        CloseWindow,
        Quit,
        MoveToBacklog,
        MoveToYesterday,
        MoveToToday,
        MoveToThisWeek,
        MoveToNextWeek,
        ToggleDone,
        DeleteItem,
        ShowBacklog,
        ShowActive,
        ShowDone,
        NextItem,
        PrevItem,
        SaveEditor,
        CancelEditor,
        TearOffEditor,
    ]
);

actions!(
    dkb_editor,
    [
        EditorBackspace,
        EditorDelete,
        EditorLeft,
        EditorRight,
        EditorSelectLeft,
        EditorSelectRight,
        EditorSelectAll,
        EditorUndo,
        EditorRedo,
        EditorPaste,
        EditorCut,
        EditorCopy,
    ]
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Backlog,
    Active,
    Done,
}

// -- ItemEditor --

pub struct ItemEditor {
    pub state: TextInputState,
    pub focus_handle: FocusHandle,
    pub editing_item_id: Option<Uuid>,
}

impl ItemEditor {
    pub fn new(cx: &mut Context<Self>, initial: &str, editing_item_id: Option<Uuid>) -> Self {
        Self {
            state: TextInputState::new(initial),
            focus_handle: cx.focus_handle().tab_stop(true),
            editing_item_id,
        }
    }

    pub fn content(&self) -> &str {
        self.state.content()
    }

    fn on_backspace(&mut self, _: &EditorBackspace, _: &mut Window, cx: &mut Context<Self>) {
        self.state.backspace();
        cx.notify();
    }

    fn on_delete(&mut self, _: &EditorDelete, _: &mut Window, cx: &mut Context<Self>) {
        self.state.delete();
        cx.notify();
    }

    fn on_left(&mut self, _: &EditorLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.state.move_left();
        cx.notify();
    }

    fn on_right(&mut self, _: &EditorRight, _: &mut Window, cx: &mut Context<Self>) {
        self.state.move_right();
        cx.notify();
    }

    fn on_select_left(&mut self, _: &EditorSelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_left();
        cx.notify();
    }

    fn on_select_right(&mut self, _: &EditorSelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_right();
        cx.notify();
    }

    fn on_select_all(&mut self, _: &EditorSelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_all();
        cx.notify();
    }

    fn on_undo(&mut self, _: &EditorUndo, _: &mut Window, cx: &mut Context<Self>) {
        self.state.undo();
        cx.notify();
    }

    fn on_redo(&mut self, _: &EditorRedo, _: &mut Window, cx: &mut Context<Self>) {
        self.state.redo();
        cx.notify();
    }

    fn on_paste(&mut self, _: &EditorPaste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.state.insert(&text);
            cx.notify();
        }
    }

    fn on_copy(&mut self, _: &EditorCopy, _: &mut Window, cx: &mut Context<Self>) {
        let range = self.state.selected_range();
        if !range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.state.content()[range].to_string(),
            ));
        }
    }

    fn on_cut(&mut self, _: &EditorCut, _: &mut Window, cx: &mut Context<Self>) {
        let range = self.state.selected_range();
        if !range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
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
    ) -> Option<UTF16Selection> {
        let range = self.state.selected_range();
        Some(UTF16Selection {
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
        let range = range_utf16
            .as_ref()
            .map(|r| self.state.range_from_utf16(r))
            .unwrap_or_else(|| self.state.selected_range());
        self.state.replace_range(range, new_text);
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
        let range = range_utf16
            .as_ref()
            .map(|r| self.state.range_from_utf16(r))
            .unwrap_or_else(|| self.state.selected_range());
        self.state.replace_range(range, new_text);
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

// -- Custom Element for text input rendering --
// This is REQUIRED for GPUI to route keyboard input to the EntityInputHandler.
// Without calling window.handle_input() during paint, no text entry occurs.

struct EditorElement {
    editor: Entity<ItemEditor>,
}

struct EditorPrepaintState {
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

        // Calculate cursor position from the first line
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

        // Selection highlight on first line (simplified for multi-line)
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

        // THIS IS THE CRITICAL CALL — registers with GPUI's input system
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xffffff))
            .track_focus(&self.focus_handle)
            .key_context("ItemEditor")
            .cursor(gpui::CursorStyle::IBeam)
            .on_action(cx.listener(Self::on_backspace))
            .on_action(cx.listener(Self::on_delete))
            .on_action(cx.listener(Self::on_left))
            .on_action(cx.listener(Self::on_right))
            .on_action(cx.listener(Self::on_select_left))
            .on_action(cx.listener(Self::on_select_right))
            .on_action(cx.listener(Self::on_select_all))
            .on_action(cx.listener(Self::on_undo))
            .on_action(cx.listener(Self::on_redo))
            .on_action(cx.listener(Self::on_paste))
            .on_action(cx.listener(Self::on_copy))
            .on_action(cx.listener(Self::on_cut))
            .child(
                div()
                    .flex_1()
                    .p(px(16.))
                    .text_sm()
                    .text_color(rgb(0x333333))
                    .child(EditorElement {
                        editor: cx.entity(),
                    }),
            )
    }
}

// -- Editing state --

pub struct EditingState {
    pub editor: gpui::Entity<ItemEditor>,
    pub is_new: bool,
    pub item_id: Option<Uuid>,
}

// -- Simple tooltip view --

struct TextTooltip {
    text: SharedString,
}

impl Render for TextTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(6.))
            .py(px(3.))
            .rounded(px(3.))
            .bg(rgb(0x333333))
            .text_xs()
            .text_color(rgb(0xffffff))
            .child(self.text.clone())
    }
}

// -- KanbanView --

pub struct KanbanView {
    pub board: Board,
    pub current_screen: Screen,
    pub config: Config,
    pub focus_handle: FocusHandle,
    pub selected_item: Option<Uuid>,
    pub editing: Option<EditingState>,
}

impl KanbanView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let config = Config::load().unwrap_or_else(|_| Config {
            data_dir: Config::default_data_dir(),
        });
        Storage::init(&config.data_dir).ok();
        let board = Storage::load_board(&config.data_dir).unwrap_or_default();

        Self {
            board,
            current_screen: Screen::Active,
            config,
            focus_handle: cx.focus_handle(),
            selected_item: None,
            editing: None,
        }
    }

    pub fn key_bindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding::new("cmd-n", NewItem, None),
            KeyBinding::new("cmd-w", CloseWindow, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-shift-b", ShowBacklog, None),
            KeyBinding::new("cmd-shift-a", ShowActive, None),
            KeyBinding::new("cmd-shift-d", ShowDone, None),
            KeyBinding::new("cmd-1", MoveToYesterday, None),
            KeyBinding::new("cmd-2", MoveToToday, None),
            KeyBinding::new("cmd-3", MoveToThisWeek, None),
            KeyBinding::new("cmd-4", MoveToNextWeek, None),
            KeyBinding::new("cmd-b", MoveToBacklog, None),
            KeyBinding::new("cmd-d", ToggleDone, None),
            KeyBinding::new("delete", DeleteItem, None),
            KeyBinding::new("tab", NextItem, None),
            KeyBinding::new("shift-tab", PrevItem, None),
            KeyBinding::new("escape", CancelEditor, Some("ItemEditor")),
            KeyBinding::new("cmd-s", SaveEditor, Some("ItemEditor")),
            KeyBinding::new("backspace", EditorBackspace, Some("ItemEditor")),
            KeyBinding::new("delete", EditorDelete, Some("ItemEditor")),
            KeyBinding::new("left", EditorLeft, Some("ItemEditor")),
            KeyBinding::new("right", EditorRight, Some("ItemEditor")),
            KeyBinding::new("shift-left", EditorSelectLeft, Some("ItemEditor")),
            KeyBinding::new("shift-right", EditorSelectRight, Some("ItemEditor")),
            KeyBinding::new("cmd-a", EditorSelectAll, Some("ItemEditor")),
            KeyBinding::new("cmd-v", EditorPaste, Some("ItemEditor")),
            KeyBinding::new("cmd-c", EditorCopy, Some("ItemEditor")),
            KeyBinding::new("cmd-x", EditorCut, Some("ItemEditor")),
            KeyBinding::new("cmd-z", EditorUndo, Some("ItemEditor")),
            KeyBinding::new("cmd-shift-z", EditorRedo, Some("ItemEditor")),
        ]
    }

    pub fn menus() -> Vec<Menu> {
        vec![
            Menu::new("dkb").items([
                MenuItem::action("Quit", Quit),
            ]),
            Menu::new("File").items([
                MenuItem::action("New Item", NewItem),
                MenuItem::separator(),
                MenuItem::action("Close Window", CloseWindow),
            ]),
            Menu::new("View").items([
                MenuItem::action("Backlog", ShowBacklog),
                MenuItem::action("Active", ShowActive),
                MenuItem::action("Done", ShowDone),
            ]),
            Menu::new("Item").items([
                MenuItem::action("Move to Backlog", MoveToBacklog),
                MenuItem::action("Move to Yesterday", MoveToYesterday),
                MenuItem::action("Move to Today", MoveToToday),
                MenuItem::action("Move to This Week", MoveToThisWeek),
                MenuItem::action("Move to Next Week", MoveToNextWeek),
                MenuItem::separator(),
                MenuItem::action("Mark Done / Reopen", ToggleDone),
                MenuItem::separator(),
                MenuItem::action("Delete", DeleteItem),
            ]),
        ]
    }

    pub fn setup_menus(cx: &mut App) {
        cx.bind_keys(Self::key_bindings());
        cx.set_menus(Self::menus());
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
    }
}

impl Focusable for KanbanView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KanbanView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let screen = self.current_screen;
        // Focus the editor when it opens
        if let Some(ref editing) = self.editing {
            let focus = editing.editor.read(cx).focus_handle.clone();
            _window.focus(&focus, cx);
        }
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xf5f5f5))
            .size_full()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_show_backlog))
            .on_action(cx.listener(Self::on_show_active))
            .on_action(cx.listener(Self::on_show_done))
            .on_action(cx.listener(Self::on_close_window))
            .on_action(cx.listener(Self::on_move_to_yesterday))
            .on_action(cx.listener(Self::on_move_to_today))
            .on_action(cx.listener(Self::on_move_to_this_week))
            .on_action(cx.listener(Self::on_move_to_next_week))
            .on_action(cx.listener(Self::on_move_to_backlog))
            .on_action(cx.listener(Self::on_toggle_done))
            .on_action(cx.listener(Self::on_delete_item))
            .on_action(cx.listener(Self::on_new_item))
            .on_action(cx.listener(Self::on_next_item))
            .on_action(cx.listener(Self::on_prev_item))
            .on_action(cx.listener(Self::on_save_editor))
            .on_action(cx.listener(Self::on_cancel_editor))
            .on_action(cx.listener(Self::on_tear_off_editor))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .p(px(8.))
                    .bg(rgb(0xe0e0e0))
                    .child(self.render_tab("Backlog", Screen::Backlog, cx))
                    .child(self.render_tab("Active", Screen::Active, cx))
                    .child(self.render_tab("Done", Screen::Done, cx)),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(match screen {
                        Screen::Backlog => self.render_backlog_screen(cx).into_any_element(),
                        Screen::Active => self.render_active_screen(cx).into_any_element(),
                        Screen::Done => self.render_done_screen(cx).into_any_element(),
                    }),
            )
            .child(self.render_editor_modal(cx))
    }
}

impl KanbanView {
    fn on_show_backlog(&mut self, _: &ShowBacklog, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_screen = Screen::Backlog;
        cx.notify();
    }

    fn on_show_active(&mut self, _: &ShowActive, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_screen = Screen::Active;
        cx.notify();
    }

    fn on_show_done(&mut self, _: &ShowDone, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_screen = Screen::Done;
        cx.notify();
    }

    fn on_close_window(&mut self, _: &CloseWindow, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }

    fn move_selected_to(&mut self, location: Location, cx: &mut Context<Self>) {
        let Some(id) = self.selected_item else {
            return;
        };
        if !self.board.can_move(&id, &location) {
            return;
        }
        let Some(from) = self.board.find_item_location(&id) else {
            return;
        };
        match Storage::move_item(&self.config.data_dir, &id, &from, &location) {
            Ok(updated_item) => {
                let _ = self.board.remove_item(&id, &from);
                self.board.insert_item(updated_item, &location);
                cx.notify();
            }
            Err(e) => {
                eprintln!("Failed to move item: {}", e);
            }
        }
    }

    fn on_move_to_yesterday(&mut self, _: &MoveToYesterday, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_selected_to(Location::Active(Category::Yesterday), cx);
    }

    fn on_move_to_today(&mut self, _: &MoveToToday, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_selected_to(Location::Active(Category::Today), cx);
    }

    fn on_move_to_this_week(&mut self, _: &MoveToThisWeek, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_selected_to(Location::Active(Category::ThisWeek), cx);
    }

    fn on_move_to_next_week(&mut self, _: &MoveToNextWeek, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_selected_to(Location::Active(Category::NextWeek), cx);
    }

    fn on_move_to_backlog(&mut self, _: &MoveToBacklog, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_selected_to(Location::Backlog, cx);
    }

    fn on_toggle_done(&mut self, _: &ToggleDone, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected_item else {
            return;
        };
        let Some(location) = self.board.find_item_location(&id) else {
            return;
        };
        let target = match location.status() {
            Status::Active => Location::Done,
            Status::Done => Location::Active(Category::Today),
            Status::Backlog => return,
        };
        self.move_selected_to(target, cx);
    }

    fn on_delete_item(&mut self, _: &DeleteItem, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected_item else {
            return;
        };
        let Some(location) = self.board.find_item_location(&id) else {
            return;
        };
        if Storage::delete_item(&self.config.data_dir, &id, &location).is_ok() {
            let _ = self.board.remove_item(&id, &location);
            self.selected_item = None;
            cx.notify();
        }
    }

    // -- Item editor --

    fn on_new_item(&mut self, _: &NewItem, _window: &mut Window, cx: &mut Context<Self>) {
        let editor = cx.new(|cx| ItemEditor::new(cx, "", None));
        self.editing = Some(EditingState {
            editor,
            is_new: true,
            item_id: None,
        });
        cx.notify();
    }

    fn open_editor_for_item(&mut self, id: Uuid, cx: &mut Context<Self>) {
        let body = self.board.find_item(&id).map(|i| i.body.clone()).unwrap_or_default();
        let editor = cx.new(|cx| ItemEditor::new(cx, &body, Some(id)));
        self.editing = Some(EditingState {
            editor,
            is_new: false,
            item_id: Some(id),
        });
        cx.notify();
    }

    fn on_save_editor(&mut self, _: &SaveEditor, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        let content = editing.editor.read(cx).content().to_string();
        let title = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("").to_string();
        if title.is_empty() {
            cx.notify();
            return;
        }

        if editing.is_new {
            let item = Item::new(&content);
            let location = match self.current_screen {
                Screen::Backlog => Location::Backlog,
                Screen::Active => Location::Active(Category::Today),
                Screen::Done => Location::Backlog,
            };
            if Storage::write_item(&self.config.data_dir, &item, &location).is_ok() {
                self.board.insert_item(item, &location);
            }
        } else if let Some(id) = editing.item_id
            && let Some(location) = self.board.find_item_location(&id) {
            if let Some(item) = self.board.find_item_mut(&id) {
                item.body = content;
                item.updated_at = chrono::Utc::now();
            }
            let item_ref = self.board.find_item(&id).cloned();
            if let Some(item) = item_ref {
                let _ = Storage::write_item(&self.config.data_dir, &item, &location);
            }
        }
        cx.notify();
    }

    fn on_cancel_editor(&mut self, _: &CancelEditor, _window: &mut Window, cx: &mut Context<Self>) {
        self.editing = None;
        cx.notify();
    }

    fn on_tear_off_editor(&mut self, _: &TearOffEditor, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        let content = editing.editor.read(cx).content().to_string();
        let item_id = editing.item_id;

        let opts = gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                gpui::size(px(600.), px(400.)),
                cx,
            ))),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Edit Item".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        let _ = cx.open_window(opts, |_, cx| {
            cx.new(|cx| ItemEditor::new(cx, &content, item_id))
        });
        cx.notify();
    }

    // -- Keyboard navigation --

    fn current_screen_items(&self) -> Vec<Uuid> {
        match self.current_screen {
            Screen::Backlog => self.board.backlog.iter().map(|i| i.id).collect(),
            Screen::Active => {
                let mut items = Vec::new();
                items.extend(self.board.active.yesterday.iter().map(|i| i.id));
                items.extend(self.board.active.today.iter().map(|i| i.id));
                items.extend(self.board.active.this_week.iter().map(|i| i.id));
                items.extend(self.board.active.next_week.iter().map(|i| i.id));
                items
            }
            Screen::Done => self.board.done.iter().map(|i| i.id).collect(),
        }
    }

    fn on_next_item(&mut self, _: &NextItem, _window: &mut Window, cx: &mut Context<Self>) {
        let items = self.current_screen_items();
        if items.is_empty() {
            return;
        }
        let next = match self.selected_item {
            None => items[0],
            Some(current) => {
                let pos = items.iter().position(|id| *id == current);
                match pos {
                    None => items[0],
                    Some(idx) => items[(idx + 1) % items.len()],
                }
            }
        };
        self.selected_item = Some(next);
        cx.notify();
    }

    fn on_prev_item(&mut self, _: &PrevItem, _window: &mut Window, cx: &mut Context<Self>) {
        let items = self.current_screen_items();
        if items.is_empty() {
            return;
        }
        let prev = match self.selected_item {
            None => items[items.len() - 1],
            Some(current) => {
                let pos = items.iter().position(|id| *id == current);
                match pos {
                    None => items[items.len() - 1],
                    Some(idx) => {
                        let len = items.len();
                        items[(idx + len - 1) % len]
                    }
                }
            }
        };
        self.selected_item = Some(prev);
        cx.notify();
    }

    // -- Rendering --

    fn render_tab(&self, label: &str, screen: Screen, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = self.current_screen == screen;
        div()
            .px(px(12.))
            .py(px(6.))
            .rounded(px(4.))
            .bg(if is_active { rgb(0xffffff) } else { rgb(0xe0e0e0) })
            .text_sm()
            .text_color(rgb(0x333333))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _window, cx| {
                    this.current_screen = screen;
                    cx.notify();
                }),
            )
            .child(label.to_string())
    }

    fn render_item_card(&self, item: &Item, cx: &mut Context<Self>) -> impl IntoElement {
        let is_selected = self.selected_item == Some(item.id);
        let item_id = item.id;
        div()
            .w_full()
            .p(px(8.))
            .mb(px(4.))
            .rounded(px(4.))
            .bg(if is_selected { rgb(0xe3f2fd) } else { rgb(0xffffff) })
            .border_1()
            .border_color(if is_selected { rgb(0x2196f3) } else { rgb(0xdddddd) })
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    this.selected_item = Some(item_id);
                    if event.click_count >= 2 {
                        this.open_editor_for_item(item_id, cx);
                    }
                    cx.notify();
                }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x333333))
                    .child(item.title()),
            )
    }

    fn render_column(
        &self,
        title: &str,
        items: &[Item],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let item_count = items.len();
        div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(rgb(0xeceff1))
            .rounded(px(4.))
            .p(px(8.))
            .m(px(4.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .mb(px(8.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x37474f))
                            .child(title.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x78909c))
                            .child(format!("{}", item_count)),
                    ),
            )
            .children({
                let mut cards: Vec<gpui::AnyElement> = Vec::new();
                for item in items {
                    cards.push(self.render_item_card(item, cx).into_any_element());
                }
                cards
            })
    }

    fn render_backlog_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_1()
            .p(px(8.))
            .child(self.render_column("Backlog", &self.board.backlog, cx))
    }

    fn render_active_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_1()
            .p(px(4.))
            .child(self.render_column("Yesterday", &self.board.active.yesterday, cx))
            .child(self.render_column("Today", &self.board.active.today, cx))
            .child(self.render_column("This Week", &self.board.active.this_week, cx))
            .child(self.render_column("Next Week", &self.board.active.next_week, cx))
    }

    fn render_done_screen(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_1()
            .p(px(8.))
            .child(self.render_column("Done", &self.board.done, cx))
    }

    fn render_editor_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(ref editing) = self.editing {
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .bg(rgba(0x00000040))
                .flex()
                .justify_center()
                .items_center()
                .child(
                    div()
                        .w(px(600.))
                        .h(px(400.))
                        .bg(rgb(0xffffff))
                        .rounded(px(8.))
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        // Top bar with tear-off button (top-right corner)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_end()
                                .p(px(4.))
                                .bg(rgb(0xf5f5f5))
                                .border_b_1()
                                .border_color(rgb(0xe0e0e0))
                                .child(
                                    div()
                                        .id("tear-off")
                                        .px(px(8.))
                                        .py(px(4.))
                                        .cursor_pointer()
                                        .text_color(rgb(0x666666))
                                        .child("\u{29C9}")
                                        .tooltip(move |_, cx| {
                                            cx.new(|_| TextTooltip {
                                                text: "tear off window".into(),
                                            })
                                            .into()
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.on_tear_off_editor(&TearOffEditor, window, cx);
                                            }),
                                        ),
                                ),
                        )
                        // Editor content area
                        .child(
                            div()
                                .flex_1()
                                .child(editing.editor.clone()),
                        )
                        // Bottom button bar: Save and Cancel
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(8.))
                                .p(px(12.))
                                .border_t_1()
                                .border_color(rgb(0xe0e0e0))
                                .child(
                                    div()
                                        .px(px(16.))
                                        .py(px(6.))
                                        .rounded(px(4.))
                                        .bg(rgb(0x4488ff))
                                        .text_color(rgb(0xffffff))
                                        .text_sm()
                                        .cursor_pointer()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.on_save_editor(&SaveEditor, window, cx);
                                            }),
                                        )
                                        .child("Save"),
                                )
                                .child(
                                    div()
                                        .px(px(16.))
                                        .py(px(6.))
                                        .rounded(px(4.))
                                        .bg(rgb(0xffffff))
                                        .border_1()
                                        .border_color(rgb(0xcccccc))
                                        .text_color(rgb(0x333333))
                                        .text_sm()
                                        .cursor_pointer()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.on_cancel_editor(&CancelEditor, window, cx);
                                            }),
                                        )
                                        .child("Cancel"),
                                ),
                        ),
                )
        } else {
            div()
        }
    }
}
