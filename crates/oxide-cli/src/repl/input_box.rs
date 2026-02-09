//! 输入框 Widget
//!
//! 使用 ratatui 渲染带边框的多行输入框，类似 Claude Code 的输入体验。

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use super::completer::Suggestion;
use super::input::LineEditor;
use crate::app::CliMode;

/// 输入框边框字符
const BORDER_TL: &str = "╭";
const BORDER_TR: &str = "╮";
const BORDER_BL: &str = "╰";
const BORDER_BR: &str = "╯";
const BORDER_H: &str = "─";
const BORDER_V: &str = "│";

/// 输入框 Widget
pub struct InputBox<'a> {
    editor: &'a LineEditor,
    mode: CliMode,
    placeholder: &'a str,
    completions: &'a [Suggestion],
    selected_completion: Option<usize>,
    focused: bool,
}

impl<'a> InputBox<'a> {
    pub fn new(editor: &'a LineEditor, mode: CliMode) -> Self {
        Self {
            editor,
            mode,
            placeholder: "输入消息... (Enter 发送, Shift+Enter 换行)",
            completions: &[],
            selected_completion: None,
            focused: true,
        }
    }

    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = text;
        self
    }

    pub fn completions(mut self, items: &'a [Suggestion], selected: Option<usize>) -> Self {
        self.completions = items;
        self.selected_completion = selected;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    fn border_color(&self) -> Color {
        if !self.focused {
            return Color::DarkGray;
        }
        match self.mode {
            CliMode::Normal => Color::Cyan,
            CliMode::Fast => Color::Yellow,
            CliMode::Plan => Color::Magenta,
        }
    }

    fn mode_indicator(&self) -> (&str, Color) {
        match self.mode {
            CliMode::Normal => ("N", Color::Green),
            CliMode::Fast => ("F", Color::Yellow),
            CliMode::Plan => ("P", Color::Cyan),
        }
    }

    /// 计算输入框需要的行数（包含边框）
    pub fn required_height(&self, width: u16) -> u16 {
        let inner_width = width.saturating_sub(4) as usize; // 2 border + 2 padding
        if inner_width == 0 {
            return 3;
        }
        let lines = self.editor.lines();
        let mut total_rows = 0u16;
        for line in &lines {
            let w = line.width();
            let rows = if w == 0 { 1 } else { ((w as u16).saturating_sub(1)) / inner_width as u16 + 1 };
            total_rows += rows;
        }
        // 边框 2 行 + 内容行（最少 1 行，最多 10 行）
        let content_rows = total_rows.max(1).min(10);
        let completion_rows = if self.completions.is_empty() { 0 } else { 1 };
        content_rows + 2 + completion_rows
    }
}

impl Widget for InputBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 3 {
            return;
        }

        let border_style = Style::default().fg(self.border_color());
        let (mode_char, mode_color) = self.mode_indicator();

        // -- 顶部边框 --
        let top_y = area.y;
        // "╭─ [N] ─────...──╮"
        let mode_label = format!(" {} ", mode_char);
        let mode_label_width = mode_label.width() as u16;
        // 左角
        buf.set_string(area.x, top_y, BORDER_TL, border_style);
        buf.set_string(area.x + 1, top_y, BORDER_H, border_style);
        // 模式标签
        buf.set_string(
            area.x + 2,
            top_y,
            &mode_label,
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        );
        // 右侧填充
        let right_start = area.x + 2 + mode_label_width;
        for x in right_start..area.x + area.width - 1 {
            buf.set_string(x, top_y, BORDER_H, border_style);
        }
        buf.set_string(area.x + area.width - 1, top_y, BORDER_TR, border_style);

        // -- 底部边框 --
        let bottom_y = if self.completions.is_empty() {
            area.y + area.height - 1
        } else {
            area.y + area.height - 2
        };
        buf.set_string(area.x, bottom_y, BORDER_BL, border_style);
        for x in area.x + 1..area.x + area.width - 1 {
            buf.set_string(x, bottom_y, BORDER_H, border_style);
        }
        // 底部右侧提示
        let hint = " Enter ⏎ ";
        let hint_width = hint.width() as u16;
        if area.width > hint_width + 4 {
            let hint_x = area.x + area.width - 1 - hint_width - 1;
            buf.set_string(hint_x, bottom_y, hint, Style::default().fg(Color::DarkGray));
        }
        buf.set_string(area.x + area.width - 1, bottom_y, BORDER_BR, border_style);

        // -- 内容区域 --
        let content_x = area.x + 2; // 边框 + 1 padding
        let content_width = (area.width.saturating_sub(4)) as usize;
        let content_start_y = area.y + 1;
        let content_height = (bottom_y - content_start_y) as usize;

        // 左右边框
        for y in content_start_y..bottom_y {
            buf.set_string(area.x, y, BORDER_V, border_style);
            buf.set_string(area.x + area.width - 1, y, BORDER_V, border_style);
        }

        let buffer_text = self.editor.buffer();

        if buffer_text.is_empty() {
            // 显示 placeholder
            let ph = if self.placeholder.width() > content_width {
                &self.placeholder[..content_width]
            } else {
                self.placeholder
            };
            buf.set_string(
                content_x,
                content_start_y,
                ph,
                Style::default().fg(Color::DarkGray),
            );
            // 光标在起始位置
            if self.focused {
                buf.set_string(
                    content_x,
                    content_start_y,
                    "▎",
                    Style::default().fg(self.border_color()),
                );
            }
            return;
        }

        // 渲染文本内容和光标
        let lines = self.editor.lines();
        let (cursor_line, cursor_col) = self.editor.cursor_line_col();

        let mut row = 0usize;
        for (line_idx, line_text) in lines.iter().enumerate() {
            if row >= content_height {
                break;
            }
            let y = content_start_y + row as u16;

            // 渲染行文本（截断到 content_width）
            let display = if line_text.width() > content_width {
                truncate_to_width(line_text, content_width)
            } else {
                line_text.to_string()
            };
            buf.set_string(content_x, y, &display, Style::default());

            // 渲染光标
            if self.focused && line_idx == cursor_line {
                let cursor_x = content_x + cursor_col as u16;
                if (cursor_x) < area.x + area.width - 1 {
                    // 获取光标位置的字符
                    let cursor_char = if cursor_col < line_text.width() {
                        line_text.chars().nth(
                            char_index_for_width(line_text, cursor_col)
                        ).unwrap_or(' ')
                    } else {
                        ' '
                    };
                    buf.set_string(
                        cursor_x,
                        y,
                        cursor_char.to_string(),
                        Style::default()
                            .fg(Color::Black)
                            .bg(self.border_color()),
                    );
                }
            }

            row += 1;
        }

        // -- 补全菜单 --
        if !self.completions.is_empty() {
            let comp_y = area.y + area.height - 1;
            let mut cx = area.x + 1;
            for (i, item) in self.completions.iter().take(6).enumerate() {
                let label = &item.value;
                let style = if Some(i) == self.selected_completion {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                };
                let padded = format!(" {} ", label);
                let w = padded.width() as u16;
                if cx + w < area.x + area.width {
                    buf.set_string(cx, comp_y, &padded, style);
                    cx += w + 1;
                }
            }
            if self.completions.len() > 6 {
                let more = format!(" +{} ", self.completions.len() - 6);
                buf.set_string(cx, comp_y, &more, Style::default().fg(Color::DarkGray));
            }
        }
    }
}

/// 按显示宽度截断字符串
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut w = 0;
    let mut end = 0;
    for (i, ch) in s.char_indices() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > max_width {
            break;
        }
        w += cw;
        end = i + ch.len_utf8();
    }
    s[..end].to_string()
}

/// 根据显示宽度找到字符索引
fn char_index_for_width(s: &str, target_width: usize) -> usize {
    let mut w = 0;
    for (idx, ch) in s.chars().enumerate() {
        if w >= target_width {
            return idx;
        }
        w += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
    }
    s.chars().count()
}
