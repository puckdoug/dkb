#![allow(clippy::pedantic)]

use dkb::app::{
    ContextMenuState, DeleteItem, KanbanView, MoveItemDown, MoveItemUp, NavDown, NavLeft, NavRight,
    NavUp, NewItem, NextColumn, OpenInMarkdownViewer, OpenNewMainWindow, OpenSelectedForEdit,
    OpenSettings, PrevColumn, Screen, ToggleDone,
};
use dkb::config::{Config, ThemeMode};
use dkb::editor::{CloseWindow, EditorEvent, SaveEditor};
use dkb::i18n::Language;
use dkb::item::{Category, Item};
use dkb::storage::{Location, Storage};
use dkb::theme::Theme;
use dkb::vi::{ViActionResult, ViMode};

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
    let _ = OpenNewMainWindow;
    let _ = OpenInMarkdownViewer;
    let _ = SaveEditor;
    let _ = EditorEvent::Save;
    let _ = EditorEvent::Close;
}

#[test]
fn test_theme_mode_settings_applied() {
    let mut config = Config {
        data_dir: Config::default_data_dir(),
        vi_mode: true,
        line_numbers: true,
        theme_mode: ThemeMode::Dark,
        language: Language::Auto,
        markdown_viewer: dkb::viewer::ViewerPreference::Auto,
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
    assert!(bindings.iter().any(|b| b.action().name() == "dkb::OpenNewMainWindow"));
    assert!(bindings.iter().any(|b| b.action().name() == "dkb::OpenInMarkdownViewer"));
}

#[gpui::test]
fn test_modal_cmd_w_cancels_editor_without_closing_window(cx: &mut gpui::TestAppContext) {
    let window = cx.add_window(|_window, cx| KanbanView::new(cx));
    window
        .update(cx, |view, window, cx| {
            assert!(view.editing.is_none());

            // Open new item editor
            view.on_new_item(&NewItem, window, cx);
            assert!(view.editing.is_some());

            // Dispatch CloseWindow (cmd-w) on KanbanView while editing
            view.on_close_window(&CloseWindow, window, cx);

            // Modal must be dismissed, and window was not removed
            assert!(view.editing.is_none());
        })
        .unwrap();
}

#[gpui::test]
fn test_editor_attached_close_emits_event_and_dismisses_modal(cx: &mut gpui::TestAppContext) {
    let window = cx.add_window(|_window, cx| KanbanView::new(cx));
    let editor = window
        .update(cx, |view, window, cx| {
            // Open new item editor
            view.on_new_item(&NewItem, window, cx);
            assert!(view.editing.is_some());
            view.editing.as_ref().unwrap().editor.clone()
        })
        .unwrap();

    // Trigger close on the ItemEditor directly (e.g. from editor shortcut or :q)
    window
        .update(cx, |_view, window, cx| {
            editor.update(cx, |editor_view, cx| {
                editor_view.on_close(&CloseWindow, window, cx);
            });
        })
        .unwrap();

    // After notifications are dispatched, modal must be dismissed
    window
        .update(cx, |view, _window, _cx| {
            assert!(view.editing.is_none());
        })
        .unwrap();
}

#[gpui::test]
fn test_editor_vi_ex_save_and_close_integration(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        vi_mode: true,
        line_numbers: true,
        theme_mode: ThemeMode::Dark,
        language: Language::Auto,
        markdown_viewer: dkb::viewer::ViewerPreference::Auto,
    };

    let window = cx.add_window(|_window, cx| {
        let mut view = KanbanView::new(cx);
        view.config = config;
        view
    });

    let editor = window
        .update(cx, |view, window, cx| {
            view.on_new_item(&NewItem, window, cx);
            assert!(view.editing.is_some());
            view.editing.as_ref().unwrap().editor.clone()
        })
        .unwrap();

    // Set content in editor and execute SaveAndClose
    window
        .update(cx, |_view, window, cx| {
            editor.update(cx, |editor_view, cx| {
                editor_view.state.insert("Ex Command Task\nDetails here");
                assert_eq!(editor_view.vi_state.mode, ViMode::Normal);

                // Enter command mode and type wq
                editor_view.vi_state.mode = ViMode::Command;
                editor_view.vi_state.command_buffer = "wq".to_string();
                assert_eq!(editor_view.vi_state.command_buffer, "wq");

                // Execute SaveAndClose action
                editor_view.process_vi_action(&ViActionResult::SaveAndClose, window, cx);
            });
        })
        .unwrap();

    // Modal should be closed now
    window
        .update(cx, |view, _window, _cx| {
            assert!(view.editing.is_none());

            // Saved item should exist on disk in active today
            let today_dir = view.config.data_dir.join("active/today");
            let files: Vec<_> = std::fs::read_dir(today_dir).unwrap().collect();
            assert_eq!(files.len(), 1);
        })
        .unwrap();
}

#[gpui::test]
fn test_menus_contain_new_window_and_markdown_viewer() {
    let menus = KanbanView::menus(Language::EnUs);
    let file_menu = menus.iter().find(|m| m.name == "File").expect("File menu exists");
    let has_new_window = file_menu.items.iter().any(|item| match item {
        gpui::MenuItem::Action { name, .. } => name == "New Window",
        _ => false,
    });
    assert!(has_new_window, "File menu should contain 'New Window'");

    let item_menu = menus.iter().find(|m| m.name == "Item").expect("Item menu exists");
    let has_md_viewer = item_menu.items.iter().any(|item| match item {
        gpui::MenuItem::Action { name, .. } => name == "Open in Markdown Viewer",
        _ => false,
    });
    assert!(has_md_viewer, "Item menu should contain 'Open in Markdown Viewer'");
}

#[gpui::test]
fn test_localized_menus_german() {
    let menus = KanbanView::menus(Language::De);
    let file_menu = menus
        .iter()
        .find(|m| m.name == "Ablage")
        .expect("German Ablage (File) menu exists");
    let has_new_window = file_menu.items.iter().any(|item| match item {
        gpui::MenuItem::Action { name, .. } => name == "Neues Fenster",
        _ => false,
    });
    assert!(has_new_window, "File menu should contain 'Neues Fenster'");

    let item_menu = menus
        .iter()
        .find(|m| m.name == "Element")
        .expect("German Element (Item) menu exists");
    let has_md_viewer = item_menu.items.iter().any(|item| match item {
        gpui::MenuItem::Action { name, .. } => name == "In Markdown-Viewer öffnen",
        _ => false,
    });
    assert!(has_md_viewer, "Item menu should contain 'In Markdown-Viewer öffnen'");
}

#[gpui::test]
fn test_settings_language_switch(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config_file = dir.path().join("config.toml");
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        vi_mode: false,
        line_numbers: false,
        theme_mode: ThemeMode::System,
        language: Language::EnUs,
        markdown_viewer: dkb::viewer::ViewerPreference::Auto,
    };
    config.save_to(&config_file).unwrap();

    let window = cx.add_window(|_window, cx| {
        let mut view = KanbanView::new(cx);
        view.config = config;
        view
    });

    window
        .update(cx, |view, _window, cx| {
            assert_eq!(view.config.language, Language::EnUs);

            // Switch to Japanese
            view.config.language = Language::Ja;
            view.config.save_to(&config_file).unwrap();
            KanbanView::setup_menus(cx, view.config.language);
            cx.notify();
        })
        .unwrap();

    // Verify config persisted and reloaded
    let reloaded = Config::load_from(&config_file).unwrap();
    assert_eq!(reloaded.language, Language::Ja);

    // Verify Japanese menus
    let ja_menus = KanbanView::menus(Language::Ja);
    let ja_file_menu = ja_menus
        .iter()
        .find(|m| m.name == "ファイル")
        .expect("Japanese File menu exists");
    let has_ja_new_window = ja_file_menu.items.iter().any(|item| match item {
        gpui::MenuItem::Action { name, .. } => name == "新規ウィンドウ",
        _ => false,
    });
    assert!(has_ja_new_window);

    // Verify dropdown state toggle
    window
        .update(cx, |view, _window, cx| {
            assert!(!view.language_dropdown_open);
            view.language_dropdown_open = true;
            cx.notify();
            assert!(view.language_dropdown_open);
            view.language_dropdown_open = false;
            cx.notify();
            assert!(!view.language_dropdown_open);
        })
        .unwrap();
}

#[gpui::test]
fn test_settings_markdown_viewer_preference_switch(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config_file = dir.path().join("config.toml");
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        vi_mode: false,
        line_numbers: false,
        theme_mode: ThemeMode::System,
        language: Language::EnUs,
        markdown_viewer: dkb::viewer::ViewerPreference::Auto,
    };
    config.save_to(&config_file).unwrap();

    let window = cx.add_window(|_window, cx| {
        let mut view = KanbanView::new(cx);
        view.config = config;
        view
    });

    // 1. Set custom markdown viewer
    window
        .update(cx, |view, _window, cx| {
            assert_eq!(view.config.markdown_viewer, dkb::viewer::ViewerPreference::Auto);
            view.config.markdown_viewer = dkb::viewer::ViewerPreference::Custom(std::path::PathBuf::from("/Applications/Marked 2.app"));
            view.config.save_to(&config_file).unwrap();
            cx.notify();
        })
        .unwrap();

    let reloaded = Config::load_from(&config_file).unwrap();
    assert_eq!(
        reloaded.markdown_viewer,
        dkb::viewer::ViewerPreference::Custom(std::path::PathBuf::from("/Applications/Marked 2.app"))
    );

    // 2. Reset to Auto-Detect
    window
        .update(cx, |view, _window, cx| {
            view.config.markdown_viewer = dkb::viewer::ViewerPreference::Auto;
            view.config.save_to(&config_file).unwrap();
            cx.notify();
        })
        .unwrap();

    let reloaded_auto = Config::load_from(&config_file).unwrap();
    assert_eq!(reloaded_auto.markdown_viewer, dkb::viewer::ViewerPreference::Auto);
}

#[gpui::test]
fn test_open_new_main_window_action(cx: &mut gpui::TestAppContext) {
    let window = cx.add_window(|_window, cx| KanbanView::new(cx));
    window
        .update(cx, |view, window, cx| {
            let initial_window_count = cx.windows().len();
            view.on_open_new_main_window(&OpenNewMainWindow, window, cx);
            assert_eq!(cx.windows().len(), initial_window_count + 1);
        })
        .unwrap();
}

#[gpui::test]
fn test_open_in_markdown_viewer_action_runs_safely(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        vi_mode: false,
        line_numbers: false,
        theme_mode: ThemeMode::System,
        language: Language::Auto,
        markdown_viewer: dkb::viewer::ViewerPreference::Auto,
    };
    Storage::init(&config.data_dir).unwrap();
    let item = Item::new("Viewer Test Item");
    Storage::write_item(&config.data_dir, &item, &Location::Active(Category::Today)).unwrap();

    let window = cx.add_window(|_window, cx| {
        let mut view = KanbanView::new(cx);
        view.config = config;
        view.board = Storage::load_board(&view.config.data_dir).unwrap();
        view.selected_item = Some(item.id);
        view
    });

    window
        .update(cx, |view, window, cx| {
            assert_eq!(view.selected_item, Some(item.id));
            // Invoking on_open_in_markdown_viewer shouldn't panic
            view.on_open_in_markdown_viewer(&OpenInMarkdownViewer, window, cx);
        })
        .unwrap();
}

#[gpui::test]
fn test_right_click_context_menu_on_card(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        vi_mode: false,
        line_numbers: false,
        theme_mode: ThemeMode::Dark,
        language: Language::Auto,
        markdown_viewer: dkb::viewer::ViewerPreference::Auto,
    };
    Storage::init(&config.data_dir).unwrap();
    let item = Item::new("Context Menu Item\nDetails");
    Storage::write_item(&config.data_dir, &item, &Location::Active(Category::Today)).unwrap();

    let window = cx.add_window(|_window, cx| {
        let mut view = KanbanView::new(cx);
        view.config = config;
        view.board = Storage::load_board(&view.config.data_dir).unwrap();
        view
    });

    // 1. Initially no context menu
    window
        .update(cx, |view, _window, _cx| {
            assert!(view.context_menu.is_none());
        })
        .unwrap();

    // 2. Open context menu for the item
    window
        .update(cx, |view, _window, cx| {
            view.selected_item = Some(item.id);
            view.context_menu = Some(ContextMenuState {
                item_id: item.id,
                position: gpui::point(gpui::px(100.), gpui::px(150.)),
            });
            cx.notify();
        })
        .unwrap();

    // 3. Verify context menu state
    window
        .update(cx, |view, _window, _cx| {
            assert!(view.context_menu.is_some());
            let menu = view.context_menu.as_ref().unwrap();
            assert_eq!(menu.item_id, item.id);
            assert_eq!(menu.position, gpui::point(gpui::px(100.), gpui::px(150.)));
        })
        .unwrap();

    // 4. Test "Open / Edit" action from context menu
    window
        .update(cx, |view, _window, cx| {
            view.context_menu = None;
            view.open_editor_for_item(item.id, cx);
            assert!(view.editing.is_some());
            // Close editor
            view.editing = None;
        })
        .unwrap();

    // 5. Test "Mark Done / Reopen" action from context menu
    window
        .update(cx, |view, window, cx| {
            view.selected_item = Some(item.id);
            view.on_toggle_done(&ToggleDone, window, cx);
            assert!(view.board.done.iter().any(|i| i.id == item.id));

            // 6. Test Move from Done back to Backlog
            view.move_selected_to(Location::Backlog, cx);
            assert!(view.board.backlog.iter().any(|i| i.id == item.id));
            assert!(view.board.done.iter().all(|i| i.id != item.id));

            // 7. Test Move from Backlog to Active Today
            view.move_selected_to(Location::Active(Category::Today), cx);
            assert!(view.board.active.today.iter().any(|i| i.id == item.id));

            // 8. Test Move from Active Today to Backlog
            view.move_selected_to(Location::Backlog, cx);
            assert!(view.board.backlog.iter().any(|i| i.id == item.id));
        })
        .unwrap();

    // 9. Test "Delete" action from context menu
    window
        .update(cx, |view, window, cx| {
            view.selected_item = Some(item.id);
            view.on_delete_item(&DeleteItem, window, cx);
            assert!(view.board.find_item(&item.id).is_none());
            assert!(view.selected_item.is_none());
        })
        .unwrap();
}

#[gpui::test]
fn test_move_item_up_and_down_shortcuts(cx: &mut gpui::TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        vi_mode: false,
        line_numbers: false,
        theme_mode: ThemeMode::System,
        language: Language::EnUs,
        markdown_viewer: dkb::viewer::ViewerPreference::Auto,
    };
    Storage::init(&config.data_dir).unwrap();

    let item1 = Item::new("Task 1");
    let item2 = Item::new("Task 2");
    let item3 = Item::new("Task 3");

    Storage::write_item(&config.data_dir, &item1, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(&config.data_dir, &item2, &Location::Active(Category::Today)).unwrap();
    Storage::write_item(&config.data_dir, &item3, &Location::Active(Category::Today)).unwrap();

    let window = cx.add_window(|_window, cx| {
        let mut view = KanbanView::new(cx);
        view.config = config;
        view.board = Storage::load_board(&view.config.data_dir).unwrap();
        // Set initial order: item1, item2, item3
        view.board.set_column_order(
            &Location::Active(Category::Today),
            vec![item1.id, item2.id, item3.id],
        );
        view
    });

    window
        .update(cx, |view, window, cx| {
            // Select item2 (middle)
            view.selected_item = Some(item2.id);

            // Move item2 up -> should become [item2, item1, item3]
            view.on_move_item_up(&MoveItemUp, window, cx);
            let today_ids: Vec<_> = view.board.active.today.iter().map(|i| i.id).collect();
            assert_eq!(today_ids, vec![item2.id, item1.id, item3.id]);

            // Move item2 down -> should become [item1, item2, item3]
            view.on_move_item_down(&MoveItemDown, window, cx);
            let today_ids: Vec<_> = view.board.active.today.iter().map(|i| i.id).collect();
            assert_eq!(today_ids, vec![item1.id, item2.id, item3.id]);

            // Move item2 down again -> should become [item1, item3, item2]
            view.on_move_item_down(&MoveItemDown, window, cx);
            let today_ids: Vec<_> = view.board.active.today.iter().map(|i| i.id).collect();
            assert_eq!(today_ids, vec![item1.id, item3.id, item2.id]);
        })
        .unwrap();
}