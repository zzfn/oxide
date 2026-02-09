//! 自定义提示符
//!
//! 根据当前模式显示不同的提示符样式。

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::CliMode;

/// Oxide 自定义提示符
pub struct OxidePrompt {
    mode: CliMode,
}

impl OxidePrompt {
    pub fn new(mode: CliMode) -> Self {
        Self { mode }
    }

    pub fn set_mode(&mut self, mode: CliMode) {
        self.mode = mode;
    }

    fn mode_color(&self) -> Color {
        match self.mode {
            CliMode::Normal => Color::Green,
            CliMode::Fast => Color::Yellow,
            CliMode::Plan => Color::Cyan,
        }
    }

    /// 构建提示符 Spans（用于内联渲染）
    pub fn spans(&self) -> Vec<Span<'_>> {
        let color = self.mode_color();
        let mode_char = self.mode.short_name();
        vec![
            Span::styled(format!("[{}]", mode_char), Style::default().fg(color)),
            Span::raw(" "),
            Span::styled("> ", Style::default().fg(Color::Green)),
        ]
    }

    /// 构建为 ratatui Paragraph widget
    pub fn widget(&self) -> Paragraph<'_> {
        Paragraph::new(Line::from(self.spans()))
    }

    /// 提示符的显示宽度（字符数）
    pub fn display_width(&self) -> u16 {
        // "[N] > " = 6
        6
    }
}

impl Default for OxidePrompt {
    fn default() -> Self {
        Self::new(CliMode::Normal)
    }
}
