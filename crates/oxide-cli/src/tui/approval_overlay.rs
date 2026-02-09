use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};

/// 审批请求
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub id: String,
    pub tool_name: String,
    pub description: String,
}

/// 审批结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalResult {
    Approved,
    Denied,
    AlwaysApprove,
    Pending,
}

/// 审批模态框
pub struct ApprovalOverlay {
    request: ApprovalRequest,
    result: ApprovalResult,
    selected: usize,
}

impl ApprovalOverlay {
    pub fn new(request: ApprovalRequest) -> Self {
        Self {
            request,
            result: ApprovalResult::Pending,
            selected: 0,
        }
    }

    pub fn request(&self) -> &ApprovalRequest {
        &self.request
    }

    pub fn result(&self) -> ApprovalResult {
        self.result
    }

    pub fn is_complete(&self) -> bool {
        self.result != ApprovalResult::Pending
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Char('y') | KeyCode::Char('Y')) => {
                self.result = ApprovalResult::Approved;
            }
            (KeyModifiers::NONE, KeyCode::Char('n') | KeyCode::Char('N')) => {
                self.result = ApprovalResult::Denied;
            }
            (KeyModifiers::NONE, KeyCode::Char('a') | KeyCode::Char('A')) => {
                self.result = ApprovalResult::AlwaysApprove;
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.result = match self.selected {
                    0 => ApprovalResult::Approved,
                    1 => ApprovalResult::AlwaysApprove,
                    _ => ApprovalResult::Denied,
                };
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                self.selected = self.selected.saturating_sub(1);
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                if self.selected < 2 {
                    self.selected += 1;
                }
            }
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.result = ApprovalResult::Denied;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.result = ApprovalResult::Denied;
            }
            _ => {}
        }
    }

    /// 渲染审批弹窗
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // 计算居中弹窗位置
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 12.min(area.height.saturating_sub(2));
        let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        // 清除背景
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(
                " 权限确认 ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        // Content
        let mut lines: Vec<Line<'static>> = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("工具: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                self.request.tool_name.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // 截断描述
        let desc_lines: Vec<&str> = self.request.description.lines().collect();
        for (i, desc_line) in desc_lines.iter().enumerate() {
            if i >= 3 {
                lines.push(Line::from(Span::styled(
                    "  ...",
                    Style::default().fg(Color::DarkGray),
                )));
                break;
            }
            let truncated = if desc_line.len() > popup_width as usize - 4 {
                format!("  {}...", &desc_line[..popup_width as usize - 7])
            } else {
                format!("  {}", desc_line)
            };
            lines.push(Line::from(Span::styled(
                truncated,
                Style::default().fg(Color::White),
            )));
        }

        lines.push(Line::from(""));

        // Options
        let options = [
            ("y", "允许本次"),
            ("a", "始终允许"),
            ("n", "拒绝"),
        ];

        for (i, (key, label)) in options.iter().enumerate() {
            let is_selected = i == self.selected;
            let prefix = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(vec![
                Span::styled(prefix.to_string(), style),
                Span::styled(format!("[{}] ", key), Style::default().fg(Color::Cyan)),
                Span::styled(label.to_string(), style),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
