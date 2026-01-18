use crate::tui::{App, ChatMessage, MessageType};
use colored::Colorize;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    render_status_bar(frame, app, chunks[0]);
    render_messages(frame, app, chunks[1]);
    render_input_box(frame, app, chunks[2]);

    if app.show_tool_panel {
        render_tool_panel(frame, app, size);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_style = match app.state {
        crate::tui::AppState::Normal => Style::default().fg(Color::Green),
        crate::tui::AppState::Processing => Style::default().fg(Color::Yellow),
        crate::tui::AppState::Error(_) => Style::default().fg(Color::Red),
    };

    let status_text = match app.state {
        crate::tui::AppState::Normal => "就绪",
        crate::tui::AppState::Processing => "处理中...",
        crate::tui::AppState::Error(ref e) => &format!("错误: {}", e),
    };

    let status_line = vec![
        Span::styled(
            " Oxide CLI ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" | "),
        Span::styled(app.model.clone(), Style::default().fg(Color::Blue)),
        Span::raw(" | "),
        Span::styled(
            format!("消息: {}", app.message_count),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" | "),
        Span::styled(status_text, status_style),
    ];

    let paragraph = Paragraph::new(Line::from(status_line))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_messages(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let (prefix, content) = match msg.msg_type {
                MessageType::User => (
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                    msg.content.clone(),
                ),
                MessageType::Assistant => (
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    render_markdown(&msg.content),
                ),
                MessageType::Tool => (
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    format!(
                        "[工具] {}\n{}",
                        msg.tool_name.as_ref().unwrap_or(&"Unknown".to_string()),
                        msg.content
                    ),
                ),
            };

            let prefix_text = match msg.msg_type {
                MessageType::User => "你 >",
                MessageType::Assistant => "AI >",
                MessageType::Tool => "工具 >",
            };

            let lines: Vec<Line> = vec![
                Line::from(Span::styled(prefix_text, prefix)),
                Line::from(""),
                Line::from(content),
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

fn render_input_box(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = vec![Line::from(vec![
        Span::styled("输入> ", Style::default().fg(Color::Yellow)),
        Span::styled(
            &app.input,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    let paragraph = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_tool_panel(frame: &mut Frame, app: &App, area: Rect) {
    let panel_width = 30;
    let panel_x = area.right() - panel_width - 1;

    let panel_area = Rect {
        x: panel_x,
        y: area.top() + 1,
        width: panel_width,
        height: area.height - 2,
    };

    let items: Vec<ListItem> = app
        .tool_status
        .iter()
        .map(|(name, status)| {
            let style = if status.contains("执行中") {
                Style::default().fg(Color::Yellow)
            } else if status.contains("成功") {
                Style::default().fg(Color::Green)
            } else if status.contains("失败") {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(name.clone(), Style::default().fg(Color::Cyan)),
                Span::raw(": "),
                Span::styled(status.clone(), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" 工具状态 ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, panel_area);
}

fn render_markdown(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();

    for line in lines {
        if line.starts_with("# ") {
            result.push_str(&format!("{}\n", line.bold()));
        } else if line.starts_with("## ") {
            result.push_str(&format!("{}\n", line.bold()));
        } else if line.starts_with("```") {
            result.push_str(&format!("{}\n", line.dimmed()));
        } else if line.starts_with("- ") {
            result.push_str(&format!("  • {}\n", &line[2..]));
        } else if !line.trim().is_empty() {
            result.push_str(&format!("{}\n", line));
        } else {
            result.push('\n');
        }
    }

    result
}
