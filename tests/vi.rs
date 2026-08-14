#![allow(clippy::pedantic)]

use dkb::text_input::TextInputState;
use dkb::vi::{ExCommand, SearchDirection, ViActionResult, ViMode, ViState, VisualKind};

#[test]
fn test_vi_mode_transitions() {
    let mut state = TextInputState::new("hello world");
    let mut vi = ViState::new();
    assert_eq!(vi.mode, ViMode::Normal);

    // 'i' enters insert mode
    assert_eq!(vi.handle_key("i", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Insert);

    // Escape returns to normal mode
    assert_eq!(vi.handle_key("escape", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Normal);

    // 'v' enters character visual mode
    assert_eq!(vi.handle_key("v", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Visual(VisualKind::Character));

    assert_eq!(vi.handle_key("escape", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Normal);

    // 'V' enters line visual mode
    assert_eq!(vi.handle_key("V", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Visual(VisualKind::Line));

    assert_eq!(vi.handle_key("escape", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Normal);

    // ':' enters command mode
    assert_eq!(vi.handle_key(":", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Command);

    assert_eq!(vi.handle_key("escape", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Normal);

    // '/' enters search forward mode
    assert_eq!(vi.handle_key("/", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Search(SearchDirection::Forward));

    assert_eq!(vi.handle_key("escape", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Normal);

    // '?' enters search backward mode
    assert_eq!(vi.handle_key("?", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Search(SearchDirection::Backward));

    assert_eq!(vi.handle_key("escape", &mut state), ViActionResult::Handled);
    assert_eq!(vi.mode, ViMode::Normal);
}

#[test]
fn test_vi_motions_h_j_k_l_0_dollar_caret() {
    let mut state = TextInputState::new("  line 1\n  line 2\n  line 3");
    let mut vi = ViState::new();

    assert_eq!(state.cursor_offset(), 0);

    // 'l' moves right
    vi.handle_key("l", &mut state);
    assert_eq!(state.cursor_offset(), 1);

    // 'h' moves left
    vi.handle_key("h", &mut state);
    assert_eq!(state.cursor_offset(), 0);

    // '^' or '_' moves to first non-whitespace
    vi.handle_key("^", &mut state);
    assert_eq!(state.cursor_offset(), 2);

    // '$' moves to end of current line
    vi.handle_key("$", &mut state);
    assert_eq!(state.cursor_offset(), 8);

    // '0' moves to start of current line
    vi.handle_key("0", &mut state);
    assert_eq!(state.cursor_offset(), 0);

    // 'j' moves down one line
    vi.handle_key("j", &mut state);
    assert_eq!(state.cursor_offset(), 9);

    // 'k' moves up one line
    vi.handle_key("k", &mut state);
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_vi_counts_with_motions() {
    let mut state = TextInputState::new("line 1\nline 2\nline 3\nline 4\nline 5");
    let mut vi = ViState::new();

    // 3j moves down 3 lines
    vi.handle_key("3", &mut state);
    vi.handle_key("j", &mut state);
    assert_eq!(state.cursor_offset(), 21); // line 4 ("line 4\n" starts at 21)

    // 2k moves up 2 lines
    vi.handle_key("2", &mut state);
    vi.handle_key("k", &mut state);
    assert_eq!(state.cursor_offset(), 7); // line 2

    // 3l moves right 3 chars
    vi.handle_key("3", &mut state);
    vi.handle_key("l", &mut state);
    assert_eq!(state.cursor_offset(), 10);

    // 2h moves left 2 chars
    vi.handle_key("2", &mut state);
    vi.handle_key("h", &mut state);
    assert_eq!(state.cursor_offset(), 8);
}

#[test]
fn test_vi_gg_and_g_motions() {
    let mut state = TextInputState::new("line 1\nline 2\nline 3\nline 4\nline 5");
    let mut vi = ViState::new();

    // 'G' moves to end of file (last line)
    vi.handle_key("G", &mut state);
    assert_eq!(state.cursor_offset(), 28); // line 5 start

    // 'gg' moves to start of file (line 1)
    vi.handle_key("g", &mut state);
    vi.handle_key("g", &mut state);
    assert_eq!(state.cursor_offset(), 0);

    // '3gg' moves to line 3
    vi.handle_key("3", &mut state);
    vi.handle_key("g", &mut state);
    vi.handle_key("g", &mut state);
    assert_eq!(state.cursor_offset(), 14); // line 3 start

    // '2G' moves to line 2
    vi.handle_key("2", &mut state);
    vi.handle_key("G", &mut state);
    assert_eq!(state.cursor_offset(), 7); // line 2 start
}

#[test]
fn test_vi_word_motions() {
    let mut state = TextInputState::new("hello.world   foo_bar  test");
    let mut vi = ViState::new();

    // 'w' moves to next small word start
    vi.handle_key("w", &mut state);
    assert_eq!(state.cursor_offset(), 5); // '.'

    vi.handle_key("w", &mut state);
    assert_eq!(state.cursor_offset(), 6); // 'world'

    vi.handle_key("w", &mut state);
    assert_eq!(state.cursor_offset(), 14); // 'foo_bar'

    // 'e' moves to word end
    vi.handle_key("e", &mut state);
    assert_eq!(state.cursor_offset(), 20); // end of 'foo_bar' ('r')

    // 'b' moves to word start
    vi.handle_key("b", &mut state);
    assert_eq!(state.cursor_offset(), 14); // 'foo_bar' start

    // BIG word motions
    state.move_to(0);
    vi.handle_key("W", &mut state);
    assert_eq!(state.cursor_offset(), 14); // 'foo_bar' (skips "hello.world")

    vi.handle_key("B", &mut state);
    assert_eq!(state.cursor_offset(), 0);

    vi.handle_key("E", &mut state);
    assert_eq!(state.cursor_offset(), 10); // 'd' in "hello.world"
}

#[test]
fn test_vi_matching_bracket_percent() {
    let mut state = TextInputState::new("fn main() {\n    let x = [1, 2, 3];\n}");
    let mut vi = ViState::new();

    // From offset 0, '%' should find first bracket on line '(' or '{' and jump to matching
    vi.handle_key("%", &mut state);
    // Finds '(' at 7 and jumps to ')' at 8
    assert_eq!(state.cursor_offset(), 8);

    // Jump back
    vi.handle_key("%", &mut state);
    assert_eq!(state.cursor_offset(), 7);

    // Move to '{' at 10
    state.move_to(10);
    vi.handle_key("%", &mut state);
    assert_eq!(state.cursor_offset(), 35); // matching '}'

    vi.handle_key("%", &mut state);
    assert_eq!(state.cursor_offset(), 10); // back to '{'
}

#[test]
fn test_vi_find_char_f_f_t_t_repeat() {
    let mut state = TextInputState::new("banana split");
    let mut vi = ViState::new();

    // 'fa' finds first 'a' forward
    vi.handle_key("f", &mut state);
    vi.handle_key("a", &mut state);
    assert_eq!(state.cursor_offset(), 1);

    // ';' repeats find forward
    vi.handle_key(";", &mut state);
    assert_eq!(state.cursor_offset(), 3);

    // ';' repeats again
    vi.handle_key(";", &mut state);
    assert_eq!(state.cursor_offset(), 5);

    // ',' repeats in reverse (backward)
    vi.handle_key(",", &mut state);
    assert_eq!(state.cursor_offset(), 3);

    // 't' till char (1 before)
    state.move_to(0);
    vi.handle_key("t", &mut state);
    vi.handle_key("s", &mut state);
    assert_eq!(state.cursor_offset(), 6); // ' ' before 's' (at 7)
}

#[test]
fn test_vi_paragraph_motions() {
    let mut state = TextInputState::new("para 1\nline 2\n\npara 2\nline 4\n\npara 3");
    let mut vi = ViState::new();

    // '}' jumps to next blank line
    vi.handle_key("}", &mut state);
    assert_eq!(state.cursor_offset(), 14); // blank line index

    vi.handle_key("}", &mut state);
    assert_eq!(state.cursor_offset(), 29); // next blank line index

    // '{' jumps to previous blank line
    vi.handle_key("{", &mut state);
    assert_eq!(state.cursor_offset(), 14);

    vi.handle_key("{", &mut state);
    assert_eq!(state.cursor_offset(), 0);
}

#[test]
fn test_vi_insert_variants() {
    let mut state = TextInputState::new("  foo");
    let mut vi = ViState::new();

    // 'a' appends after cursor
    vi.handle_key("a", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.cursor_offset(), 1);

    vi.handle_key("escape", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);

    // 'I' inserts at first non-whitespace
    state.move_to(0);
    vi.handle_key("I", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.cursor_offset(), 2);

    vi.handle_key("escape", &mut state);

    // 'A' appends at end of line
    state.move_to(0);
    vi.handle_key("A", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.cursor_offset(), 5);

    vi.handle_key("escape", &mut state);

    // 'o' opens line below
    vi.handle_key("o", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.content(), "  foo\n");
    assert_eq!(state.cursor_offset(), 6);

    vi.handle_key("escape", &mut state);

    // 'O' opens line above
    vi.handle_key("O", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.content(), "  foo\n\n");
    assert_eq!(state.cursor_offset(), 6);
}

#[test]
fn test_vi_operators_d_c_y_with_motions_and_counts() {
    let mut state = TextInputState::new("first second third fourth");
    let mut vi = ViState::new();

    // 'dw' deletes "first "
    vi.handle_key("d", &mut state);
    vi.handle_key("w", &mut state);
    assert_eq!(state.content(), "second third fourth");
    assert_eq!(vi.yank_buffer, Some("first ".to_string()));

    // 'd2w' deletes "second third "
    vi.handle_key("d", &mut state);
    vi.handle_key("2", &mut state);
    vi.handle_key("w", &mut state);
    assert_eq!(state.content(), "fourth");

    // 'u' undos
    vi.handle_key("u", &mut state);
    assert_eq!(state.content(), "second third fourth");

    // 'cw' changes "second " and enters insert mode
    vi.handle_key("c", &mut state);
    vi.handle_key("w", &mut state);
    assert_eq!(vi.mode, ViMode::Insert);
    assert_eq!(state.content(), "third fourth");

    vi.handle_key("escape", &mut state);

    // 'D' deletes to end of line
    vi.handle_key("D", &mut state);
    assert_eq!(state.content(), "");
}

#[test]
fn test_vi_linewise_operations_dd_yy_p_p() {
    let mut state = TextInputState::new("line 1\nline 2\nline 3");
    let mut vi = ViState::new();

    // 'dd' deletes line 1 linewise
    vi.handle_key("d", &mut state);
    vi.handle_key("d", &mut state);
    assert_eq!(state.content(), "line 2\nline 3");
    assert_eq!(vi.yank_buffer, Some("line 1\n".to_string()));
    assert!(vi.is_linewise_yank);

    // 'p' pastes linewise below current line
    vi.handle_key("p", &mut state);
    assert_eq!(state.content(), "line 2\nline 1\nline 3");

    // 'P' pastes linewise above current line
    vi.handle_key("P", &mut state);
    assert_eq!(state.content(), "line 2\nline 1\nline 1\nline 3");

    // '2dd' deletes 2 lines
    vi.handle_key("2", &mut state);
    vi.handle_key("d", &mut state);
    vi.handle_key("d", &mut state);
    assert_eq!(state.content(), "line 2\nline 3");
}

#[test]
fn test_vi_misc_editing_x_r_tilde_j_indent() {
    let mut state = TextInputState::new("hello\nworld");
    let mut vi = ViState::new();

    // '3x' deletes 3 chars
    vi.handle_key("3", &mut state);
    vi.handle_key("x", &mut state);
    assert_eq!(state.content(), "lo\nworld");

    // 'r' replaces char
    state.move_to(0);
    vi.handle_key("r", &mut state);
    vi.handle_key("z", &mut state);
    assert_eq!(state.content(), "zo\nworld");

    // '~' toggles case
    state.move_to(0);
    vi.handle_key("~", &mut state);
    assert_eq!(state.content(), "Zo\nworld");

    // 'J' joins lines
    state.move_to(0);
    vi.handle_key("J", &mut state);
    assert_eq!(state.content(), "Zo world");

    // '>>' indents line
    vi.handle_key(">", &mut state);
    vi.handle_key(">", &mut state);
    assert_eq!(state.content(), "    Zo world");

    // '<<' outdents line
    vi.handle_key("<", &mut state);
    vi.handle_key("<", &mut state);
    assert_eq!(state.content(), "Zo world");
}

#[test]
fn test_vi_text_objects() {
    let mut state = TextInputState::new("hello \"world test\" (foo bar) end");
    let mut vi = ViState::new();

    // ci" inside quotes
    state.move_to(9); // inside "world test"
    vi.handle_key("c", &mut state);
    vi.handle_key("i", &mut state);
    vi.handle_key("\"", &mut state);
    assert_eq!(state.content(), "hello \"\" (foo bar) end");
    assert_eq!(vi.mode, ViMode::Insert);

    vi.handle_key("escape", &mut state);

    // da( around parens
    state.move_to(11); // inside (foo bar)
    vi.handle_key("d", &mut state);
    vi.handle_key("a", &mut state);
    vi.handle_key("(", &mut state);
    assert_eq!(state.content(), "hello \"\"  end");

    // diw inside word
    state.move_to(0); // in "hello"
    vi.handle_key("d", &mut state);
    vi.handle_key("i", &mut state);
    vi.handle_key("w", &mut state);
    assert_eq!(state.content(), " \"\"  end");
}

#[test]
fn test_vi_visual_character_and_line_modes() {
    let mut state = TextInputState::new("first line\nsecond line\nthird line");
    let mut vi = ViState::new();

    // Visual Character mode
    vi.handle_key("v", &mut state);
    assert_eq!(vi.mode, ViMode::Visual(VisualKind::Character));
    vi.handle_key("w", &mut state);
    assert_eq!(state.selected_range(), 0..6);

    vi.handle_key("d", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);
    assert_eq!(state.content(), "line\nsecond line\nthird line");

    // Visual Line mode
    state.move_to(0);
    vi.handle_key("V", &mut state);
    assert_eq!(vi.mode, ViMode::Visual(VisualKind::Line));
    vi.handle_key("j", &mut state);
    // Selection should cover first two lines
    assert_eq!(state.selected_range(), 0..17);

    vi.handle_key("d", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);
    assert_eq!(state.content(), "third line");
}

#[test]
fn test_vi_search_forward_and_backward() {
    let mut state = TextInputState::new("apple banana apple cherry apple");
    let mut vi = ViState::new();

    // '/' search for "apple"
    vi.handle_key("/", &mut state);
    assert_eq!(vi.mode, ViMode::Search(SearchDirection::Forward));
    for ch in "banana".chars() {
        vi.handle_key(&ch.to_string(), &mut state);
    }
    vi.handle_key("enter", &mut state);
    assert_eq!(vi.mode, ViMode::Normal);
    assert_eq!(state.cursor_offset(), 6); // start of "banana"

    // '*' searches word under cursor forward
    state.move_to(0); // on "apple"
    vi.handle_key("*", &mut state);
    assert_eq!(state.cursor_offset(), 13); // second "apple"

    // 'n' repeats next match
    vi.handle_key("n", &mut state);
    assert_eq!(state.cursor_offset(), 26); // third "apple"

    // 'N' repeats previous match
    vi.handle_key("N", &mut state);
    assert_eq!(state.cursor_offset(), 13);
}

#[test]
fn test_vi_ex_command_line_and_execution() {
    let mut state = TextInputState::new("line 1\nline 2\nline 3");
    let mut vi = ViState::new();

    // ':w'
    vi.handle_key(":", &mut state);
    assert_eq!(vi.mode, ViMode::Command);
    vi.handle_key("w", &mut state);
    let res = vi.handle_key("enter", &mut state);
    assert_eq!(res, ViActionResult::Save);
    assert_eq!(vi.mode, ViMode::Normal);

    // ':q'
    vi.handle_key(":", &mut state);
    vi.handle_key("q", &mut state);
    let res = vi.handle_key("enter", &mut state);
    assert_eq!(res, ViActionResult::Close { force: false });

    // ':q!'
    vi.handle_key(":", &mut state);
    vi.handle_key("q", &mut state);
    vi.handle_key("!", &mut state);
    let res = vi.handle_key("enter", &mut state);
    assert_eq!(res, ViActionResult::Close { force: true });

    // ':wq'
    vi.handle_key(":", &mut state);
    vi.handle_key("w", &mut state);
    vi.handle_key("q", &mut state);
    let res = vi.handle_key("enter", &mut state);
    assert_eq!(res, ViActionResult::SaveAndClose);

    // ':2' jump to line 2
    vi.handle_key(":", &mut state);
    vi.handle_key("2", &mut state);
    let res = vi.handle_key("enter", &mut state);
    assert_eq!(res, ViActionResult::Handled);
    assert_eq!(state.cursor_offset(), 7); // line 2

    // ':%s/line/row/g'
    vi.handle_key(":", &mut state);
    for ch in "%s/line/row/g".chars() {
        vi.handle_key(&ch.to_string(), &mut state);
    }
    let res = vi.handle_key("enter", &mut state);
    assert_eq!(res, ViActionResult::Handled);
    assert_eq!(state.content(), "row 1\nrow 2\nrow 3");

    // ':d' delete line
    state.move_to(0);
    vi.handle_key(":", &mut state);
    vi.handle_key("d", &mut state);
    let res = vi.handle_key("enter", &mut state);
    assert_eq!(res, ViActionResult::Handled);
    assert_eq!(state.content(), "row 2\nrow 3");
}

#[test]
fn test_vi_ex_command_parser_direct() {
    assert_eq!(ViState::parse_ex_command("w"), Some(ExCommand::Write));
    assert_eq!(ViState::parse_ex_command(":write"), Some(ExCommand::Write));
    assert_eq!(ViState::parse_ex_command("q"), Some(ExCommand::Quit { force: false }));
    assert_eq!(ViState::parse_ex_command("q!"), Some(ExCommand::Quit { force: true }));
    assert_eq!(ViState::parse_ex_command(":quit!"), Some(ExCommand::Quit { force: true }));
    assert_eq!(ViState::parse_ex_command("wq"), Some(ExCommand::WriteQuit));
    assert_eq!(ViState::parse_ex_command("x"), Some(ExCommand::WriteQuit));
    assert_eq!(ViState::parse_ex_command("42"), Some(ExCommand::GotoLine(42)));
    assert_eq!(ViState::parse_ex_command(":d"), Some(ExCommand::DeleteLine));
    assert_eq!(
        ViState::parse_ex_command("%s/foo/bar/gi"),
        Some(ExCommand::Substitute {
            pattern: "foo".to_string(),
            replacement: "bar".to_string(),
            global: true,
            ignore_case: true,
        })
    );
}

#[test]
fn test_vi_escape_variants() {
    let mut state = TextInputState::new("hello");
    let mut vi = ViState::new();

    for esc_key in ["escape", "Escape", "Esc", "\x1b", "\u{1b}", "\u{001b}"] {
        vi.handle_key("i", &mut state);
        assert_eq!(vi.mode, ViMode::Insert);
        assert_eq!(vi.handle_key(esc_key, &mut state), ViActionResult::Handled);
        assert_eq!(vi.mode, ViMode::Normal);
    }
}
