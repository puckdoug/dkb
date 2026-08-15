#![allow(clippy::pedantic)]

use dkb_core::i18n::{detect_system_language, t, Language};

#[test]
fn test_supported_languages_count() {
    assert_eq!(Language::all().len(), 45); // Auto + 44 languages
}

#[test]
fn test_translation_fallback() {
    assert_eq!(t("tab.backlog", Language::EnUs), "Backlog");
    assert_eq!(t("tab.backlog", Language::EsEs), "Pendientes");
    assert_eq!(t("tab.backlog", Language::FrFr), "Backlog");
    assert_eq!(t("tab.backlog", Language::De), "Rückstand");
    // Fallback to English when key is missing in custom language
    assert_eq!(
        t("tab.backlog", Language::Auto),
        t("tab.backlog", detect_system_language())
    );
}

#[test]
fn test_language_display_names() {
    assert_eq!(Language::EnUs.display_name(), "English (US)");
    assert_eq!(Language::Auto.display_name(), "System Default (Auto)");
    assert_eq!(Language::Ja.display_name(), "Japanese");
    assert_eq!(Language::ZhHans.display_name(), "Chinese (Simplified)");
    assert_eq!(Language::ZhHant.display_name(), "Chinese (Traditional)");
}

#[test]
fn test_language_codes_and_from_code() {
    assert_eq!(Language::EnUs.code(), "en-US");
    assert_eq!(Language::from_code("en-US"), Some(Language::EnUs));
    assert_eq!(Language::from_code("en_US"), Some(Language::EnUs));
    assert_eq!(Language::from_code("en"), Some(Language::EnUs));
    assert_eq!(Language::from_code("es-ES"), Some(Language::EsEs));
    assert_eq!(Language::from_code("es-MX"), Some(Language::EsMx));
    assert_eq!(Language::from_code("es_419"), Some(Language::Es419));
    assert_eq!(Language::from_code("zh-Hans"), Some(Language::ZhHans));
    assert_eq!(Language::from_code("zh-Hant"), Some(Language::ZhHant));
    assert_eq!(Language::from_code("ja_JP"), Some(Language::Ja));
    assert_eq!(Language::from_code("auto"), Some(Language::Auto));
    assert_eq!(Language::from_code("unknown_code"), None);
}

#[test]
fn test_all_catalog_keys_exist_in_english() {
    let keys = [
        "tab.backlog",
        "tab.active",
        "tab.done",
        "tab.settings",
        "col.yesterday",
        "col.today",
        "col.this_week",
        "col.next_week",
        "col.backlog",
        "col.done",
        "col.sub_items",
        "menu.app.settings",
        "menu.app.quit",
        "menu.file",
        "menu.file.new_item",
        "menu.file.new_sub_item",
        "menu.file.new_window",
        "menu.file.close_window",
        "menu.view",
        "menu.item",
        "menu.item.open_edit",
        "menu.item.open_markdown_viewer",
        "menu.item.mark_done",
        "menu.item.delete",
        "settings.title",
        "settings.appearance",
        "settings.vi_mode",
        "settings.line_numbers",
        "settings.language",
        "settings.markdown_viewer",
        "settings.storage_dir",
        "settings.browse",
        "settings.reset_auto",
        "editor.save",
        "editor.cancel",
        "editor.tear_off",
    ];

    for key in keys {
        let text = t(key, Language::EnUs);
        assert_ne!(text, key, "Key {key} should have translation in EnUs");
    }
}
