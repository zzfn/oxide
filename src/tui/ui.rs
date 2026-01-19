use crate::tui::{App, ChatMessage, MessageType};
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

    // 固定显示帮助栏
    let help_height = 1;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // 状态栏（从 3 改为 1）
            Constraint::Min(0),              // 消息区域
            Constraint::Length(1),           // 输入框（从 3 改为 1）
            Constraint::Length(help_height), // 帮助栏
        ])
        .split(size);

    render_status_bar(frame, app, main_chunks[0]);
    render_messages(frame, app, main_chunks[1]);
    render_input_box(frame, app, main_chunks[2]);
    render_help_bar(frame, app, main_chunks[3]);

    // 工具面板（如果需要）
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
            Span::styled("Ctrl+T 工具面板  ↑↓ 滚动", Style::default().fg(Color::Gray)),
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
        "Ctrl+C 退出 | Ctrl+T 工具面板 | /clear 清空",
        Style::default().fg(Color::DarkGray),
    )])];

    let footer_paragraph = Paragraph::new(footer_text).alignment(Alignment::Center);
    frame.render_widget(footer_paragraph, chunks[3]);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    // 左侧内容
    let left_spans = vec![
        Span::styled("Oxide", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(format!("v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::DarkGray)),
        Span::raw(" · "),
        Span::styled(&app.model, Style::default().fg(Color::Cyan)),
    ];

    // 右侧状态
    let (status_icon, status_color) = match app.state {
        crate::tui::AppState::Normal => ("● 就绪", Color::Green),
        crate::tui::AppState::Processing => ("⟳ 处理中", Color::Yellow),
        crate::tui::AppState::Error(_) => ("✗ 错误", Color::Red),
    };

    let right_spans = vec![
        Span::styled(status_icon, Style::default().fg(status_color)),
    ];

    // 使用水平布局分隔左右
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(12)])
        .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(left_spans)),
        chunks[0]
    );
    frame.render_widget(
        Paragraph::new(Line::from(right_spans))
            .alignment(Alignment::Right),
        chunks[1]
    );
}

fn render_messages(frame: &mut Frame, app: &App, area: Rect) {
    // 固定使用边框模式
    let show_borders = true;
    let spacing = 1;

    // 收集所有消息的行
    let mut all_lines = Vec::new();

    for (idx, msg) in app.messages.iter().enumerate() {
        let mut card_lines = render_message_card(msg, show_borders, spacing);
        all_lines.append(&mut card_lines);

        // 如果是 Assistant 消息且有工具状态，内嵌工具状态
        if msg.msg_type == MessageType::Assistant && !app.tool_status.is_empty() {
            // 只在最后一条 Assistant 消息后显示工具状态
            let is_last_assistant = app.messages[idx + 1..]
                .iter()
                .all(|m| m.msg_type != MessageType::Assistant);

            if is_last_assistant {
                let tool_lines = render_inline_tool_status(app);
                all_lines.extend(tool_lines);
            }
        }
    }

    // 直接用 Paragraph 渲染所有行
    let paragraph = Paragraph::new(all_lines)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// 渲染消息卡片
fn render_message_card(msg: &ChatMessage, show_borders: bool, spacing: usize) -> Vec<Line> {
    let (title, title_color) = match msg.msg_type {
        MessageType::User => ("User", Color::Cyan),
        MessageType::Assistant => ("Assistant", Color::Blue),
        MessageType::Tool => ("Tool", Color::Yellow),
    };

    let mut lines = Vec::new();

    if show_borders {
        // 顶部边框
        let top_border = format!("╭─ {} ", title);
        lines.push(Line::from(vec![
            Span::styled(top_border, Style::default().fg(title_color).add_modifier(Modifier::BOLD)),
        ]));

        // 内容 - 支持多行消息
        let content_spans = render_message_content(msg);
        let text: String = content_spans.iter().map(|s| s.content.as_ref()).collect();

        // 按行分割消息内容
        let mut has_content = false;
        for content_line in text.lines() {
            has_content = true;
            lines.push(Line::from(vec![
                Span::raw("│ "),
                Span::styled(content_line.to_string(), get_message_color(&msg.msg_type)),
            ]));
        }

        // 如果内容为空，至少显示一个空行
        if !has_content {
            lines.push(Line::from(vec![Span::raw("│ ")]));
        }

        // 底部边框
        lines.push(Line::from(vec![Span::styled("╰", Style::default().fg(Color::DarkGray))]));
    } else {
        // 无边框模式，只显示内容
        let content_spans = render_message_content(msg);
        let text: String = content_spans.iter().map(|s| s.content.as_ref()).collect();

        let mut has_content = false;
        for content_line in text.lines() {
            has_content = true;
            lines.push(Line::from(vec![
                Span::styled(content_line.to_string(), get_message_color(&msg.msg_type)),
            ]));
        }

        if !has_content {
            lines.push(Line::from(""));
        }
    }

    // 添加间距
    for _ in 0..spacing {
        lines.push(Line::from(""));
    }

    lines
}

/// 渲染消息内容
fn render_message_content(msg: &ChatMessage) -> Vec<Span> {
    match msg.msg_type {
        MessageType::User => {
            vec![Span::styled(
                msg.content.clone(),
                Style::default().fg(Color::White),
            )]
        }
        MessageType::Assistant => {
            vec![Span::styled(
                msg.content.clone(),
                Style::default().fg(Color::Cyan),
            )]
        }
        MessageType::Tool => {
            let tool_name = msg
                .tool_name
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("Unknown");
            let content = format!("{} · {}", tool_name, msg.content);

            vec![Span::styled(content, Style::default().fg(Color::Yellow))]
        }
    }
}

/// 获取消息类型对应的颜色
fn get_message_color(msg_type: &MessageType) -> Style {
    match msg_type {
        MessageType::User => Style::default().fg(Color::White),
        MessageType::Assistant => Style::default().fg(Color::Cyan),
        MessageType::Tool => Style::default().fg(Color::Yellow),
    }
}

/// 渲染内嵌的工具状态
fn render_inline_tool_status(app: &App) -> Vec<Line> {
    let mut lines = Vec::new();

    // 添加空行分隔
    lines.push(Line::from(""));

    let spinner_frames = ['|', '/', '-', '\\'];
    let spinner = spinner_frames[(app.tick_count as usize) % spinner_frames.len()];

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

        lines.push(Line::from(vec![
            Span::raw("│ "),
            Span::styled(icon, status_style),
            Span::styled(name.clone(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(status.clone(), Style::default().fg(Color::Gray)),
        ]));
    }

    // 替换底部边框
    if lines.len() > 1 {
        let last = lines.last_mut().unwrap();
        *last = Line::from(vec![Span::styled("╰", Style::default().fg(Color::DarkGray))]);
    }

    lines
}

fn render_input_box(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = vec![Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(
            &app.input,
            Style::default().fg(Color::White),
        ),
    ])];

    let paragraph = Paragraph::new(input_text)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_help_bar(frame: &mut Frame, _app: &App, area: Rect) {
    let hints = vec![
        ("[Ctrl+T]", "工具面板"),
        ("[↑↓]", "滚动"),
        ("[Ctrl+C]", "退出"),
    ];

    let spans: Vec<Span> = hints
        .into_iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(key, Style::default().fg(Color::Blue)),
                Span::styled(format!("{} ", desc), Style::default().fg(Color::DarkGray)),
            ]
        })
        .collect();

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center),
        area,
    );
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
