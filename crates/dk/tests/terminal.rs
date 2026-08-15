#![allow(clippy::pedantic)]

use dk::terminal::format_list_line;

#[test]
fn test_format_normal_line() {
    let line = format_list_line(0, false, "Buy groceries", 1, 40);
    assert_eq!(line, "  0 Buy groceries");
}

#[test]
fn test_format_current_line() {
    let line = format_list_line(1, true, "Current task", 1, 40);
    assert_eq!(line, "* 1 Current task");
}

#[test]
fn test_right_justify_with_width() {
    let line = format_list_line(3, false, "task", 2, 10);
    assert_eq!(line, "   3 task");
}

#[test]
fn test_truncation() {
    let line = format_list_line(0, false, "a very long title that exceeds the width", 1, 10);
    assert_eq!(line, "  0 a ver");
}

#[test]
fn test_truncation_with_current_marker() {
    let line = format_list_line(0, true, "a very long title that exceeds the width", 1, 10);
    assert_eq!(line, "* 0 a ver");
}
