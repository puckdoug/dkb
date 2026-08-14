use gpui::{App, prelude::*};
use gpui_platform::application;

use dkb::app::KanbanView;
use dkb::config::{Config, ThemeMode};
use dkb::i18n::Language;
use dkb::viewer::ViewerPreference;

fn main() {
    application().run(|cx: &mut App| {
        cx.activate(true);

        let config = Config::load().unwrap_or_else(|_| Config {
            data_dir: Config::default_data_dir(),
            vi_mode: false,
            line_numbers: false,
            theme_mode: ThemeMode::System,
            language: Language::Auto,
            markdown_viewer: ViewerPreference::Auto,
        });

        KanbanView::setup_menus(cx, config.language);

        let opts = KanbanView::main_window_options(cx);
        cx.open_window(opts, |_, cx| {
            cx.new(KanbanView::new)
        })
        .unwrap();
    });
}
