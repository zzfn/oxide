//! 多行输入编辑器
//!
//! 基于 crossterm 事件的多行编辑器，支持自动换行、历史记录和 Emacs 快捷键。

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

/// 输入信号
pub enum InputSignal {
    Line(String),
    CtrlC,
    CtrlD,
    TabComplete,
    ClearScreen,
}

/// 多行编辑器
pub struct LineEditor {
    buffer: String,
    /// 光标在 buffer 中的字节偏移
    cursor: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    history_path: Option<PathBuf>,
    stashed_input: Option<String>,
}

impl LineEditor {
    pub fn new() -> Self {
        let history_path = dirs::home_dir().map(|h| h.join(".oxide_history"));
        let history = history_path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|content| {
                content
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Self {
            buffer: String::new(),
            cursor: 0,
            history,
            history_index: None,
            history_path,
            stashed_input: None,
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.history_index = None;
        self.stashed_input = None;
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.trim().is_empty()
    }

    /// 获取光标所在的行号和列号（基于 '\n' 分割）
    pub fn cursor_line_col(&self) -> (usize, usize) {
        let before = &self.buffer[..self.cursor];
        let line = before.matches('\n').count();
        let last_nl = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = before[last_nl..].width();
        (line, col)
    }

    /// 获取所有行
    pub fn lines(&self) -> Vec<&str> {
        if self.buffer.is_empty() {
            vec![""]
        } else {
            self.buffer.split('\n').collect()
        }
    }

    /// 行数
    pub fn line_count(&self) -> usize {
        self.lines().len()
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<InputSignal> {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: crossterm::event::KeyEventKind::Press,
                ..
            }) => self.handle_key(*code, *modifiers),
            Event::Paste(text) => {
                self.insert_str(text);
                None
            }
            _ => None,
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Option<InputSignal> {
        match (modifiers, code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(InputSignal::CtrlC),
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                if self.buffer.is_empty() {
                    Some(InputSignal::CtrlD)
                } else {
                    self.delete_char_forward();
                    None
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(InputSignal::ClearScreen),
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                self.move_to_line_start();
                None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                self.move_to_line_end();
                None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.kill_to_line_start();
                None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                self.kill_to_line_end();
                None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('w')) | (KeyModifiers::ALT, KeyCode::Backspace) => {
                self.delete_word_back();
                None
            }
            (KeyModifiers::ALT, KeyCode::Char('f')) => {
                self.move_word_right();
                None
            }
            (KeyModifiers::ALT, KeyCode::Char('b')) => {
                self.move_word_left();
                None
            }
            // Enter 提交，Shift+Enter / Alt+Enter 换行
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let line = self.buffer.clone();
                if !line.trim().is_empty() {
                    self.add_to_history(&line);
                }
                self.clear();
                Some(InputSignal::Line(line))
            }
            (KeyModifiers::SHIFT, KeyCode::Enter) | (KeyModifiers::ALT, KeyCode::Enter) => {
                self.insert_char('\n');
                None
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.delete_char_back();
                None
            }
            (KeyModifiers::NONE, KeyCode::Delete) => {
                self.delete_char_forward();
                None
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
                self.move_left();
                None
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                self.move_right();
                None
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                if self.line_count() > 1 {
                    self.move_up();
                } else {
                    self.history_prev();
                }
                None
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                if self.line_count() > 1 {
                    self.move_down();
                } else {
                    self.history_next();
                }
                None
            }
            (KeyModifiers::NONE, KeyCode::Home) => {
                self.move_to_line_start();
                None
            }
            (KeyModifiers::NONE, KeyCode::End) => {
                self.move_to_line_end();
                None
            }
            (KeyModifiers::NONE, KeyCode::Tab) => Some(InputSignal::TabComplete),
            (_, KeyCode::Char(c)) if !modifiers.contains(KeyModifiers::CONTROL) => {
                self.insert_char(c);
                None
            }
            _ => None,
        }
    }

    // -- 编辑操作 --

    fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    fn insert_str(&mut self, s: &str) {
        self.buffer.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    fn delete_char_back(&mut self) {
        if self.cursor > 0 {
            let prev = self.prev_char_boundary();
            self.buffer.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = self.next_char_boundary();
            self.buffer.drain(self.cursor..next);
        }
    }

    fn delete_word_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut pos = self.cursor;
        while pos > 0 && self.char_before(pos) == Some(' ') {
            pos = self.prev_boundary_from(pos);
        }
        while pos > 0 && self.char_before(pos) != Some(' ') && self.char_before(pos) != Some('\n') {
            pos = self.prev_boundary_from(pos);
        }
        self.buffer.drain(pos..self.cursor);
        self.cursor = pos;
    }

    fn move_to_line_start(&mut self) {
        let before = &self.buffer[..self.cursor];
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        self.cursor = line_start;
    }

    fn move_to_line_end(&mut self) {
        let after = &self.buffer[self.cursor..];
        let line_end = after.find('\n').map(|i| self.cursor + i).unwrap_or(self.buffer.len());
        self.cursor = line_end;
    }

    fn kill_to_line_start(&mut self) {
        let before = &self.buffer[..self.cursor];
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        self.buffer.drain(line_start..self.cursor);
        self.cursor = line_start;
    }

    fn kill_to_line_end(&mut self) {
        let after = &self.buffer[self.cursor..];
        let line_end = after.find('\n').map(|i| self.cursor + i).unwrap_or(self.buffer.len());
        self.buffer.drain(self.cursor..line_end);
    }

    // -- 光标移动 --

    fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary();
        }
    }

    fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor = self.next_char_boundary();
        }
    }

    fn move_up(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line == 0 {
            return;
        }
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        let target_col = col.min(lines[line - 1].width());
        // 计算目标行的字节偏移
        let mut offset = 0;
        for i in 0..line - 1 {
            offset += lines[i].len() + 1; // +1 for '\n'
        }
        // 在目标行中按显示宽度定位
        let target_line = lines[line - 1];
        self.cursor = offset + byte_offset_for_width(target_line, target_col);
    }

    fn move_down(&mut self) {
        let (line, col) = self.cursor_line_col();
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        if line + 1 >= lines.len() {
            return;
        }
        let target_col = col.min(lines[line + 1].width());
        let mut offset = 0;
        for i in 0..=line {
            offset += lines[i].len() + 1;
        }
        let target_line = lines[line + 1];
        self.cursor = offset + byte_offset_for_width(target_line, target_col);
    }

    fn move_word_right(&mut self) {
        let len = self.buffer.len();
        while self.cursor < len && self.char_at(self.cursor) == Some(' ') {
            self.cursor = self.next_char_boundary();
        }
        while self.cursor < len && self.char_at(self.cursor) != Some(' ') && self.char_at(self.cursor) != Some('\n') {
            self.cursor = self.next_char_boundary();
        }
    }

    fn move_word_left(&mut self) {
        while self.cursor > 0 && self.char_before(self.cursor) == Some(' ') {
            self.cursor = self.prev_char_boundary();
        }
        while self.cursor > 0 && self.char_before(self.cursor) != Some(' ') && self.char_before(self.cursor) != Some('\n') {
            self.cursor = self.prev_char_boundary();
        }
    }

    // -- 辅助方法 --

    fn char_at(&self, pos: usize) -> Option<char> {
        self.buffer[pos..].chars().next()
    }

    fn char_before(&self, pos: usize) -> Option<char> {
        self.buffer[..pos].chars().next_back()
    }

    fn prev_char_boundary(&self) -> usize {
        self.prev_boundary_from(self.cursor)
    }

    fn prev_boundary_from(&self, pos: usize) -> usize {
        let mut p = pos.saturating_sub(1);
        while p > 0 && !self.buffer.is_char_boundary(p) {
            p -= 1;
        }
        p
    }

    fn next_char_boundary(&self) -> usize {
        let mut pos = self.cursor + 1;
        while pos < self.buffer.len() && !self.buffer.is_char_boundary(pos) {
            pos += 1;
        }
        pos
    }

    // -- 历史 --

    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
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
        if let Some(ref path) = self.history_path {
            let max = 1000;
            let start = self.history.len().saturating_sub(max);
            let content = self.history[start..].join("\n");
            let _ = std::fs::write(path, content);
        }
    }

    pub fn apply_completion(&mut self, value: &str) {
        let word_start = self.buffer[..self.cursor]
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        self.buffer.replace_range(word_start..self.cursor, value);
        self.cursor = word_start + value.len();
    }
}

/// 根据目标显示宽度计算字节偏移
fn byte_offset_for_width(s: &str, target_width: usize) -> usize {
    let mut w = 0;
    for (i, ch) in s.char_indices() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > target_width {
            return i;
        }
        w += cw;
    }
    s.len()
}
