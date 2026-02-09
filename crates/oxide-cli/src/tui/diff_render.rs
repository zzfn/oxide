use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use super::style;

/// 渲染 diff cell
pub fn render_diff_cell(
    file_path: &str,
    additions: usize,
    deletions: usize,
    diff_text: &str,
    _width: u16,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from("")];

    // File header
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            file_path.to_string(),
            style::diff_header_style(),
        ),
        Span::styled(
            format!("  +{} -{}", additions, deletions),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    // Diff lines
    for line_text in diff_text.lines() {
        let (styled_line, line_style) = if line_text.starts_with('+') && !line_text.starts_with("+++") {
            (line_text.to_string(), style::diff_add_style())
        } else if line_text.starts_with('-') && !line_text.starts_with("---") {
            (line_text.to_string(), style::diff_remove_style())
        } else if line_text.starts_with("@@") {
            (line_text.to_string(), Style::default().fg(Color::Cyan))
        } else {
            (line_text.to_string(), Style::default().fg(Color::DarkGray))
        };

        lines.push(Line::from(Span::styled(
            format!("  {}", styled_line),
            line_style,
        )));
    }

    lines
}

/// 从文件的旧/新内容生成 unified diff 并渲染
pub fn render_unified_diff(
    file_path: &str,
    old_content: &str,
    new_content: &str,
    width: u16,
) -> Vec<Line<'static>> {
    let patch = diffy::create_patch(old_content, new_content);
    let diff_text = format!("{}", patch);

    let additions = diff_text
        .lines()
        .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
        .count();
    let deletions = diff_text
        .lines()
        .filter(|l| l.starts_with('-') && !l.starts_with("---"))
        .count();

    render_diff_cell(file_path, additions, deletions, &diff_text, width)
}
