use gpui::{
    App, Context, FocusHandle, Focusable, KeyBinding, Menu, MenuItem, Render, Window,
    actions, div, prelude::*, px, rgb,
};

use crate::board::Board;
use crate::config::Config;
use crate::item::{Category, Item, Status};
use crate::storage::{Location, Storage};
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
    ]
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Backlog,
    Active,
    Done,
}

pub struct KanbanView {
    pub board: Board,
    pub current_screen: Screen,
    pub config: Config,
    pub focus_handle: FocusHandle,
    pub selected_item: Option<Uuid>,
    pub quick_add_active: bool,
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
            quick_add_active: false,
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

    fn on_new_item(&mut self, _: &NewItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.quick_add_active = true;
        cx.notify();
    }

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

    fn commit_quick_add(&mut self, title: &str, cx: &mut Context<Self>) {
        let title = title.trim();
        if title.is_empty() {
            self.quick_add_active = false;
            cx.notify();
            return;
        }
        let item = Item::new(title);
        let location = match self.current_screen {
            Screen::Backlog => Location::Backlog,
            Screen::Active => Location::Active(Category::Today),
            Screen::Done => Location::Backlog,
        };
        if Storage::write_item(&self.config.data_dir, &item, &location).is_ok() {
            self.board.insert_item(item, &location);
        }
        self.quick_add_active = false;
        cx.notify();
    }

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
                gpui::MouseButton::Left,
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
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _window, cx| {
                    this.selected_item = Some(item_id);
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
            .child(self.render_quick_add_bar(cx))
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

    fn render_quick_add_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.quick_add_active {
            div()
                .p(px(8.))
                .bg(rgb(0xffffff))
                .border_b_1()
                .border_color(rgb(0xcccccc))
                .child(
                    div()
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .border_1()
                        .border_color(rgb(0x2196f3))
                        .text_sm()
                        .text_color(rgb(0x999999))
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _window, cx| {
                                this.quick_add_active = false;
                                cx.notify();
                            }),
                        )
                        .child("Type item title, press Enter to create... (click to cancel)"),
                )
        } else {
            div()
        }
    }
}
