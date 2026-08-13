use gpui::{App, WindowOptions, div, prelude::*, px, rgb, size};
use gpui_platform::application;

fn main() {
    application().run(|cx: &mut App| {
        cx.activate(true);

        let opts = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1000.), px(700.)),
                cx,
            ))),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Daily Kanban".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        cx.open_window(opts, |_, cx| {
            cx.new(|_cx| dkb::app::KanbanView::new(_cx))
        })
        .unwrap();
    });
}
