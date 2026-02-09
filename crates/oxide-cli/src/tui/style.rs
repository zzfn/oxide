use ratatui::style::{Color, Modifier, Style};

pub fn user_message_style() -> Style {
    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
}

pub fn assistant_header_style() -> Style {
    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
}

pub fn thinking_style() -> Style {
    Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
}

pub fn error_style() -> Style {
    Style::default().fg(Color::Red)
}

pub fn warning_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn info_style() -> Style {
    Style::default().fg(Color::Blue)
}

pub fn success_style() -> Style {
    Style::default().fg(Color::Green)
}

pub fn tool_name_style() -> Style {
    Style::default().fg(Color::Green)
}

pub fn code_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn diff_add_style() -> Style {
    Style::default().fg(Color::Green)
}

pub fn diff_remove_style() -> Style {
    Style::default().fg(Color::Red)
}

pub fn diff_header_style() -> Style {
    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

pub fn border_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn status_line_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn spinner_style() -> Style {
    Style::default().fg(Color::Cyan)
}
