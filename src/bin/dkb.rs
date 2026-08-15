use gpui::{App, prelude::*};
use gpui_platform::application;

use dkb::app::KanbanView;
use dkb::config::Config;

fn main() {
    application().run(|cx: &mut App| {
        cx.activate(true);

        let config = Config::load().unwrap_or_default();

        KanbanView::setup_menus(cx, config.language);

        let opts = KanbanView::main_window_options(cx);
        cx.open_window(opts, |_, cx| {
            cx.new(KanbanView::new)
        })
        .unwrap();
    });
}
