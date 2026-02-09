use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

use super::style;
use crate::app::CliMode;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// 输入提交结果
pub enum InputResult {
    /// 提交文本
    Submit(String),
    /// 无操作
    None,
    /// Ctrl+C
    Interrupt,
    /// 退出
    Quit,
}

/// 底部面板：状态指示器 + 输入框
pub struct BottomPane {
    /// 输入缓冲
    buffer: String,
    /// 光标字节偏移
    cursor: usize,
    /// 当前模式
    mode: CliMode,
    /// 是否有任务在运行
    task_running: bool,
    /// Spinner 帧
    spinner_frame: usize,
    /// 状态文本
    status_text: Option<String>,
    /// 状态详情
    status_details: Option<String>,
    /// Ctrl+C 计数
    ctrl_c_count: u8,
    /// 历史
    history: Vec<String>,
    history_index: Option<usize>,
    stashed_input: Option<String>,
    /// Token 信息
    token_info: String,
    /// 消息计数
    message_count: usize,
}

impl BottomPane {
    pub fn new() -> Self {
        let history = dirs::home_dir()
            .map(|h| h.join(".oxide_history"))
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|content| {
                content.lines().filter(|l| !l.is_empty()).map(String::from).collect()
            })
            .unwrap_or_default();

        Self {
            buffer: String::new(),
            cursor: 0,
            mode: CliMode::Normal,
            task_running: false,
            spinner_frame: 0,
            status_text: None,
            status_details: None,
            ctrl_c_count: 0,
            history,
            history_index: None,
            stashed_input: None,
            token_info: String::new(),
            message_count: 0,
        }
    }

    pub fn set_mode(&mut self, mode: CliMode) {
        self.mode = mode;
    }

    pub fn set_task_running(&mut self, running: bool) {
        self.task_running = running;
        if running {
            self.status_text = Some("Working".to_string());
        } else {
            self.status_text = None;
            self.status_details = None;
        }
    }

    pub fn update_status(&mut self, text: String, details: Option<String>) {
        self.status_text = Some(text);
        self.status_details = details;
    }

    pub fn set_token_info(&mut self, info: String) {
        self.token_info = info;
    }

    pub fn set_message_count(&mut self, count: usize) {
        self.message_count = count;
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    pub fn reset_ctrl_c(&mut self) {
        self.ctrl_c_count = 0;
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.trim().is_empty()
    }

    /// 处理键盘事件
    pub fn handle_event(&mut self, event: &Event) -> InputResult {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: crossterm::event::KeyEventKind::Press,
                ..
            }) => self.handle_key(*code, *modifiers),
            Event::Paste(text) => {
                self.insert_str(text);
                self.ctrl_c_count = 0;
                InputResult::None
            }
            _ => InputResult::None,
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> InputResult {
        // Reset ctrl_c counter on any other key
        if !(modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c')) {
            self.ctrl_c_count = 0;
        }

        match (modifiers, code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                if self.task_running {
                    return InputResult::Interrupt;
                }
                self.ctrl_c_count += 1;
                if self.ctrl_c_count >= 2 {
                    return InputResult::Quit;
                }
                if !self.buffer.is_empty() {
                    self.clear();
                }
                InputResult::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                if self.buffer.is_empty() {
                    InputResult::Quit
                } else {
                    self.delete_char_forward();
                    InputResult::None
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let text = self.buffer.clone();
                if !text.trim().is_empty() {
                    self.add_to_history(&text);
                }
                self.clear();
                InputResult::Submit(text)
            }
            (KeyModifiers::SHIFT, KeyCode::Enter) | (KeyModifiers::ALT, KeyCode::Enter) => {
                self.insert_char('\n');
                InputResult::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                self.move_to_line_start();
                InputResult::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                self.move_to_line_end();
                InputResult::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.kill_to_line_start();
                InputResult::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                self.kill_to_line_end();
                InputResult::None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('w')) | (KeyModifiers::ALT, KeyCode::Backspace) => {
                self.delete_word_back();
                InputResult::None
            }
            (KeyModifiers::ALT, KeyCode::Char('f')) => {
                self.move_word_right();
                InputResult::None
            }
            (KeyModifiers::ALT, KeyCode::Char('b')) => {
                self.move_word_left();
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.delete_char_back();
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Delete) => {
                self.delete_char_forward();
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
                self.move_left();
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                self.move_right();
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                if self.line_count() > 1 {
                    self.move_up();
                } else {
                    self.history_prev();
                }
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                if self.line_count() > 1 {
                    self.move_down();
                } else {
                    self.history_next();
                }
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Home) => {
                self.move_to_line_start();
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::End) => {
                self.move_to_line_end();
                InputResult::None
            }
            (KeyModifiers::NONE, KeyCode::Esc) => {
                if self.task_running {
                    InputResult::Interrupt
                } else {
                    InputResult::None
                }
            }
            (_, KeyCode::Char(c)) if !modifiers.contains(KeyModifiers::CONTROL) => {
                self.insert_char(c);
                InputResult::None
            }
            _ => InputResult::None,
        }
    }

    // -- Editing --

    fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    fn insert_str(&mut self, s: &str) {
        self.buffer.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.history_index = None;
        self.stashed_input = None;
    }

    fn delete_char_back(&mut self) {
        if self.cursor > 0 {
            let prev = self.prev_boundary(self.cursor);
            self.buffer.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = self.next_boundary(self.cursor);
            self.buffer.drain(self.cursor..next);
        }
    }

    fn delete_word_back(&mut self) {
        if self.cursor == 0 { return; }
        let mut pos = self.cursor;
        while pos > 0 && self.char_before(pos) == Some(' ') {
            pos = self.prev_boundary(pos);
        }
        while pos > 0 && self.char_before(pos) != Some(' ') && self.char_before(pos) != Some('\n') {
            pos = self.prev_boundary(pos);
        }
        self.buffer.drain(pos..self.cursor);
        self.cursor = pos;
    }

    fn move_to_line_start(&mut self) {
        let before = &self.buffer[..self.cursor];
        self.cursor = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    }

    fn move_to_line_end(&mut self) {
        let after = &self.buffer[self.cursor..];
        self.cursor = after.find('\n').map(|i| self.cursor + i).unwrap_or(self.buffer.len());
    }

    fn kill_to_line_start(&mut self) {
        let before = &self.buffer[..self.cursor];
        let start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        self.buffer.drain(start..self.cursor);
        self.cursor = start;
    }

    fn kill_to_line_end(&mut self) {
        let after = &self.buffer[self.cursor..];
        let end = after.find('\n').map(|i| self.cursor + i).unwrap_or(self.buffer.len());
        self.buffer.drain(self.cursor..end);
    }

    fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.prev_boundary(self.cursor);
        }
    }

    fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor = self.next_boundary(self.cursor);
        }
    }

    fn move_word_right(&mut self) {
        let len = self.buffer.len();
        while self.cursor < len && self.char_at(self.cursor) == Some(' ') {
            self.cursor = self.next_boundary(self.cursor);
        }
        while self.cursor < len && self.char_at(self.cursor) != Some(' ') && self.char_at(self.cursor) != Some('\n') {
            self.cursor = self.next_boundary(self.cursor);
        }
    }

    fn move_word_left(&mut self) {
        while self.cursor > 0 && self.char_before(self.cursor) == Some(' ') {
            self.cursor = self.prev_boundary(self.cursor);
        }
        while self.cursor > 0 && self.char_before(self.cursor) != Some(' ') && self.char_before(self.cursor) != Some('\n') {
            self.cursor = self.prev_boundary(self.cursor);
        }
    }

    fn move_up(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line == 0 { return; }
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        let target_col = col.min(lines[line - 1].width());
        let mut offset = 0;
        for i in 0..line - 1 {
            offset += lines[i].len() + 1;
        }
        self.cursor = offset + byte_offset_for_width(lines[line - 1], target_col);
    }

    fn move_down(&mut self) {
        let (line, col) = self.cursor_line_col();
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        if line + 1 >= lines.len() { return; }
        let target_col = col.min(lines[line + 1].width());
        let mut offset = 0;
        for i in 0..=line {
            offset += lines[i].len() + 1;
        }
        self.cursor = offset + byte_offset_for_width(lines[line + 1], target_col);
    }

    fn line_count(&self) -> usize {
        if self.buffer.is_empty() { 1 } else { self.buffer.split('\n').count() }
    }

    fn cursor_line_col(&self) -> (usize, usize) {
        let before = &self.buffer[..self.cursor];
        let line = before.matches('\n').count();
        let last_nl = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = before[last_nl..].width();
        (line, col)
    }

    // -- History --

    fn history_prev(&mut self) {
        if self.history.is_empty() { return; }
        match self.history_index {
            None => {
                self.stashed_input = Some(self.buffer.clone());
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => return,
            Some(ref mut idx) => *idx -= 1,
        }
        if let Some(idx) = self.history_index {
            self.buffer = self.history[idx].clone();
            self.cursor = self.buffer.len();
        }
    }

    fn history_next(&mut self) {
        match self.history_index {
            None => return,
            Some(idx) => {
                if idx + 1 >= self.history.len() {
                    self.history_index = None;
                    self.buffer = self.stashed_input.take().unwrap_or_default();
                    self.cursor = self.buffer.len();
                } else {
                    self.history_index = Some(idx + 1);
                    self.buffer = self.history[idx + 1].clone();
                    self.cursor = self.buffer.len();
                }
            }
        }
    }

    fn add_to_history(&mut self, line: &str) {
        let line = line.to_string();
        if self.history.last().map(|l| l.as_str()) != Some(&line) {
            self.history.push(line);
        }
        self.save_history();
    }

    fn save_history(&self) {
        if let Some(path) = dirs::home_dir().map(|h| h.join(".oxide_history")) {
            let max = 1000;
            let start = self.history.len().saturating_sub(max);
            let content = self.history[start..].join("\n");
            let _ = std::fs::write(path, content);
        }
    }

    // -- Helpers --

    fn char_at(&self, pos: usize) -> Option<char> {
        self.buffer[pos..].chars().next()
    }

    fn char_before(&self, pos: usize) -> Option<char> {
        self.buffer[..pos].chars().next_back()
    }

    fn prev_boundary(&self, pos: usize) -> usize {
        let mut p = pos.saturating_sub(1);
        while p > 0 && !self.buffer.is_char_boundary(p) { p -= 1; }
        p
    }

    fn next_boundary(&self, pos: usize) -> usize {
        let mut p = pos + 1;
        while p < self.buffer.len() && !self.buffer.is_char_boundary(p) { p += 1; }
        p
    }

    /// 计算渲染所需行数
    pub fn desired_height(&self, _width: u16) -> u16 {
        let mut h: u16 = 0;
        // Status line
        if self.task_running {
            h += 1;
            if self.status_details.is_some() {
                h += 1;
            }
            h += 1; // spacer
        }
        // Input box: border(1) + content lines + border(1) + footer(1)
        let content_lines = self.line_count().max(1) as u16;
        h += 1 + content_lines + 1 + 1;
        h
    }

    /// 渲染底部面板
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut y = area.y;

        // Status indicator (when task is running)
        if self.task_running {
            if let Some(status) = &self.status_text {
                let spinner = SPINNER_FRAMES[self.spinner_frame];
                let status_line = Line::from(vec![
                    Span::styled(
                        format!("  {} • ", spinner),
                        style::spinner_style(),
                    ),
                    Span::styled(
                        status.clone(),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        "  (esc to interrupt)",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                let p = Paragraph::new(status_line);
                let status_area = Rect::new(area.x, y, area.width, 1);
                p.render(status_area, buf);
                y += 1;
            }

            if let Some(details) = &self.status_details {
                let detail_line = Line::from(Span::styled(
                    format!("    {}", details),
                    Style::default().fg(Color::DarkGray),
                ));
                let detail_area = Rect::new(area.x, y, area.width, 1);
                Paragraph::new(detail_line).render(detail_area, buf);
                y += 1;
            }

            // Spacer
            y += 1;
        }

        let remaining_height = area.bottom().saturating_sub(y);
        if remaining_height < 3 { return; }

        let input_area = Rect::new(area.x, y, area.width, remaining_height);
        self.render_input_box(input_area, buf);
    }

    fn render_input_box(&self, area: Rect, buf: &mut Buffer) {
        let w = area.width as usize;
        if w < 4 || area.height < 3 { return; }

        let border_color = if self.task_running {
            Color::DarkGray
        } else {
            Color::Cyan
        };
        let border_style = Style::default().fg(border_color);

        // Top border
        let mode_tag = format!("[{}]", self.mode.short_name());
        // Render top border manually to apply style
        let top_area = Rect::new(area.x, area.y, area.width, 1);
        Paragraph::new(Line::from(vec![
            Span::styled("╭", border_style),
            Span::styled(
                mode_tag,
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "─".repeat(w.saturating_sub(6)),
                border_style,
            ),
            Span::styled("╮", border_style),
        ])).render(top_area, buf);

        // Content area
        let content_lines = self.buffer.split('\n').collect::<Vec<_>>();
        let content_height = (area.height as usize).saturating_sub(3); // top + bottom + footer
        let (cursor_line, cursor_col) = self.cursor_line_col();

        for i in 0..content_height {
            let y = area.y + 1 + i as u16;
            let row_area = Rect::new(area.x, y, area.width, 1);

            let text = content_lines.get(i).copied().unwrap_or("");
            let is_cursor_line = i == cursor_line;

            let mut spans = vec![
                Span::styled("│ ", border_style),
            ];

            if text.is_empty() && i == 0 && self.buffer.is_empty() {
                spans.push(Span::styled(
                    "输入消息... (Enter 发送, Shift+Enter 换行)",
                    Style::default().fg(Color::DarkGray),
                ));
            } else {
                if is_cursor_line {
                    // Render text with cursor
                    let before_cursor = &text[..byte_offset_for_col(text, cursor_col)];
                    let at_cursor = text[byte_offset_for_col(text, cursor_col)..].chars().next();
                    let after_start = byte_offset_for_col(text, cursor_col) + at_cursor.map(|c| c.len_utf8()).unwrap_or(0);
                    let after_cursor = &text[after_start..];

                    spans.push(Span::raw(before_cursor.to_string()));
                    if let Some(c) = at_cursor {
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));
                    } else {
                        spans.push(Span::styled(
                            " ",
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));
                    }
                    spans.push(Span::raw(after_cursor.to_string()));
                } else {
                    spans.push(Span::raw(text.to_string()));
                }
            }

            // Right border padding
            let content_width: usize = spans.iter().map(|s| s.content.width()).sum();
            let padding = w.saturating_sub(content_width + 1);
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled("│", border_style));

            Paragraph::new(Line::from(spans)).render(row_area, buf);
        }

        // Bottom border
        let bottom_y = area.y + area.height.saturating_sub(2);
        let bottom_area = Rect::new(area.x, bottom_y, area.width, 1);
        Paragraph::new(Line::from(vec![
            Span::styled("╰", border_style),
            Span::styled(
                "─".repeat(w.saturating_sub(2)),
                border_style,
            ),
            Span::styled("╯", border_style),
        ])).render(bottom_area, buf);

        // Footer line (token info)
        let footer_y = area.y + area.height.saturating_sub(1);
        let footer_area = Rect::new(area.x, footer_y, area.width, 1);
        let mut footer_spans = vec![
            Span::styled(
                format!("  {} msgs", self.message_count),
                style::status_line_style(),
            ),
        ];
        if !self.token_info.is_empty() {
            footer_spans.push(Span::styled(
                format!("  │  {}", self.token_info),
                style::status_line_style(),
            ));
        }
        if self.ctrl_c_count == 1 {
            footer_spans.push(Span::styled(
                "  │  再按一次 Ctrl+C 退出",
                Style::default().fg(Color::Yellow),
            ));
        }
        Paragraph::new(Line::from(footer_spans)).render(footer_area, buf);
    }
}

fn byte_offset_for_width(s: &str, target_width: usize) -> usize {
    let mut w = 0;
    for (i, ch) in s.char_indices() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > target_width { return i; }
        w += cw;
    }
    s.len()
}

fn byte_offset_for_col(s: &str, target_col: usize) -> usize {
    byte_offset_for_width(s, target_col)
}
