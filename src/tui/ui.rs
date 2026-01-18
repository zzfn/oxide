use crate::tui::{App, MessageType, OXIDE_LOGO};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
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
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let logo_lines: Vec<Line> = OXIDE_LOGO
        .lines()
        .map(|line| Line::from(vec![Span::styled(line, Style::default().fg(Color::Cyan))]))
        .collect();

    let logo_paragraph = Paragraph::new(logo_lines).alignment(Alignment::Center);
    frame.render_widget(logo_paragraph, chunks[0]);

    let welcome_text = vec![
        Line::from(vec![
            Span::styled("✨ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "欢迎使用 Oxide CLI!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("版本: ", Style::default().fg(Color::DarkGray)),
            Span::styled(env!("CARGO_PKG_VERSION"), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("模型: ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.model.clone(), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("会话: ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.session_id.clone(), Style::default().fg(Color::Magenta)),
        ]),
    ];

    let welcome_paragraph = Paragraph::new(welcome_text).alignment(Alignment::Center);
    frame.render_widget(welcome_paragraph, chunks[1]);

    let tips_text = vec![
        Line::from(vec![
            Span::styled("💡 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "快速开始提示",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("1. ", Style::default().fg(Color::White)),
            Span::styled("提问、编辑文件或运行命令", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("2. ", Style::default().fg(Color::White)),
            Span::styled("具体描述以获得最佳结果", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("3. ", Style::default().fg(Color::White)),
            Span::styled("输入 /help 查看可用命令", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "按任意键开始",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let tips_paragraph = Paragraph::new(tips_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(tips_paragraph, chunks[2]);

    let footer_text = vec![Line::from(vec![Span::styled(
        format!("Ctrl+C 退出 | /help 查看命令"),
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
        crate::tui::AppState::Normal => "✓ 就绪",
        crate::tui::AppState::Processing => "⟳ 处理中...",
        crate::tui::AppState::Error(ref e) => &format!("✗ {}", e),
    };

    let status_line = vec![
        Span::styled(
            "🤖 Oxide ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(""),
        Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(""),
        Span::styled(
            app.model.clone(),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(""),
        Span::styled(app.session_id.clone(), Style::default().fg(Color::Magenta)),
        Span::raw(""),
        Span::styled(
            format!("消息: {}", app.message_count),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(""),
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
            let display_lines = match msg.msg_type {
                MessageType::User => {
                    let separator = "─".repeat(40);
                    let prefix_style = Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD);
                    let prefix_text = "👤 你";

                    vec![
                        Line::from(vec![
                            Span::styled(separator, Style::default().fg(Color::DarkGray)),
                            Span::raw(" "),
                            Span::styled(prefix_text, prefix_style),
                        ]),
                        Line::from(""),
                        Line::from(vec![Span::styled(
                            msg.content.clone(),
                            Style::default().fg(Color::White),
                        )]),
                        Line::from(""),
                    ]
                }
                MessageType::Assistant => {
                    let separator = "─".repeat(40);
                    let prefix_style = Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD);
                    let prefix_text = "🤖 AI";
                    let content = render_markdown(&msg.content);

                    let mut lines = vec![
                        Line::from(vec![
                            Span::styled(separator, Style::default().fg(Color::DarkGray)),
                            Span::raw(" "),
                            Span::styled(prefix_text, prefix_style),
                        ]),
                        Line::from(""),
                    ];

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
                    let separator = "─".repeat(40);
                    let prefix_style = Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                    let prefix_text = "🔧 工具";
                    let tool_name = msg
                        .tool_name
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("Unknown");
                    let content = format!("{}\n{}", tool_name, msg.content);

                    let mut lines = vec![
                        Line::from(vec![
                            Span::styled(separator, Style::default().fg(Color::DarkGray)),
                            Span::raw(" "),
                            Span::styled(prefix_text, prefix_style),
                        ]),
                        Line::from(""),
                    ];

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

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

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
    let panel_width = 32;
    let panel_x = area.right() - panel_width - 1;

    let panel_area = Rect {
        x: panel_x,
        y: area.top() + 1,
        width: panel_width,
        height: area.height - 2,
    };

    let items: Vec<ListItem> = if app.tool_status.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "暂无工具执行",
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        app.tool_status
            .iter()
            .map(|(name, status)| {
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
                    "⟳ "
                } else if status.contains("成功") {
                    "✓ "
                } else if status.contains("失败") {
                    "✗ "
                } else {
                    "• "
                };

                ListItem::new(Line::from(vec![
                    Span::styled(icon, status_style),
                    Span::styled(name.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw("\n"),
                    Span::styled(format!("  {}", status), Style::default().fg(Color::Gray)),
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .title(" 🔧 工具状态 ")
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
