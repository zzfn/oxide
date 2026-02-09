use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::time::Instant;

use super::style;

const MAX_OUTPUT_LINES: usize = 15;
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// 活跃的命令执行状态
pub struct ActiveExecCell {
    pub name: String,
    pub arguments: String,
    pub output_lines: Vec<String>,
    pub start_time: Instant,
    pub spinner_frame: usize,
}

impl ActiveExecCell {
    pub fn new(name: String, arguments: String) -> Self {
        Self {
            name,
            arguments,
            output_lines: Vec::new(),
            start_time: Instant::now(),
            spinner_frame: 0,
        }
    }

    pub fn push_output(&mut self, text: &str) {
        for line in text.lines() {
            self.output_lines.push(line.to_string());
        }
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    /// 渲染为 ratatui Lines（活跃状态，带 spinner）
    pub fn render_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("")];

        let spinner = SPINNER_FRAMES[self.spinner_frame];
        let elapsed = self.start_time.elapsed();

        // Header: spinner + tool name + args
        let args_display = if self.arguments.len() > 60 {
            format!("({}...)", &self.arguments[..60])
        } else {
            format!("({})", self.arguments)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                style::spinner_style(),
            ),
            Span::styled("⏺ ", style::tool_name_style()),
            Span::styled(
                self.name.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                args_display,
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!(" [{:.1}s]", elapsed.as_secs_f64()),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // Output (last N lines)
        let total = self.output_lines.len();
        let start = total.saturating_sub(MAX_OUTPUT_LINES);
        for line_text in &self.output_lines[start..] {
            let truncated = if line_text.len() > width as usize - 6 {
                format!("  ⎿ {}...", &line_text[..width as usize - 9])
            } else {
                format!("  ⎿ {}", line_text)
            };
            lines.push(Line::from(Span::styled(
                truncated,
                Style::default().fg(Color::DarkGray),
            )));
        }
        if start > 0 {
            lines.insert(2, Line::from(Span::styled(
                format!("  ⎿ ... ({} lines above)", start),
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }
}
