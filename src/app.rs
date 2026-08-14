use gpui::{
    App, Context, FocusHandle, Focusable, KeyBinding, Menu, MenuItem, Render, Window,
    actions, div, prelude::*, px, rgb,
};

use crate::board::Board;
use crate::config::Config;
use crate::storage::Storage;
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let screen = self.current_screen;
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xf5f5f5))
            .size_full()
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(4.))
                    .p(px(8.))
                    .bg(rgb(0xe0e0e0))
                    .child(self.render_tab("Backlog", Screen::Backlog))
                    .child(self.render_tab("Active", Screen::Active))
                    .child(self.render_tab("Done", Screen::Done)),
            )
            .child(
                div()
                    .flex_1()
                    .p(px(16.))
                    .child(match screen {
                        Screen::Backlog => "Backlog Screen",
                        Screen::Active => "Active Screen",
                        Screen::Done => "Done Screen",
                    }),
            )
    }
}

impl KanbanView {
    fn render_tab(&self, label: &str, screen: Screen) -> impl IntoElement {
        let is_active = self.current_screen == screen;
        div()
            .px(px(12.))
            .py(px(6.))
            .rounded(px(4.))
            .bg(if is_active { rgb(0xffffff) } else { rgb(0xe0e0e0) })
            .text_sm()
            .text_color(rgb(0x333333))
            .child(label.to_string())
    }
}
