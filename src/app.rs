use gpui::{Context, FocusHandle, Focusable, Render, Window, div, prelude::*, rgb};

pub struct KanbanView {
    focus_handle: FocusHandle,
}

impl KanbanView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for KanbanView {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KanbanView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .size_full()
            .track_focus(&self.focus_handle)
    }
}
