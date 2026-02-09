use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::markdown_render::render_markdown;
use super::style;

/// 对话历史中的一个 cell（渲染单元）
#[derive(Debug, Clone)]
pub enum HistoryCell {
    /// 用户消息
    UserMessage { text: String },
    /// 助手消息（markdown 渲染）
    AssistantMessage { text: String },
    /// 助手推理/思考
    Reasoning { text: String },
    /// 工具调用（命令执行）
    ToolCall {
        name: String,
        arguments: String,
        output: Option<String>,
        success: Option<bool>,
        elapsed: Option<std::time::Duration>,
    },
    /// 错误
    Error { message: String },
    /// 警告
    Warning { message: String },
    /// 信息
    Info { message: String },
    /// 欢迎消息
    Welcome,
    /// Diff 摘要
    DiffSummary {
        file_path: String,
        additions: usize,
        deletions: usize,
        diff_text: String,
    },
    /// 空行
    Separator,
}

impl HistoryCell {
    /// 渲染为 ratatui Lines
    pub fn render_lines(&self, width: u16) -> Vec<Line<'static>> {
        match self {
            HistoryCell::UserMessage { text } => {
                let mut lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "> ",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )),
                ];
                for line_text in text.lines() {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(
                            line_text.to_string(),
                            style::user_message_style(),
                        ),
                    ]));
                }
                lines
            }
            HistoryCell::AssistantMessage { text } => {
                let mut lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Assistant",
                        style::assistant_header_style(),
                    )),
                    Line::from(""),
                ];
                let md_lines = render_markdown(text, Some(width as usize - 2));
                for md_line in md_lines {
                    lines.push(md_line);
                }
                lines
            }
            HistoryCell::Reasoning { text } => {
                let mut lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Thinking:",
                        style::thinking_style(),
                    )),
                ];
                for line_text in text.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", line_text),
                        style::thinking_style(),
                    )));
                }
                lines
            }
            HistoryCell::ToolCall {
                name,
                arguments,
                output,
                success,
                elapsed,
            } => {
                let mut lines = vec![Line::from("")];

                // Tool header
                let status_icon = match success {
                    Some(true) => Span::styled("⏺ ", style::success_style()),
                    Some(false) => Span::styled("⏺ ", style::error_style()),
                    None => Span::styled("⏺ ", Style::default().fg(Color::DarkGray)),
                };

                let mut header_spans = vec![status_icon];
                header_spans.push(Span::styled(
                    name.clone(),
                    style::tool_name_style(),
                ));

                // 截断参数显示
                let args_display = if arguments.len() > 60 {
                    format!("({}...)", &arguments[..60])
                } else {
                    format!("({})", arguments)
                };
                header_spans.push(Span::styled(
                    args_display,
                    Style::default().fg(Color::DarkGray),
                ));

                if let Some(dur) = elapsed {
                    header_spans.push(Span::styled(
                        format!(" [{:.1}s]", dur.as_secs_f64()),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                lines.push(Line::from(header_spans));

                // Output (truncated)
                if let Some(out) = output {
                    let max_lines = 15;
                    let out_lines: Vec<&str> = out.lines().collect();
                    let display_count = out_lines.len().min(max_lines);
                    for line_text in &out_lines[..display_count] {
                        let truncated = if line_text.len() > width as usize - 4 {
                            format!("⎿  {}...", &line_text[..width as usize - 7])
                        } else {
                            format!("⎿  {}", line_text)
                        };
                        lines.push(Line::from(Span::styled(
                            truncated,
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                    if out_lines.len() > max_lines {
                        lines.push(Line::from(Span::styled(
                            format!("⎿  ... ({} more lines)", out_lines.len() - max_lines),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                lines
            }
            HistoryCell::Error { message } => {
                vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Error: ", style::error_style()),
                        Span::styled(message.clone(), Style::default().fg(Color::Red)),
                    ]),
                ]
            }
            HistoryCell::Warning { message } => {
                vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Warning: ", style::warning_style()),
                        Span::styled(message.clone(), Style::default().fg(Color::Yellow)),
                    ]),
                ]
            }
            HistoryCell::Info { message } => {
                vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Info: ", style::info_style()),
                        Span::styled(message.clone(), Style::default().fg(Color::White)),
                    ]),
                ]
            }
            HistoryCell::Welcome => {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "╭─────────────────────────────────────╮",
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        "│         Oxide - AI 编程助手         │",
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        "╰─────────────────────────────────────╯",
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::Green)),
                        Span::raw("输入问题开始对话"),
                    ]),
                    Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::Green)),
                        Span::raw("输入 "),
                        Span::styled("/help", Style::default().fg(Color::Yellow)),
                        Span::raw(" 查看帮助"),
                    ]),
                    Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::Green)),
                        Span::raw("按 "),
                        Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
                        Span::raw(" 两次退出"),
                    ]),
                    Line::from(""),
                ]
            }
            HistoryCell::DiffSummary {
                file_path,
                additions,
                deletions,
                diff_text,
            } => {
                super::diff_render::render_diff_cell(file_path, *additions, *deletions, diff_text, width)
            }
            HistoryCell::Separator => {
                vec![Line::from("")]
            }
        }
    }

    /// 计算渲染所需的行数
    pub fn height(&self, width: u16) -> u16 {
        self.render_lines(width).len() as u16
    }
}
