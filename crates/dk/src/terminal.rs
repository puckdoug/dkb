#[must_use]
pub fn terminal_width() -> usize {
    80
}

#[must_use]
pub fn format_list_line(
    index: usize,
    is_current: bool,
    title: &str,
    index_col_width: usize,
    total_width: usize,
) -> String {
    let prefix = if is_current {
        "* ".to_string()
    } else {
        format!("{index:>index_col_width$} ")
    };
    let available = total_width
        .saturating_sub(prefix.chars().count())
        .saturating_sub(1);
    let truncated: String = title.chars().take(available).collect();
    format!("{prefix}{truncated}")
}
