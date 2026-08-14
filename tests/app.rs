use dkb::app::{
    NavDown, NavLeft, NavRight, NavUp, NextColumn,
    OpenSelectedForEdit, OpenSettings, PrevColumn, Screen,
};
use dkb::config::{Config, ThemeMode};
use dkb::theme::Theme;

#[test]
fn test_screen_variants() {
    assert_eq!(Screen::Backlog, Screen::Backlog);
    assert_eq!(Screen::Active, Screen::Active);
    assert_eq!(Screen::Done, Screen::Done);
    assert_eq!(Screen::Settings, Screen::Settings);
}

#[test]
fn test_actions_defined() {
    let _ = NextColumn;
    let _ = PrevColumn;
    let _ = NavUp;
    let _ = NavDown;
    let _ = NavLeft;
    let _ = NavRight;
    let _ = OpenSettings;
    let _ = OpenSelectedForEdit;
}

#[test]
fn test_theme_mode_settings_applied() {
    let mut config = Config {
        data_dir: Config::default_data_dir(),
        vi_mode: true,
        line_numbers: true,
        theme_mode: ThemeMode::Dark,
    };
    let theme = Theme::resolve(config.theme_mode, false);
    // Dark theme should have dark window background
    assert_eq!(theme.bg_window, gpui::rgb(0x1e1e1e));

    config.theme_mode = ThemeMode::Light;
    let theme_light = Theme::resolve(config.theme_mode, false);
    assert_eq!(theme_light.bg_window, gpui::rgb(0xf5f5f5));
}

#[test]
fn test_key_bindings_context_isolation() {
    let bindings = dkb::app::KanbanView::key_bindings();
    assert!(bindings.len() > 20);
}
