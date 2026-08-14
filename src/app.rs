use gpui::{
    actions, div, prelude::*, px, rgb, rgba, App, Context, FocusHandle,
    Focusable, IntoElement, KeyBinding, Menu, MenuItem, MouseButton, MouseDownEvent, Render,
    SharedString, Window,
};

use crate::board::Board;
use crate::config::{Config, ThemeMode};
use crate::editor::{
    CloseWindow, EditorBackspace, EditorCopy, EditorCut, EditorDelete, EditorDown, EditorEnter,
    EditorEscape, EditorLeft, EditorPaste, EditorRedo, EditorRight, EditorSelectAll,
    EditorSelectDown, EditorSelectLeft, EditorSelectRight, EditorSelectUp, EditorUndo, EditorUp,
    ItemEditor, SaveEditor,
};
use crate::item::{Category, Item, Status};
use crate::link::count_recursive_subitems;
use crate::storage::{Location, Storage};
use crate::theme::Theme;
use uuid::Uuid;

actions!(
    dkb,
    [
        NewItem,
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
        CancelEditor,
        TearOffEditor,
        NextColumn,
        PrevColumn,
        NavUp,
        NavDown,
        NavLeft,
        NavRight,
        OpenSettings,
        OpenSelectedForEdit,
        CreateSubItem,
        DrillDownSubItem,
        DrillUpBreadcrumb,
    ]
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Backlog,
    Active,
    Done,
    Settings,
}

// -- Drag Payload --

#[derive(Clone, Copy, Debug)]
pub struct DraggedCard {
    pub id: Uuid,
}

impl Render for DraggedCard {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(4.))
            .bg(rgb(0x4488ff))
            .text_color(rgb(0xffffff))
            .text_sm()
            .shadow_md()
            .child("Moving Card")
    }
}

// -- Editing state --

pub struct EditingState {
    pub editor: gpui::Entity<ItemEditor>,
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
    pub drill_down_stack: Vec<Uuid>,
}

impl KanbanView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let config = Config::load().unwrap_or_else(|_| Config {
            data_dir: Config::default_data_dir(),
            vi_mode: false,
            line_numbers: false,
            theme_mode: ThemeMode::System,
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
            drill_down_stack: Vec::new(),
        }
    }

    pub fn key_bindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding::new("cmd-n", NewItem, Some("KanbanView")),
            KeyBinding::new("cmd-shift-n", CreateSubItem, Some("KanbanView")),
            KeyBinding::new("cmd-w", CloseWindow, Some("KanbanView")),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-,", OpenSettings, None),
            KeyBinding::new("cmd-shift-b", ShowBacklog, Some("KanbanView")),
            KeyBinding::new("cmd-shift-a", ShowActive, Some("KanbanView")),
            KeyBinding::new("cmd-shift-d", ShowDone, Some("KanbanView")),
            KeyBinding::new("cmd-]", NextColumn, Some("KanbanView")),
            KeyBinding::new("cmd-[", PrevColumn, Some("KanbanView")),
            KeyBinding::new("enter", OpenSelectedForEdit, Some("KanbanView")),
            KeyBinding::new("cmd-right", DrillDownSubItem, Some("KanbanView")),
            KeyBinding::new("cmd-left", DrillUpBreadcrumb, Some("KanbanView")),
            // Arrow navigation
            KeyBinding::new("up", NavUp, Some("KanbanView")),
            KeyBinding::new("down", NavDown, Some("KanbanView")),
            KeyBinding::new("left", NavLeft, Some("KanbanView")),
            KeyBinding::new("right", NavRight, Some("KanbanView")),
            // Vi navigation (when not editing)
            KeyBinding::new("k", NavUp, Some("KanbanView")),
            KeyBinding::new("j", NavDown, Some("KanbanView")),
            KeyBinding::new("h", NavLeft, Some("KanbanView")),
            KeyBinding::new("l", NavRight, Some("KanbanView")),
            // Move item shortcuts
            KeyBinding::new("cmd-1", MoveToYesterday, Some("KanbanView")),
            KeyBinding::new("cmd-2", MoveToToday, Some("KanbanView")),
            KeyBinding::new("cmd-3", MoveToThisWeek, Some("KanbanView")),
            KeyBinding::new("cmd-4", MoveToNextWeek, Some("KanbanView")),
            KeyBinding::new("cmd-b", MoveToBacklog, Some("KanbanView")),
            KeyBinding::new("cmd-d", ToggleDone, Some("KanbanView")),
            KeyBinding::new("delete", DeleteItem, Some("KanbanView")),
            KeyBinding::new("backspace", DeleteItem, Some("KanbanView")),
            KeyBinding::new("tab", NextItem, Some("KanbanView")),
            KeyBinding::new("shift-tab", PrevItem, Some("KanbanView")),
            // Editor keybindings
            KeyBinding::new("cmd-s", SaveEditor, Some("ItemEditor")),
            KeyBinding::new("cmd-w", CloseWindow, Some("ItemEditor")),
            KeyBinding::new("up", EditorUp, Some("ItemEditor")),
            KeyBinding::new("down", EditorDown, Some("ItemEditor")),
            KeyBinding::new("left", EditorLeft, Some("ItemEditor")),
            KeyBinding::new("right", EditorRight, Some("ItemEditor")),
            KeyBinding::new("shift-up", EditorSelectUp, Some("ItemEditor")),
            KeyBinding::new("shift-down", EditorSelectDown, Some("ItemEditor")),
            KeyBinding::new("shift-left", EditorSelectLeft, Some("ItemEditor")),
            KeyBinding::new("shift-right", EditorSelectRight, Some("ItemEditor")),
            KeyBinding::new("cmd-a", EditorSelectAll, Some("ItemEditor")),
            KeyBinding::new("cmd-v", EditorPaste, Some("ItemEditor")),
            KeyBinding::new("cmd-c", EditorCopy, Some("ItemEditor")),
            KeyBinding::new("cmd-x", EditorCut, Some("ItemEditor")),
            KeyBinding::new("cmd-z", EditorUndo, Some("ItemEditor")),
            KeyBinding::new("cmd-shift-z", EditorRedo, Some("ItemEditor")),
            KeyBinding::new("backspace", EditorBackspace, Some("ItemEditor")),
            KeyBinding::new("delete", EditorDelete, Some("ItemEditor")),
            KeyBinding::new("enter", EditorEnter, Some("ItemEditor")),
            KeyBinding::new("escape", EditorEscape, Some("ItemEditor")),
        ]
    }

    pub fn menus() -> Vec<Menu> {
        vec![
            Menu::new("dkb").items([
                MenuItem::action("Settings...", OpenSettings),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ]),
            Menu::new("File").items([
                MenuItem::action("New Item", NewItem),
                MenuItem::action("New Sub-Item", CreateSubItem),
                MenuItem::separator(),
                MenuItem::action("Close Window", CloseWindow),
            ]),
            Menu::new("View").items([
                MenuItem::action("Backlog", ShowBacklog),
                MenuItem::action("Active", ShowActive),
                MenuItem::action("Done", ShowDone),
                MenuItem::separator(),
                MenuItem::action("Next Column", NextColumn),
                MenuItem::action("Previous Column", PrevColumn),
            ]),
            Menu::new("Item").items([
                MenuItem::action("Open / Edit", OpenSelectedForEdit),
                MenuItem::separator(),
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

    fn theme(&self, cx: &App) -> Theme {
        let system_dark = matches!(
            cx.window_appearance(),
            gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
        );
        Theme::resolve(self.config.theme_mode, system_dark)
    }
}

impl Focusable for KanbanView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KanbanView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme(cx);
        let screen = self.current_screen;

        // Focus the editor when it opens
        if let Some(ref editing) = self.editing {
            let focus = editing.editor.read(cx).focus_handle.clone();
            window.focus(&focus, cx);
        }

        div()
            .flex()
            .flex_col()
            .bg(theme.bg_window)
            .text_color(theme.text_primary)
            .size_full()
            .track_focus(&self.focus_handle)
            .when(self.editing.is_none(), |this| this.key_context("KanbanView"))
            .on_action(cx.listener(Self::on_show_backlog))
            .on_action(cx.listener(Self::on_show_active))
            .on_action(cx.listener(Self::on_show_done))
            .on_action(cx.listener(Self::on_open_settings))
            .on_action(cx.listener(Self::on_next_column))
            .on_action(cx.listener(Self::on_prev_column))
            .on_action(cx.listener(Self::on_nav_up))
            .on_action(cx.listener(Self::on_nav_down))
            .on_action(cx.listener(Self::on_nav_left))
            .on_action(cx.listener(Self::on_nav_right))
            .on_action(cx.listener(Self::on_open_selected_for_edit))
            .on_action(cx.listener(Self::on_create_sub_item))
            .on_action(cx.listener(Self::on_drill_down_sub_item))
            .on_action(cx.listener(Self::on_drill_up_breadcrumb))
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
            // Tab bar header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .px(px(8.))
                    .py(px(6.))
                    .bg(theme.bg_tab_bar)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(4.))
                            .child(self.render_tab("Backlog", Screen::Backlog, theme, cx))
                            .child(self.render_tab("Active", Screen::Active, theme, cx))
                            .child(self.render_tab("Done", Screen::Done, theme, cx)),
                    )
                    .child(
                        self.render_tab("Settings ⌘,", Screen::Settings, theme, cx),
                    ),
            )
            // Breadcrumbs bar (if drilled down)
            .children(if !self.drill_down_stack.is_empty() {
                Some(self.render_breadcrumbs(theme, cx))
            } else {
                None
            })
            // Main content area
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(match screen {
                        Screen::Backlog => self.render_backlog_screen(theme, cx).into_any_element(),
                        Screen::Active => self.render_active_screen(theme, cx).into_any_element(),
                        Screen::Done => self.render_done_screen(theme, cx).into_any_element(),
                        Screen::Settings => self.render_settings_screen(theme, cx).into_any_element(),
                    }),
            )
            // Editor modal
            .child(self.render_editor_modal(theme, cx))
    }
}

impl KanbanView {
    fn on_show_backlog(&mut self, _: &ShowBacklog, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        self.current_screen = Screen::Backlog;
        self.drill_down_stack.clear();
        cx.notify();
    }

    fn on_show_active(&mut self, _: &ShowActive, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        self.current_screen = Screen::Active;
        self.drill_down_stack.clear();
        cx.notify();
    }

    fn on_show_done(&mut self, _: &ShowDone, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        self.current_screen = Screen::Done;
        self.drill_down_stack.clear();
        cx.notify();
    }

    fn on_open_settings(&mut self, _: &OpenSettings, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        self.current_screen = Screen::Settings;
        cx.notify();
    }

    fn on_close_window(&mut self, _: &CloseWindow, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }

    fn get_columns_for_current_screen(&self) -> Vec<(&'static str, Location, Vec<Item>)> {
        if let Some(&sub_item_root_id) = self.drill_down_stack.last()
            && let Some(item) = self.board.find_item(&sub_item_root_id)
        {
            let sub_ids = crate::link::extract_links(&item.body);
            let items: Vec<Item> = sub_ids.into_iter().filter_map(|id| self.board.find_item(&id).cloned()).collect();
            return vec![("Sub-Items", Location::Active(Category::Today), items)];
        }

        match self.current_screen {
            Screen::Backlog => vec![("Backlog", Location::Backlog, self.board.backlog.clone())],
            Screen::Active => vec![
                ("Yesterday", Location::Active(Category::Yesterday), self.board.active.yesterday.clone()),
                ("Today", Location::Active(Category::Today), self.board.active.today.clone()),
                ("This Week", Location::Active(Category::ThisWeek), self.board.active.this_week.clone()),
                ("Next Week", Location::Active(Category::NextWeek), self.board.active.next_week.clone()),
            ],
            Screen::Done => vec![("Done", Location::Done, self.board.done.clone())],
            Screen::Settings => vec![],
        }
    }

    fn on_next_column(&mut self, _: &NextColumn, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        if self.current_screen != Screen::Active {
            self.current_screen = Screen::Active;
            if let Some(first) = self.board.active.yesterday.first() {
                self.selected_item = Some(first.id);
            }
            cx.notify();
            return;
        }

        let cols = [
            &self.board.active.yesterday,
            &self.board.active.today,
            &self.board.active.this_week,
            &self.board.active.next_week,
        ];

        let current_col_idx = self.selected_item.and_then(|id| {
            cols.iter().position(|col| col.iter().any(|item| item.id == id))
        }).unwrap_or(0);

        let next_col_idx = (current_col_idx + 1) % cols.len();
        if let Some(first) = cols[next_col_idx].first() {
            self.selected_item = Some(first.id);
        } else {
            for offset in 1..cols.len() {
                let idx = (current_col_idx + offset) % cols.len();
                if let Some(first) = cols[idx].first() {
                    self.selected_item = Some(first.id);
                    break;
                }
            }
        }
        cx.notify();
    }

    fn on_prev_column(&mut self, _: &PrevColumn, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        if self.current_screen != Screen::Active {
            self.current_screen = Screen::Active;
            if let Some(first) = self.board.active.next_week.first() {
                self.selected_item = Some(first.id);
            }
            cx.notify();
            return;
        }

        let cols = [
            &self.board.active.yesterday,
            &self.board.active.today,
            &self.board.active.this_week,
            &self.board.active.next_week,
        ];

        let current_col_idx = self.selected_item.and_then(|id| {
            cols.iter().position(|col| col.iter().any(|item| item.id == id))
        }).unwrap_or(0);

        let prev_col_idx = (current_col_idx + cols.len() - 1) % cols.len();
        if let Some(first) = cols[prev_col_idx].first() {
            self.selected_item = Some(first.id);
        } else {
            for offset in 1..cols.len() {
                let idx = (current_col_idx + cols.len() - offset) % cols.len();
                if let Some(first) = cols[idx].first() {
                    self.selected_item = Some(first.id);
                    break;
                }
            }
        }
        cx.notify();
    }

    fn on_nav_up(&mut self, _: &NavUp, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        let cols = self.get_columns_for_current_screen();
        for (_, _, items) in &cols {
            if let Some(sel_id) = self.selected_item
                && let Some(idx) = items.iter().position(|i| i.id == sel_id)
            {
                if idx > 0 {
                    self.selected_item = Some(items[idx - 1].id);
                    cx.notify();
                }
                return;
            }
        }
    }

    fn on_nav_down(&mut self, _: &NavDown, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        let cols = self.get_columns_for_current_screen();
        for (_, _, items) in &cols {
            if let Some(sel_id) = self.selected_item
                && let Some(idx) = items.iter().position(|i| i.id == sel_id)
            {
                if idx + 1 < items.len() {
                    self.selected_item = Some(items[idx + 1].id);
                    cx.notify();
                }
                return;
            }
        }
        if self.selected_item.is_none() {
            for (_, _, items) in &cols {
                if let Some(first) = items.first() {
                    self.selected_item = Some(first.id);
                    cx.notify();
                    return;
                }
            }
        }
    }

    fn on_nav_left(&mut self, _: &NavLeft, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() || self.current_screen != Screen::Active {
            return;
        }
        let cols = [
            &self.board.active.yesterday,
            &self.board.active.today,
            &self.board.active.this_week,
            &self.board.active.next_week,
        ];

        let Some(sel_id) = self.selected_item else {
            return;
        };

        let col_and_row = cols.iter().enumerate().find_map(|(col_idx, items)| {
            items.iter().position(|i| i.id == sel_id).map(|row_idx| (col_idx, row_idx))
        });

        let Some((col_idx, row_idx)) = col_and_row else {
            return;
        };

        if col_idx > 0 {
            let left_col = cols[col_idx - 1];
            if !left_col.is_empty() {
                let target_row = row_idx.min(left_col.len() - 1);
                self.selected_item = Some(left_col[target_row].id);
                cx.notify();
            }
        }
    }

    fn on_nav_right(&mut self, _: &NavRight, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() || self.current_screen != Screen::Active {
            return;
        }
        let cols = [
            &self.board.active.yesterday,
            &self.board.active.today,
            &self.board.active.this_week,
            &self.board.active.next_week,
        ];

        let Some(sel_id) = self.selected_item else {
            return;
        };

        let col_and_row = cols.iter().enumerate().find_map(|(col_idx, items)| {
            items.iter().position(|i| i.id == sel_id).map(|row_idx| (col_idx, row_idx))
        });

        let Some((col_idx, row_idx)) = col_and_row else {
            return;
        };

        if col_idx + 1 < cols.len() {
            let right_col = cols[col_idx + 1];
            if !right_col.is_empty() {
                let target_row = row_idx.min(right_col.len() - 1);
                self.selected_item = Some(right_col[target_row].id);
                cx.notify();
            }
        }
    }

    fn on_open_selected_for_edit(&mut self, _: &OpenSelectedForEdit, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        if let Some(id) = self.selected_item {
            self.open_editor_for_item(id, cx);
        }
    }

    fn move_selected_to(&mut self, location: Location, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
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
                let _ = Storage::save_board_state(&self.config.data_dir, &self.board);
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
        if self.editing.is_some() {
            return;
        }
        let Some(id) = self.selected_item else {
            return;
        };
        let Some(location) = self.board.find_item_location(&id) else {
            return;
        };
        if Storage::delete_item(&self.config.data_dir, &id, &location).is_ok() {
            let _ = self.board.remove_item(&id, &location);
            let _ = Storage::save_board_state(&self.config.data_dir, &self.board);
            self.selected_item = None;
            cx.notify();
        }
    }

    // -- Sub-item Drill-down and Creation --

    fn on_create_sub_item(&mut self, _: &CreateSubItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        let Some(parent_id) = self.selected_item else {
            return;
        };
        self.create_sub_item_for_parent(parent_id, cx);
    }

    fn create_sub_item_for_parent(&mut self, parent_id: Uuid, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        let sub_item = Item::new("New Sub-Item");
        let location = Location::Active(Category::Today);
        if Storage::write_item(&self.config.data_dir, &sub_item, &location).is_ok() {
            self.board.insert_item(sub_item.clone(), &location);
            
            // Append markdown link to parent body
            if let Some(parent_loc) = self.board.find_item_location(&parent_id) {
                if let Some(parent) = self.board.find_item_mut(&parent_id) {
                    let link_str = format!("\n- [{}]({}.md)", sub_item.title(), sub_item.id);
                    parent.body.push_str(&link_str);
                    parent.updated_at = chrono::Utc::now();
                }
                if let Some(parent_ref) = self.board.find_item(&parent_id).cloned() {
                    let _ = Storage::write_item(&self.config.data_dir, &parent_ref, &parent_loc);
                }
            }
            let _ = Storage::save_board_state(&self.config.data_dir, &self.board);
            self.open_editor_for_item(sub_item.id, cx);
        }
    }

    fn on_drill_down_sub_item(&mut self, _: &DrillDownSubItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        if let Some(id) = self.selected_item {
            let count = count_recursive_subitems(id, &self.config.data_dir);
            if count > 0 {
                self.drill_down_stack.push(id);
                self.selected_item = None;
                cx.notify();
            }
        }
    }

    fn on_drill_up_breadcrumb(&mut self, _: &DrillUpBreadcrumb, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        if !self.drill_down_stack.is_empty() {
            let popped = self.drill_down_stack.pop();
            self.selected_item = popped;
            cx.notify();
        }
    }

    // -- Item editor --

    fn on_new_item(&mut self, _: &NewItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        let editor = cx.new(|cx| ItemEditor::new(cx, "", None, true, self.config.clone(), false));
        self.editing = Some(EditingState { editor });
        cx.notify();
    }

    fn open_editor_for_item(&mut self, id: Uuid, cx: &mut Context<Self>) {
        let body = self.board.find_item(&id).map(|i| i.body.clone()).unwrap_or_default();
        let editor = cx.new(|cx| ItemEditor::new(cx, &body, Some(id), false, self.config.clone(), false));
        self.editing = Some(EditingState { editor });
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

        let is_new = editing.editor.read(cx).is_new;
        let item_id = editing.editor.read(cx).editing_item_id;

        if is_new {
            let item = Item::new(&content);
            let location = match self.current_screen {
                Screen::Backlog => Location::Backlog,
                Screen::Active => Location::Active(Category::Today),
                Screen::Done => Location::Backlog,
                Screen::Settings => Location::Backlog,
            };
            if Storage::write_item(&self.config.data_dir, &item, &location).is_ok() {
                self.board.insert_item(item.clone(), &location);
                let _ = Storage::save_board_state(&self.config.data_dir, &self.board);
                self.selected_item = Some(item.id);
            }
        } else if let Some(id) = item_id
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
        let item_id = editing.editor.read(cx).editing_item_id;
        let is_new = editing.editor.read(cx).is_new;
        let config = self.config.clone();

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
            cx.new(|cx| ItemEditor::new(cx, &content, item_id, is_new, config, true))
        });
        cx.notify();
    }

    // -- Keyboard navigation fallback --

    fn current_screen_items(&self) -> Vec<Uuid> {
        let cols = self.get_columns_for_current_screen();
        let mut items = Vec::new();
        for (_, _, col_items) in cols {
            items.extend(col_items.iter().map(|i| i.id));
        }
        items
    }

    fn on_next_item(&mut self, _: &NextItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
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
        if self.editing.is_some() {
            return;
        }
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

    // -- Drag and Drop handling --

    fn handle_card_drop_on_column(&mut self, dragged_id: Uuid, target_location: Location, cx: &mut Context<Self>) {
        let Some(from) = self.board.find_item_location(&dragged_id) else {
            return;
        };

        if from == target_location {
            return;
        }

        if !self.board.can_move(&dragged_id, &target_location) {
            return;
        }

        match Storage::move_item(&self.config.data_dir, &dragged_id, &from, &target_location) {
            Ok(updated_item) => {
                let _ = self.board.remove_item(&dragged_id, &from);
                self.board.insert_item(updated_item, &target_location);
                let _ = Storage::save_board_state(&self.config.data_dir, &self.board);
                self.selected_item = Some(dragged_id);
                cx.notify();
            }
            Err(e) => {
                eprintln!("Failed to move dragged item: {}", e);
            }
        }
    }

    fn handle_card_reorder(&mut self, dragged_id: Uuid, target_id: Uuid, target_location: Location, cx: &mut Context<Self>) {
        let Some(from) = self.board.find_item_location(&dragged_id) else {
            return;
        };

        if from != target_location {
            if !self.board.can_move(&dragged_id, &target_location) {
                return;
            }
            if let Ok(updated) = Storage::move_item(&self.config.data_dir, &dragged_id, &from, &target_location) {
                let _ = self.board.remove_item(&dragged_id, &from);
                self.board.insert_item(updated, &target_location);
            }
        }

        // Reorder within target column
        let mut ordered_ids = match target_location {
            Location::Backlog => self.board.backlog.iter().map(|i| i.id).collect::<Vec<_>>(),
            Location::Active(Category::Yesterday) => self.board.active.yesterday.iter().map(|i| i.id).collect::<Vec<_>>(),
            Location::Active(Category::Today) => self.board.active.today.iter().map(|i| i.id).collect::<Vec<_>>(),
            Location::Active(Category::ThisWeek) => self.board.active.this_week.iter().map(|i| i.id).collect::<Vec<_>>(),
            Location::Active(Category::NextWeek) => self.board.active.next_week.iter().map(|i| i.id).collect::<Vec<_>>(),
            Location::Done => self.board.done.iter().map(|i| i.id).collect::<Vec<_>>(),
        };

        if let Some(pos) = ordered_ids.iter().position(|id| *id == dragged_id) {
            ordered_ids.remove(pos);
        }
        if let Some(target_pos) = ordered_ids.iter().position(|id| *id == target_id) {
            ordered_ids.insert(target_pos, dragged_id);
        } else {
            ordered_ids.push(dragged_id);
        }

        self.board.set_column_order(&target_location, ordered_ids);
        let _ = Storage::save_board_state(&self.config.data_dir, &self.board);
        self.selected_item = Some(dragged_id);
        cx.notify();
    }

    // -- Rendering helpers --

    fn render_tab(&self, label: &str, screen: Screen, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = self.current_screen == screen;
        div()
            .px(px(12.))
            .py(px(6.))
            .rounded(px(4.))
            .bg(if is_active { theme.bg_surface } else { theme.bg_tab_bar })
            .border_1()
            .border_color(if is_active { theme.border } else { rgba(0x00000000) })
            .text_sm()
            .text_color(if is_active { theme.accent } else { theme.text_secondary })
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

    fn render_breadcrumbs(&self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(12.))
            .py(px(6.))
            .bg(theme.bg_column)
            .border_b_1()
            .border_color(theme.border)
            .text_sm()
            .child(
                div()
                    .cursor_pointer()
                    .text_color(theme.accent)
                    .child("Root Board")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.drill_down_stack.clear();
                            this.selected_item = None;
                            cx.notify();
                        }),
                    ),
            );

        for (idx, &item_id) in self.drill_down_stack.iter().enumerate() {
            let item_title = self.board.find_item(&item_id).map(|i| i.title()).unwrap_or_else(|| "Item".to_string());
            let is_last = idx == self.drill_down_stack.len() - 1;
            let stack_len = idx + 1;

            row = row.child(div().text_color(theme.text_secondary).child(">"));
            row = row.child(
                div()
                    .cursor_pointer()
                    .text_color(if is_last { theme.text_primary } else { theme.accent })
                    .child(item_title)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _window, cx| {
                            this.drill_down_stack.truncate(stack_len);
                            cx.notify();
                        }),
                    ),
            );
        }

        row
    }

    fn render_item_card(&self, item: &Item, location: Location, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let is_selected = self.selected_item == Some(item.id);
        let item_id = item.id;
        let subitem_count = count_recursive_subitems(item.id, &self.config.data_dir);

        div()
            .id(SharedString::from(format!("card-{}", item.id)))
            .w_full()
            .p(px(8.))
            .mb(px(6.))
            .rounded(px(6.))
            .bg(if is_selected {
                if theme.bg_window == rgb(0x1e1e1e) { rgb(0x1a365d) } else { rgb(0xe3f2fd) }
            } else {
                theme.bg_surface
            })
            .border_1()
            .border_color(if is_selected { theme.selection } else { theme.border })
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
            .on_drag(DraggedCard { id: item_id }, |drag, _point, _window, cx| {
                cx.new(|_| *drag)
            })
            .drag_over::<DraggedCard>(|this, _, _window, _cx| {
                this.border_color(rgb(0x0a84ff))
            })
            .on_drop(cx.listener(move |this, dragged: &DraggedCard, _window, cx| {
                this.handle_card_reorder(dragged.id, item_id, location, cx);
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(theme.text_primary)
                            .child(item.title()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(4.))
                            .children(if subitem_count > 0 {
                                Some(
                                    div()
                                        .px(px(4.))
                                        .py(px(1.))
                                        .rounded(px(3.))
                                        .bg(theme.bg_column)
                                        .text_xs()
                                        .text_color(theme.accent)
                                        .cursor_pointer()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _window, cx| {
                                                this.drill_down_stack.push(item_id);
                                                this.selected_item = None;
                                                cx.notify();
                                            }),
                                        )
                                        .child(format!("\u{21AA} {}", subitem_count)),
                                )
                            } else {
                                None
                            })
                            .child(
                                div()
                                    .px(px(4.))
                                    .py(px(1.))
                                    .rounded(px(3.))
                                    .text_xs()
                                    .text_color(theme.text_secondary)
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
                                            this.create_sub_item_for_parent(item_id, cx);
                                        }),
                                    )
                                    .child("+"),
                            ),
                    ),
            )
    }

    fn render_column(
        &self,
        title: &str,
        location: Location,
        items: &[Item],
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let item_count = items.len();
        div()
            .id(SharedString::from(format!("column-{}", title)))
            .flex_1()
            .flex()
            .flex_col()
            .bg(theme.bg_column)
            .rounded(px(6.))
            .p(px(8.))
            .m(px(4.))
            .border_1()
            .border_color(theme.border)
            .drag_over::<DraggedCard>(|this, _, _window, _cx| {
                this.bg(rgb(0xd0e8ff))
            })
            .on_drop(cx.listener(move |this, dragged: &DraggedCard, _window, cx| {
                this.handle_card_drop_on_column(dragged.id, location, cx);
            }))
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
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child(title.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(10.))
                            .bg(theme.bg_surface)
                            .text_color(theme.text_secondary)
                            .child(format!("{}", item_count)),
                    ),
            )
            .children({
                let mut cards: Vec<gpui::AnyElement> = Vec::new();
                for item in items {
                    cards.push(self.render_item_card(item, location, theme, cx).into_any_element());
                }
                cards
            })
    }

    fn render_backlog_screen(&self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_1()
            .p(px(8.))
            .child(self.render_column("Backlog", Location::Backlog, &self.board.backlog, theme, cx))
    }

    fn render_active_screen(&self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.drill_down_stack.is_empty() {
            let last_id = *self.drill_down_stack.last().unwrap();
            let sub_ids = self.board.find_item(&last_id).map(|i| crate::link::extract_links(&i.body)).unwrap_or_default();
            let sub_items: Vec<Item> = sub_ids.into_iter().filter_map(|id| self.board.find_item(&id).cloned()).collect();

            return div()
                .flex()
                .flex_row()
                .flex_1()
                .p(px(8.))
                .child(self.render_column("Sub-Items", Location::Active(Category::Today), &sub_items, theme, cx));
        }

        div()
            .flex()
            .flex_row()
            .flex_1()
            .p(px(4.))
            .child(self.render_column("Yesterday", Location::Active(Category::Yesterday), &self.board.active.yesterday, theme, cx))
            .child(self.render_column("Today", Location::Active(Category::Today), &self.board.active.today, theme, cx))
            .child(self.render_column("This Week", Location::Active(Category::ThisWeek), &self.board.active.this_week, theme, cx))
            .child(self.render_column("Next Week", Location::Active(Category::NextWeek), &self.board.active.next_week, theme, cx))
    }

    fn render_done_screen(&self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_1()
            .p(px(8.))
            .child(self.render_column("Done", Location::Done, &self.board.done, theme, cx))
    }

    fn render_settings_screen(&self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let vi_mode = self.config.vi_mode;
        let line_numbers = self.config.line_numbers;
        let theme_mode = self.config.theme_mode;
        let data_dir_display = self.config.data_dir.to_string_lossy().to_string();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .p(px(24.))
            .gap(px(16.))
            .bg(theme.bg_surface)
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child("Settings"),
            )
            // Theme Mode Selector
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child("Appearance / Theme Mode"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.))
                            .child(self.render_theme_option("System", ThemeMode::System, theme_mode, theme, cx))
                            .child(self.render_theme_option("Light", ThemeMode::Light, theme_mode, theme, cx))
                            .child(self.render_theme_option("Dark", ThemeMode::Dark, theme_mode, theme, cx)),
                    ),
            )
            // Vi Mode Toggle
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .py(px(8.))
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_primary)
                                    .child("Vi Mode Navigation & Editing"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_secondary)
                                    .child("Enable modal vi navigation (hjkl) in kanban view and editor"),
                            ),
                    )
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(4.))
                            .rounded(px(4.))
                            .bg(if vi_mode { theme.accent } else { theme.bg_column })
                            .text_color(if vi_mode { rgb(0xffffff) } else { theme.text_primary })
                            .text_xs()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.config.vi_mode = !this.config.vi_mode;
                                    let _ = this.config.save_to(&Config::config_file_path());
                                    cx.notify();
                                }),
                            )
                            .child(if vi_mode { "Enabled" } else { "Disabled" }),
                    ),
            )
            // Line Numbers Toggle
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .py(px(8.))
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_primary)
                                    .child("Editor Line Numbers"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_secondary)
                                    .child("Show line number gutter in markdown item editor"),
                            ),
                    )
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(4.))
                            .rounded(px(4.))
                            .bg(if line_numbers { theme.accent } else { theme.bg_column })
                            .text_color(if line_numbers { rgb(0xffffff) } else { theme.text_primary })
                            .text_xs()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.config.line_numbers = !this.config.line_numbers;
                                    let _ = this.config.save_to(&Config::config_file_path());
                                    cx.notify();
                                }),
                            )
                            .child(if line_numbers { "Enabled" } else { "Disabled" }),
                    ),
            )
            // Storage Directory
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child("Storage Directory"),
                    )
                    .child(
                        div()
                            .p(px(8.))
                            .rounded(px(4.))
                            .bg(theme.bg_column)
                            .text_xs()
                            .text_color(theme.text_secondary)
                            .child(data_dir_display),
                    ),
            )
    }

    fn render_theme_option(
        &self,
        label: &str,
        mode: ThemeMode,
        active_mode: ThemeMode,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = mode == active_mode;
        div()
            .px(px(12.))
            .py(px(6.))
            .rounded(px(4.))
            .bg(if is_active { theme.accent } else { theme.bg_column })
            .text_color(if is_active { rgb(0xffffff) } else { theme.text_primary })
            .text_xs()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _window, cx| {
                    this.config.theme_mode = mode;
                    let _ = this.config.save_to(&Config::config_file_path());
                    cx.notify();
                }),
            )
            .child(label.to_string())
    }

    fn render_editor_modal(&self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(ref editing) = self.editing {
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .bg(rgba(0x00000060))
                .flex()
                .justify_center()
                .items_center()
                .child(
                    div()
                        .w(px(600.))
                        .h(px(400.))
                        .bg(theme.bg_surface)
                        .rounded(px(8.))
                        .border_1()
                        .border_color(theme.border)
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        // Top bar with tear-off button
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_end()
                                .p(px(4.))
                                .bg(theme.bg_tab_bar)
                                .border_b_1()
                                .border_color(theme.border)
                                .child(
                                    div()
                                        .id("tear-off")
                                        .px(px(8.))
                                        .py(px(4.))
                                        .cursor_pointer()
                                        .text_color(theme.text_secondary)
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
                                .border_color(theme.border)
                                .bg(theme.bg_tab_bar)
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
                                        .bg(theme.bg_surface)
                                        .border_1()
                                        .border_color(theme.border)
                                        .text_color(theme.text_primary)
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
