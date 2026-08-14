use dkb::text_input::TextInputState;
use dkb::vi::{ViMode, ViState};

#[test]
fn test_vi_mode_transitions() {
    let mut state = TextInputState::new("hello world");
    let mut vi = ViState::new();
    assert_eq!(vi.mode, ViMode::Normal);

    // 'i' enters insert mode
    assert!(vi.handle_key("i", &mut state));
    assert_eq!(vi.mode, ViMode::Insert);

    // Escape returns to normal mode
    assert!(vi.handle_key("escape", &mut state));
    assert_eq!(vi.mode, ViMode::Normal);
}

#[test]
fn test_vi_motions_h_j_k_l_0_dollar() {
    let mut state = TextInputState::new("line 1\nline 2\nline 3");
    let mut vi = ViState::new();

    // Start at 0
    assert_eq!(state.cursor_offset(), 0);

    // 'l' moves right
    vi.handle_key("l", &mut state);
    assert_eq!(state.cursor_offset(), 1);

    // 'h' moves left
    vi.handle_key("h", &mut state);
    assert_eq!(state.cursor_offset(), 0);

    // '$' moves to end of current line
    vi.handle_key("$", &mut state);
    assert_eq!(state.cursor_offset(), 6);

    // '0' moves to start of current line
    vi.handle_key("0", &mut state);
    assert_eq!(state.cursor_offset(), 0);

    // 'j' moves down one line
    vi.handle_key("j", &mut state);
    assert_eq!(state.cursor_offset(), 7);

    // 'k' moves up one line
    vi.handle_key("k", &mut state);
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_vi_word_motions() {
    let mut state = TextInputState::new("hello   world  test");
    let mut vi = ViState::new();

    vi.handle_key("w", &mut state);
    assert_eq!(state.cursor_offset(), 8); // start of "world"

    vi.handle_key("w", &mut state);
    assert_eq!(state.cursor_offset(), 15); // start of "test"

    vi.handle_key("b", &mut state);
    assert_eq!(state.cursor_offset(), 8); // start of "world"

    vi.handle_key("b", &mut state);
    assert_eq!(state.cursor_offset(), 0); // start of "hello"
}

#[test]
fn test_vi_insert_variants() {
    let mut state = TextInputState::new("foo");
    let mut vi = ViState::new();

    // 'a' appends after cursor
    vi.handle_key("a", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.cursor_offset(), 1);

    vi.handle_key("escape", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);

    // 'o' opens line below
    vi.handle_key("o", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.content(), "foo\n");
    assert_eq!(state.cursor_offset(), 4);

    vi.handle_key("escape", &mut state);

    // 'O' opens line above current line (which is line 2 after 'o')
    vi.handle_key("O", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.content(), "foo\n\n");
    assert_eq!(state.cursor_offset(), 4);
}

#[test]
fn test_vi_editing_x_dd_yy_p_u_ctrl_r() {
    let mut state = TextInputState::new("hello");
    let mut vi = ViState::new();

    // 'x' deletes char at cursor
    vi.handle_key("x", &mut state);
    assert_eq!(state.content(), "ello");

    // 'u' undos
    vi.handle_key("u", &mut state);
    assert_eq!(state.content(), "hello");

    // 'Ctrl-r' redos
    vi.handle_key("Ctrl-r", &mut state);
    assert_eq!(state.content(), "ello");

    vi.handle_key("u", &mut state);
    assert_eq!(state.content(), "hello");

    // 'dd' deletes current line/content and yanks it
    vi.handle_key("d", &mut state);
    assert_eq!(vi.pending_op, Some('d'));
    vi.handle_key("d", &mut state);
    assert_eq!(vi.pending_op, None);
    assert_eq!(state.content(), "");
    assert_eq!(vi.yank_buffer, Some("hello".to_string()));

    // 'p' pastes yanked content
    vi.handle_key("p", &mut state);
    assert_eq!(state.content(), "hello");

    // 'yy' yanks line without deleting
    vi.handle_key("y", &mut state);
    assert_eq!(vi.pending_op, Some('y'));
    vi.handle_key("y", &mut state);
    assert_eq!(vi.pending_op, None);
    assert_eq!(state.content(), "hello");
    assert_eq!(vi.yank_buffer, Some("hello".to_string()));
}

#[test]
fn test_vi_visual_mode() {
    let mut state = TextInputState::new("hello world");
    let mut vi = ViState::new();

    // 'v' enters visual mode
    vi.handle_key("v", &mut state);
    assert_eq!(vi.mode, ViMode::Visual);

    // 'w' extends selection to next word
    vi.handle_key("w", &mut state);
    assert_eq!(state.selected_range(), 0..6);

    // 'y' yanks visual selection and returns to normal mode
    vi.handle_key("y", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);
    assert_eq!(vi.yank_buffer, Some("hello ".to_string()));
    assert_eq!(state.content(), "hello world");

    // 'v' then 'w' and 'd' deletes selection
    vi.handle_key("0", &mut state);
    vi.handle_key("v", &mut state);
    vi.handle_key("w", &mut state);
    vi.handle_key("d", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);
    assert_eq!(state.content(), "world");
}

#[test]
fn test_vi_escape_variants() {
    let mut state = TextInputState::new("hello");
    let mut vi = ViState::new();

    for esc_key in ["escape", "Escape", "Esc", "\x1b", "\u{1b}", "\u{001b}"] {
        vi.handle_key("i", &mut state);
        assert_eq!(vi.mode, ViMode::Insert);
        assert!(vi.handle_key(esc_key, &mut state));
        assert_eq!(vi.mode, ViMode::Normal);
    }
}
