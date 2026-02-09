//! 输入框 Widget
//!
//! 使用 ratatui 渲染带边框的多行输入框，类似 Claude Code 的输入体验。
//! 支持 `/` 命令菜单和 `@` 文件补全菜单。

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use super::completer::{Suggestion, SuggestionKind};
use super::input::LineEditor;
use crate::app::CliMode;

const BORDER_TL: &str = "╭";
const BORDER_TR: &str = "╮";
const BORDER_BL: &str = "╰";
const BORDER_BR: &str = "╯";
const BORDER_H: &str = "─";
const BORDER_V: &str = "│";

pub const MAX_VISIBLE_ITEMS: usize = 8;

pub struct InputBox<'a> {
    editor: &'a LineEditor,
    mode: CliMode,
    placeholder: &'a str,
    completions: &'a [Suggestion],
    selected_completion: Option<usize>,
    scroll_offset: usize,
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
            scroll_offset: 0,
            focused: true,
        }
    }

    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = text;
        self
    }

    pub fn completions(mut self, items: &'a [Suggestion], selected: Option<usize>, scroll_offset: usize) -> Self {
        self.completions = items;
        self.selected_completion = selected;
        self.scroll_offset = scroll_offset;
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

    fn menu_rows(&self) -> u16 {
        if self.completions.is_empty() {
            return 0;
        }
        let items = self.completions.len().min(MAX_VISIBLE_ITEMS) as u16;
        items + 2
    }

    pub fn required_height(&self, width: u16) -> u16 {
        let inner_width = width.saturating_sub(4) as usize;
        if inner_width == 0 {
            return 3;
        }
        let lines = self.editor.lines();
        let mut total_rows = 0u16;
        for line in &lines {
            let w = line.width();
            let rows = if w == 0 {
                1
            } else {
                ((w as u16).saturating_sub(1)) / inner_width as u16 + 1
            };
            total_rows += rows;
        }
        let content_rows = total_rows.max(1).min(10);
        content_rows + 2 + self.menu_rows()
    }
}

impl Widget for InputBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 3 {
            return;
        }

        let border_style = Style::default().fg(self.border_color());
        let (mode_char, mode_color) = self.mode_indicator();
        let menu_rows = self.menu_rows();

        // -- 顶部边框 --
        let top_y = area.y;
        let mode_label = format!(" {} ", mode_char);
        let mode_label_width = mode_label.width() as u16;
        buf.set_string(area.x, top_y, BORDER_TL, border_style);
        buf.set_string(area.x + 1, top_y, BORDER_H, border_style);
        buf.set_string(
            area.x + 2,
            top_y,
            &mode_label,
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        );
        let right_start = area.x + 2 + mode_label_width;
        for x in right_start..area.x + area.width - 1 {
            buf.set_string(x, top_y, BORDER_H, border_style);
        }
        buf.set_string(area.x + area.width - 1, top_y, BORDER_TR, border_style);

        // -- 底部边框 (above menu if present) --
        let bottom_y = area.y + area.height - 1 - menu_rows;
        buf.set_string(area.x, bottom_y, BORDER_BL, border_style);
        for x in area.x + 1..area.x + area.width - 1 {
            buf.set_string(x, bottom_y, BORDER_H, border_style);
        }
        let hint = " Enter ⏎ ";
        let hint_width = hint.width() as u16;
        if area.width > hint_width + 4 {
            let hint_x = area.x + area.width - 1 - hint_width - 1;
            buf.set_string(hint_x, bottom_y, hint, Style::default().fg(Color::DarkGray));
        }
        buf.set_string(area.x + area.width - 1, bottom_y, BORDER_BR, border_style);

        // -- 内容区域 --
        let content_x = area.x + 2;
        let content_width = (area.width.saturating_sub(4)) as usize;
        let content_start_y = area.y + 1;
        let content_height = (bottom_y - content_start_y) as usize;

        for y in content_start_y..bottom_y {
            buf.set_string(area.x, y, BORDER_V, border_style);
            buf.set_string(area.x + area.width - 1, y, BORDER_V, border_style);
        }

        let buffer_text = self.editor.buffer();

        if buffer_text.is_empty() {
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
            if self.focused {
                buf.set_string(
                    content_x,
                    content_start_y,
                    "▎",
                    Style::default().fg(self.border_color()),
                );
            }
        } else {
            let lines = self.editor.lines();
            let (cursor_line, cursor_col) = self.editor.cursor_line_col();

            let mut row = 0usize;
            for (line_idx, line_text) in lines.iter().enumerate() {
                if row >= content_height {
                    break;
                }
                let y = content_start_y + row as u16;

                let display = if line_text.width() > content_width {
                    truncate_to_width(line_text, content_width)
                } else {
                    line_text.to_string()
                };
                buf.set_string(content_x, y, &display, Style::default());

                if self.focused && line_idx == cursor_line {
                    let cursor_x = content_x + cursor_col as u16;
                    if cursor_x < area.x + area.width - 1 {
                        let cursor_char = if cursor_col < line_text.width() {
                            line_text
                                .chars()
                                .nth(char_index_for_width(line_text, cursor_col))
                                .unwrap_or(' ')
                        } else {
                            ' '
                        };
                        buf.set_string(
                            cursor_x,
                            y,
                            cursor_char.to_string(),
                            Style::default().fg(Color::Black).bg(self.border_color()),
                        );
                    }
                }

                row += 1;
            }
        }

        // -- 补全菜单 --
        if !self.completions.is_empty() {
            self.render_dropdown_menu(area, buf, bottom_y + 1);
        }
    }
}

impl InputBox<'_> {
    fn render_dropdown_menu(&self, area: Rect, buf: &mut Buffer, start_y: u16) {
        let menu_style = Style::default().fg(Color::DarkGray);
        let total = self.completions.len();
        let max_visible = total.min(MAX_VISIBLE_ITEMS);
        let offset = self.scroll_offset;
        let has_scroll_up = offset > 0;
        let has_scroll_down = offset + max_visible < total;

        // Menu top border
        buf.set_string(area.x, start_y, BORDER_TL, menu_style);
        for x in area.x + 1..area.x + area.width - 1 {
            buf.set_string(x, start_y, BORDER_H, menu_style);
        }
        if has_scroll_up {
            let indicator = " ▲ ";
            let iw = indicator.width() as u16;
            if area.width > iw + 4 {
                buf.set_string(
                    area.x + area.width - 1 - iw - 1,
                    start_y,
                    indicator,
                    Style::default().fg(Color::Yellow),
                );
            }
        }
        buf.set_string(area.x + area.width - 1, start_y, BORDER_TR, menu_style);

        // Menu items
        let inner_width = area.width.saturating_sub(4) as usize;
        for (vi, item) in self.completions.iter().skip(offset).take(max_visible).enumerate() {
            let abs_idx = offset + vi;
            let y = start_y + 1 + vi as u16;
            buf.set_string(area.x, y, BORDER_V, menu_style);
            buf.set_string(area.x + area.width - 1, y, BORDER_V, menu_style);

            let is_selected = Some(abs_idx) == self.selected_completion;

            let icon = match item.kind {
                SuggestionKind::Command => "/ ",
                SuggestionKind::File => "📄 ",
                SuggestionKind::Directory => "📁 ",
                SuggestionKind::Tag => "# ",
            };
            let label = &item.value;
            let desc = item.description.as_deref().unwrap_or("");

            let name_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            };
            let desc_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let icon_w = icon.width();
            let name_w = label.width();
            let sep = "  ";
            let sep_w = sep.width();
            let desc_max = inner_width.saturating_sub(icon_w + name_w + sep_w);
            let desc_display = if desc.width() > desc_max {
                truncate_to_width(desc, desc_max)
            } else {
                desc.to_string()
            };

            if is_selected {
                let bg_str: String = " ".repeat(inner_width);
                buf.set_string(area.x + 2, y, &bg_str, desc_style);
            }

            buf.set_string(area.x + 2, y, icon, name_style);
            buf.set_string(area.x + 2 + icon_w as u16, y, label, name_style);
            buf.set_string(area.x + 2 + icon_w as u16 + name_w as u16, y, sep, desc_style);
            buf.set_string(
                area.x + 2 + icon_w as u16 + name_w as u16 + sep_w as u16,
                y,
                &desc_display,
                desc_style,
            );
        }

        // Menu bottom border
        let menu_bottom = start_y + 1 + max_visible as u16;
        buf.set_string(area.x, menu_bottom, BORDER_BL, menu_style);
        for x in area.x + 1..area.x + area.width - 1 {
            buf.set_string(x, menu_bottom, BORDER_H, menu_style);
        }
        if has_scroll_down {
            let indicator = format!(" ▼ +{} ", total - offset - max_visible);
            let iw = indicator.width() as u16;
            if area.width > iw + 4 {
                buf.set_string(
                    area.x + area.width - 1 - iw - 1,
                    menu_bottom,
                    &indicator,
                    Style::default().fg(Color::Yellow),
                );
            }
        }
        buf.set_string(area.x + area.width - 1, menu_bottom, BORDER_BR, menu_style);
    }
}

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
