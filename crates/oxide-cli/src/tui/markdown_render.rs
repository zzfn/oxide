use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, CodeBlockKind};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// 将 markdown 文本渲染为 ratatui Lines
pub fn render_markdown(source: &str, width: Option<usize>) -> Vec<Line<'static>> {
    let mut renderer = MarkdownRenderer::new(width);
    renderer.render(source);
    renderer.into_lines()
}

/// 流式 Markdown 收集器
pub struct MarkdownStreamCollector {
    buffer: String,
    width: Option<usize>,
    committed_lines: Vec<Line<'static>>,
}

impl MarkdownStreamCollector {
    pub fn new(width: Option<usize>) -> Self {
        Self {
            buffer: String::new(),
            width,
            committed_lines: Vec::new(),
        }
    }

    /// 追加增量文本
    pub fn push(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    /// 提交当前已完成的行（以换行符为界），返回新增行数
    pub fn commit_lines(&mut self) -> usize {
        if self.buffer.is_empty() {
            return 0;
        }

        let last_nl = self.buffer.rfind('\n');
        let committed_source = match last_nl {
            Some(pos) => {
                let committed = self.buffer[..=pos].to_string();
                self.buffer = self.buffer[pos + 1..].to_string();
                committed
            }
            None => return 0,
        };

        let new_lines = render_markdown(&committed_source, self.width);
        let count = new_lines.len();
        self.committed_lines.extend(new_lines);
        count
    }

    /// 刷新所有剩余内容
    pub fn flush(&mut self) -> usize {
        if self.buffer.is_empty() {
            return 0;
        }
        let remaining = std::mem::take(&mut self.buffer);
        let new_lines = render_markdown(&remaining, self.width);
        let count = new_lines.len();
        self.committed_lines.extend(new_lines);
        count
    }

    /// 取出所有已提交的行
    pub fn take_lines(&mut self) -> Vec<Line<'static>> {
        std::mem::take(&mut self.committed_lines)
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.committed_lines.clear();
    }

    pub fn has_pending(&self) -> bool {
        !self.buffer.is_empty()
    }
}

struct MarkdownRenderer {
    lines: Vec<Line<'static>>,
    current_spans: Vec<Span<'static>>,
    style_stack: Vec<Style>,
    in_code_block: bool,
    code_block_lang: Option<String>,
    code_block_buffer: String,
    list_depth: usize,
    ordered_list_index: Option<u64>,
    _width: Option<usize>,
}

impl MarkdownRenderer {
    fn new(width: Option<usize>) -> Self {
        Self {
            lines: Vec::new(),
            current_spans: Vec::new(),
            style_stack: vec![Style::default()],
            in_code_block: false,
            code_block_lang: None,
            code_block_buffer: String::new(),
            list_depth: 0,
            ordered_list_index: None,
            _width: width,
        }
    }

    fn current_style(&self) -> Style {
        self.style_stack.last().copied().unwrap_or_default()
    }

    fn push_style(&mut self, style: Style) {
        let merged = self.current_style().patch(style);
        self.style_stack.push(merged);
    }

    fn pop_style(&mut self) {
        if self.style_stack.len() > 1 {
            self.style_stack.pop();
        }
    }

    fn flush_line(&mut self) {
        if !self.current_spans.is_empty() {
            let spans = std::mem::take(&mut self.current_spans);
            self.lines.push(Line::from(spans));
        }
    }

    fn push_empty_line(&mut self) {
        self.flush_line();
        self.lines.push(Line::from(""));
    }

    fn render(&mut self, source: &str) {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TABLES);

        let events: Vec<Event<'_>> = Parser::new_ext(source, opts).collect();

        for event in events {
            match event {
                Event::Start(tag) => self.handle_start(tag),
                Event::End(tag) => self.handle_end(tag),
                Event::Text(text) => self.handle_text(&text),
                Event::Code(code) => {
                    let style = Style::default().fg(Color::Yellow);
                    self.current_spans.push(Span::styled(
                        format!("`{}`", code),
                        style,
                    ));
                }
                Event::SoftBreak => {
                    self.current_spans.push(Span::raw(" "));
                }
                Event::HardBreak => {
                    self.flush_line();
                }
                Event::Rule => {
                    self.flush_line();
                    self.lines.push(Line::from(Span::styled(
                        "─".repeat(40),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                _ => {}
            }
        }
        self.flush_line();
    }

    fn handle_start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.flush_line();
                let color = match level {
                    pulldown_cmark::HeadingLevel::H1 => Color::Cyan,
                    pulldown_cmark::HeadingLevel::H2 => Color::Blue,
                    _ => Color::Magenta,
                };
                self.push_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
                let prefix = "#".repeat(level as usize);
                self.current_spans.push(Span::styled(
                    format!("{} ", prefix),
                    self.current_style(),
                ));
            }
            Tag::Paragraph => {
                self.flush_line();
            }
            Tag::CodeBlock(kind) => {
                self.flush_line();
                self.in_code_block = true;
                self.code_block_buffer.clear();
                self.code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let l = lang.to_string();
                        if l.is_empty() { None } else { Some(l) }
                    }
                    CodeBlockKind::Indented => None,
                };
                let header = if let Some(lang) = &self.code_block_lang {
                    format!("```{}", lang)
                } else {
                    "```".to_string()
                };
                self.lines.push(Line::from(Span::styled(
                    header,
                    Style::default().fg(Color::DarkGray),
                )));
            }
            Tag::Strong => {
                self.push_style(Style::default().add_modifier(Modifier::BOLD));
            }
            Tag::Emphasis => {
                self.push_style(Style::default().add_modifier(Modifier::ITALIC));
            }
            Tag::Strikethrough => {
                self.push_style(Style::default().add_modifier(Modifier::CROSSED_OUT));
            }
            Tag::List(start) => {
                self.flush_line();
                self.list_depth += 1;
                self.ordered_list_index = start;
            }
            Tag::Item => {
                self.flush_line();
                let indent = "  ".repeat(self.list_depth.saturating_sub(1));
                let bullet = if let Some(ref mut idx) = self.ordered_list_index {
                    let s = format!("{}{}. ", indent, idx);
                    *idx += 1;
                    s
                } else {
                    format!("{}• ", indent)
                };
                self.current_spans.push(Span::styled(
                    bullet,
                    Style::default().fg(Color::DarkGray),
                ));
            }
            Tag::BlockQuote(_) => {
                self.flush_line();
                self.push_style(Style::default().fg(Color::DarkGray));
                self.current_spans.push(Span::styled(
                    "│ ",
                    Style::default().fg(Color::DarkGray),
                ));
            }
            Tag::Link { dest_url, .. } => {
                self.push_style(Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED));
                // Store URL for end tag
                let _ = dest_url;
            }
            _ => {}
        }
    }

    fn handle_end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                self.pop_style();
                self.flush_line();
            }
            TagEnd::Paragraph => {
                self.flush_line();
                self.push_empty_line();
            }
            TagEnd::CodeBlock => {
                if self.in_code_block {
                    for line in self.code_block_buffer.clone().lines() {
                        self.lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(Color::Yellow),
                        )));
                    }
                    self.lines.push(Line::from(Span::styled(
                        "```",
                        Style::default().fg(Color::DarkGray),
                    )));
                    self.in_code_block = false;
                    self.code_block_buffer.clear();
                    self.code_block_lang = None;
                }
            }
            TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough => {
                self.pop_style();
            }
            TagEnd::List(_) => {
                self.list_depth = self.list_depth.saturating_sub(1);
                if self.list_depth == 0 {
                    self.ordered_list_index = None;
                    self.push_empty_line();
                }
            }
            TagEnd::Item => {
                self.flush_line();
            }
            TagEnd::BlockQuote(_) => {
                self.pop_style();
                self.flush_line();
            }
            TagEnd::Link => {
                self.pop_style();
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, text: &str) {
        if self.in_code_block {
            self.code_block_buffer.push_str(text);
        } else {
            self.current_spans.push(Span::styled(
                text.to_string(),
                self.current_style(),
            ));
        }
    }

    fn into_lines(mut self) -> Vec<Line<'static>> {
        self.flush_line();
        // Remove trailing empty lines
        while self.lines.last().is_some_and(|l| l.spans.is_empty() || (l.spans.len() == 1 && l.spans[0].content.is_empty())) {
            self.lines.pop();
        }
        self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let lines = render_markdown("Hello **world**", None);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_code_block() {
        let lines = render_markdown("```rust\nfn main() {}\n```", None);
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_stream_collector() {
        let mut collector = MarkdownStreamCollector::new(None);
        collector.push("Hello ");
        assert_eq!(collector.commit_lines(), 0);
        collector.push("world\n");
        assert!(collector.commit_lines() > 0);
        let lines = collector.take_lines();
        assert!(!lines.is_empty());
    }
}
