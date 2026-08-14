use dkb::text_input::TextInputState;

// -- Construction --

#[test]
fn test_new_with_content() {
    let state = TextInputState::new("hello");
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 0);
    assert!(state.selected_range().is_empty());
}

#[test]
fn test_new_empty() {
    let state = TextInputState::new("");
    assert_eq!(state.content(), "");
    assert_eq!(state.cursor_offset(), 0);
}

// -- Cursor movement --

#[test]
fn test_move_right() {
    let mut state = TextInputState::new("abc");
    state.move_right();
    assert_eq!(state.cursor_offset(), 1);
    state.move_right();
    assert_eq!(state.cursor_offset(), 2);
    state.move_right();
    assert_eq!(state.cursor_offset(), 3);
    state.move_right();
    assert_eq!(state.cursor_offset(), 3);
}

#[test]
fn test_move_left() {
    let mut state = TextInputState::new("abc");
    state.move_to(3);
    state.move_left();
    assert_eq!(state.cursor_offset(), 2);
    state.move_left();
    assert_eq!(state.cursor_offset(), 1);
    state.move_left();
    assert_eq!(state.cursor_offset(), 0);
    state.move_left();
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_move_to_home_end() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    assert_eq!(state.cursor_offset(), 5);
    state.move_to_home();
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_move_right_with_multibyte() {
    let mut state = TextInputState::new("café");
    state.move_right();
    assert_eq!(state.cursor_offset(), 1);
    state.move_right();
    assert_eq!(state.cursor_offset(), 2);
    state.move_right();
    assert_eq!(state.cursor_offset(), 3);
    state.move_right();
    assert_eq!(state.cursor_offset(), 5);
}

// -- Text insertion --

#[test]
fn test_insert_at_cursor() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 5);
}

#[test]
fn test_insert_in_middle() {
    let mut state = TextInputState::new("hllo");
    state.move_right();
    state.insert("e");
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 2);
}

#[test]
fn test_insert_replaces_selection() {
    let mut state = TextInputState::new("hello world");
    state.move_to(0);
    state.select_to(5);
    state.insert("goodbye");
    assert_eq!(state.content(), "goodbye world");
    assert_eq!(state.cursor_offset(), 7);
}

#[test]
fn test_insert_multiline() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.insert("\nworld");
    assert_eq!(state.content(), "hello\nworld");
    assert_eq!(state.cursor_offset(), 11);
}

// -- Backspace / Delete --

#[test]
fn test_backspace() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.backspace();
    assert_eq!(state.content(), "hell");
    assert_eq!(state.cursor_offset(), 4);
}

#[test]
fn test_backspace_at_start() {
    let mut state = TextInputState::new("hello");
    state.backspace();
    assert_eq!(state.content(), "hello");
}

#[test]
fn test_backspace_deletes_selection() {
    let mut state = TextInputState::new("hello world");
    state.select_to(5);
    state.backspace();
    assert_eq!(state.content(), " world");
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_delete_forward() {
    let mut state = TextInputState::new("hello");
    state.delete();
    assert_eq!(state.content(), "ello");
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_delete_at_end() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.delete();
    assert_eq!(state.content(), "hello");
}

// -- Selection --

#[test]
fn test_select_right() {
    let mut state = TextInputState::new("hello");
    state.select_right();
    assert_eq!(state.selected_range(), 0..1);
    state.select_right();
    assert_eq!(state.selected_range(), 0..2);
}

#[test]
fn test_select_left() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.select_left();
    assert_eq!(state.selected_range(), 4..5);
}

#[test]
fn test_select_all() {
    let mut state = TextInputState::new("hello");
    state.select_all();
    assert_eq!(state.selected_range(), 0..5);
}

#[test]
fn test_move_collapses_selection() {
    let mut state = TextInputState::new("hello");
    state.select_all();
    state.move_right();
    assert!(state.selected_range().is_empty());
    assert_eq!(state.cursor_offset(), 5);
}

// -- Replace range --

#[test]
fn test_replace_range() {
    let mut state = TextInputState::new("hello world");
    state.replace_range(0..5, "goodbye");
    assert_eq!(state.content(), "goodbye world");
    assert_eq!(state.cursor_offset(), 7);
}

// -- Undo / Redo --

#[test]
fn test_undo_insert() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    assert_eq!(state.content(), "hello");
    state.undo();
    assert_eq!(state.content(), "");
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_redo_after_undo() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    state.undo();
    assert_eq!(state.content(), "");
    state.redo();
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 5);
}

#[test]
fn test_undo_backspace() {
    let mut state = TextInputState::new("hello");
    state.move_to_end();
    state.backspace();
    assert_eq!(state.content(), "hell");
    state.undo();
    assert_eq!(state.content(), "hello");
    assert_eq!(state.cursor_offset(), 5);
}

#[test]
fn test_multiple_undos() {
    let mut state = TextInputState::new("");
    state.insert("a");
    state.insert("b");
    state.insert("c");
    assert_eq!(state.content(), "abc");
    state.undo();
    assert_eq!(state.content(), "ab");
    state.undo();
    assert_eq!(state.content(), "a");
    state.undo();
    assert_eq!(state.content(), "");
}

#[test]
fn test_new_edit_clears_redo_stack() {
    let mut state = TextInputState::new("");
    state.insert("hello");
    state.undo();
    assert_eq!(state.content(), "");
    state.insert("world");
    state.redo();
    assert_eq!(state.content(), "world");
}

// -- UTF-16 conversion --

#[test]
fn test_utf16_offset_ascii() {
    let state = TextInputState::new("hello");
    assert_eq!(state.offset_to_utf16(3), 3);
    assert_eq!(state.offset_from_utf16(3), 3);
}

#[test]
fn test_utf16_offset_multibyte() {
    let state = TextInputState::new("€");
    assert_eq!(state.offset_to_utf16(3), 1);
    assert_eq!(state.offset_from_utf16(1), 3);
}

// -- Word boundaries --

#[test]
fn test_word_start_in_middle_of_word() {
    let state = TextInputState::new("hello world");
    assert_eq!(state.word_start(3), 0);
}

#[test]
fn test_word_end_from_beginning() {
    let state = TextInputState::new("hello world");
    assert_eq!(state.word_end(0), 5);
}

#[test]
fn test_select_word_at() {
    let mut state = TextInputState::new("hello world");
    state.select_word_at(3);
    assert_eq!(state.selected_range(), 0..5);
}

#[test]
fn test_multiline_up_down_navigation() {
    let mut state = TextInputState::new("line one\nline two\nline three");
    state.move_to(2); // 'n' in "line one"
    state.move_down();
    assert_eq!(state.cursor_offset(), 11); // 'n' in "line two"
    state.move_down();
    assert_eq!(state.cursor_offset(), 20); // 'n' in "line three"
    state.move_up();
    assert_eq!(state.cursor_offset(), 11);
    state.move_up();
    assert_eq!(state.cursor_offset(), 2);
}
