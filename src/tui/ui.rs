use crate::tui::{App, MessageType, OXIDE_LOGO};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    if app.show_welcome {
        render_welcome(frame, app, size);
        return;
    }

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

fn render_welcome(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let title_lines = vec![Line::from(vec![Span::styled(
        "Oxide CLI",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )])];

    let title_paragraph = Paragraph::new(title_lines).alignment(Alignment::Center);
    frame.render_widget(title_paragraph, chunks[0]);

    let welcome_text = vec![
        Line::from(vec![
            Span::styled("版本 ", Style::default().fg(Color::DarkGray)),
            Span::styled(env!("CARGO_PKG_VERSION"), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled("模型 ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.model.clone(), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("会话 ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.session_id.clone(), Style::default().fg(Color::Magenta)),
        ]),
    ];

    let welcome_paragraph = Paragraph::new(welcome_text).alignment(Alignment::Center);
    frame.render_widget(welcome_paragraph, chunks[1]);

    let tips_text = vec![
        Line::from(vec![
            Span::styled("输入内容后回车发送", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+T 切换工具面板", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("/clear 清空 | /exit 退出", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "按任意键开始",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let tips_paragraph = Paragraph::new(tips_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(tips_paragraph, chunks[2]);

    let footer_text = vec![Line::from(vec![Span::styled(
        "Ctrl+C 退出 | /clear 清空 | Ctrl+T 工具面板",
        Style::default().fg(Color::DarkGray),
    )])];

    let footer_paragraph = Paragraph::new(footer_text).alignment(Alignment::Center);
    frame.render_widget(footer_paragraph, chunks[3]);
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
        crate::tui::AppState::Error(ref e) => e,
    };

    let status_line = vec![
        Span::styled(
            "Oxide",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(app.model.clone(), Style::default().fg(Color::Blue)),
        Span::raw("  "),
        Span::styled(app.session_id.clone(), Style::default().fg(Color::Magenta)),
        Span::raw("  "),
        Span::styled(
            format!("消息 {}", app.message_count),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(status_text, status_style),
    ];

    let separator = "─".repeat(area.width as usize);
    let paragraph = Paragraph::new(vec![
        Line::from(status_line),
        Line::from(vec![Span::styled(
            separator,
            Style::default().fg(Color::DarkGray),
        )]),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_messages(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let display_lines = match msg.msg_type {
                MessageType::User => {
                    vec![
                        Line::from(vec![Span::styled(
                            msg.content.clone(),
                            Style::default().fg(Color::White),
                        )]),
                        Line::from(""),
                    ]
                }
                MessageType::Assistant => {
                    let content = render_markdown(&msg.content);

                    let mut lines = Vec::new();

                    for line in content.lines() {
                        lines.push(Line::from(vec![Span::styled(
                            line.to_string(),
                            Style::default().fg(Color::Cyan),
                        )]));
                    }

                    lines.push(Line::from(""));
                    lines
                }
                MessageType::Tool => {
                    let tool_name = msg
                        .tool_name
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("Unknown");
                    let content = format!("{} · {}", tool_name, msg.content);

                    let mut lines = Vec::new();

                    for line in content.lines() {
                        lines.push(Line::from(vec![Span::styled(
                            line.to_string(),
                            Style::default().fg(Color::Yellow),
                        )]));
                    }

                    lines.push(Line::from(""));
                    lines
                }
            };

            ListItem::new(display_lines)
        })
        .collect();

    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}

fn render_input_box(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = vec![Line::from(vec![
        Span::styled("💬 ", Style::default().fg(Color::Yellow)),
        Span::styled("输入: ", Style::default().fg(Color::Gray)),
        Span::styled(
            &app.input,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[Enter]发送 [Ctrl+C]退出",
            Style::default().fg(Color::DarkGray),
        ),
    ])];

    let paragraph = Paragraph::new(input_text).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_tool_panel(frame: &mut Frame, app: &App, area: Rect) {
    let panel_width = 32;
    let panel_x = area.right().saturating_sub(panel_width);

    let panel_area = Rect {
        x: panel_x,
        y: area.top(),
        width: panel_width,
        height: area.height,
    };

    let mut items: Vec<ListItem> = vec![ListItem::new(Line::from(vec![Span::styled(
        "工具状态",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]))];

    let total = app.tool_status.len();
    let completed = app
        .tool_status
        .iter()
        .filter(|(_, status)| status.contains("成功") || status.contains("失败"))
        .count();
    let progress_line = if total == 0 {
        "进度 0/0".to_string()
    } else {
        format!("进度 {}/{}", completed, total)
    };
    items.push(ListItem::new(Line::from(vec![Span::styled(
        progress_line,
        Style::default().fg(Color::DarkGray),
    )])));

    let spinner_frames = ['|', '/', '-', '\\'];
    let spinner = spinner_frames[(app.tick_count as usize) % spinner_frames.len()];

    if app.tool_status.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "暂无工具执行",
            Style::default().fg(Color::DarkGray),
        )])));
    } else {
        for (name, status) in &app.tool_status {
            let status_style = if status.contains("执行中") {
                Style::default().fg(Color::Yellow)
            } else if status.contains("成功") {
                Style::default().fg(Color::Green)
            } else if status.contains("失败") {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Gray)
            };

            let icon = if status.contains("执行中") {
                format!("{} ", spinner)
            } else if status.contains("成功") {
                "✓ ".to_string()
            } else if status.contains("失败") {
                "✗ ".to_string()
            } else {
                "• ".to_string()
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(icon, status_style),
                Span::styled(name.clone(), Style::default().fg(Color::Cyan)),
            ])));
            items.push(ListItem::new(Line::from(vec![Span::styled(
                format!("  {}", status),
                Style::default().fg(Color::Gray),
            )])));
        }
    }

    let recent_logs: Vec<_> = app
        .messages
        .iter()
        .rev()
        .filter(|msg| msg.msg_type == MessageType::Tool)
        .take(4)
        .collect();

    if !recent_logs.is_empty() {
        items.push(ListItem::new(Line::from("")));
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "最近日志",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )])));

        for log in recent_logs.into_iter().rev() {
            let name = log.tool_name.as_deref().unwrap_or("Unknown");
            let content = if log.content.len() > 26 {
                format!("{}...", &log.content[..26])
            } else {
                log.content.clone()
            };
            items.push(ListItem::new(Line::from(vec![
                Span::styled(name, Style::default().fg(Color::Cyan)),
                Span::raw(": "),
                Span::styled(content, Style::default().fg(Color::DarkGray)),
            ])));
        }
    }

    let list = List::new(items);

    frame.render_widget(list, panel_area);
}

fn render_markdown(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut in_code_block = false;

    for line in lines {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            result.push_str(&format!("{}\n", line));
            continue;
        }

        if in_code_block {
            result.push_str(&format!("{}\n", line));
            continue;
        }

        if line.starts_with("# ") {
            result.push_str(&format!("{}\n", &line[2..]));
        } else if line.starts_with("## ") {
            result.push_str(&format!("{}\n", &line[3..]));
        } else if line.starts_with("### ") {
            result.push_str(&format!("{}\n", &line[4..]));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            result.push_str(&format!("  • {}\n", &line[2..]));
        } else if line.starts_with("1. ") {
            result.push_str(&format!("  1. {}\n", &line[3..]));
        } else if line.starts_with("```") {
            result.push_str(&format!("{}\n", line));
        } else if !line.trim().is_empty() {
            result.push_str(&format!("{}\n", line));
        } else {
            result.push('\n');
        }
    }

    result
}
