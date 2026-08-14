use gpui::{Rgba, rgb};
use crate::config::ThemeMode;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg_window: Rgba,
    pub bg_surface: Rgba,
    pub bg_column: Rgba,
    pub bg_tab_bar: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub border: Rgba,
    pub selection: Rgba,
    pub accent: Rgba,
}

impl Theme {
    pub fn resolve(mode: ThemeMode, system_is_dark: bool) -> Self {
        let is_dark = match mode {
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
            ThemeMode::System => system_is_dark,
        };

        if is_dark {
            Self {
                bg_window: rgb(0x1e1e1e),
                bg_surface: rgb(0x252526),
                bg_column: rgb(0x2d2d2d),
                bg_tab_bar: rgb(0x181818),
                text_primary: rgb(0xe0e0e0),
                text_secondary: rgb(0x9e9e9e),
                border: rgb(0x383838),
                selection: rgb(0x0a84ff),
                accent: rgb(0x4488ff),
            }
        } else {
            Self {
                bg_window: rgb(0xf5f5f5),
                bg_surface: rgb(0xffffff),
                bg_column: rgb(0xeceff1),
                bg_tab_bar: rgb(0xe0e0e0),
                text_primary: rgb(0x212121),
                text_secondary: rgb(0x757575),
                border: rgb(0xd0d0d0),
                selection: rgb(0x2196f3),
                accent: rgb(0x007aff),
            }
        }
    }
}
