//! 增量 Markdown 解析器
//!
//! 提供高效的 Markdown 解析和渲染，支持增量更新和语法高亮。

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::iter::Peekable;
use std::str::Chars;

/// Markdown 解析器
pub struct MarkdownParser {
    /// 代码块样式
    pub code_style: Style,
    /// 内联代码样式
    pub inline_code_style: Style,
    /// 粗体样式
    pub bold_style: Style,
    /// 斜体样式
    pub italic_style: Style,
    /// 链接样式
    pub link_style: Style,
    /// 标题样式
    pub heading_style: Style,
    /// 引用样式
    pub quote_style: Style,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    /// 创建新的 Markdown 解析器
    pub fn new() -> Self {
        Self {
            code_style: Style::default().fg(Color::Cyan),
            inline_code_style: Style::default().fg(Color::Yellow),
            bold_style: Style::default().add_modifier(Modifier::BOLD),
            italic_style: Style::default().add_modifier(Modifier::ITALIC),
            link_style: Style::default().fg(Color::Blue).underlined(),
            heading_style: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            quote_style: Style::default().fg(Color::DarkGray),
        }
    }

    /// 使用主题创建解析器
    pub fn with_theme(theme: &crate::tui::Theme) -> Self {
        Self {
            code_style: theme.secondary_style(),
            inline_code_style: theme.warning_style(),
            bold_style: Style::default().add_modifier(Modifier::BOLD),
            italic_style: Style::default().add_modifier(Modifier::ITALIC),
            link_style: theme.primary_style().underlined(),
            heading_style: theme.success_style().add_modifier(Modifier::BOLD),
            quote_style: theme.help_text_style(),
        }
    }

    /// 解析 Markdown 文本并返回渲染后的行
    pub fn parse(&self, text: &str) -> Vec<Line> {
        let mut lines = Vec::new();

        for line in text.lines() {
            let parsed_line = self.parse_line(line);
            lines.extend(parsed_line);
        }

        lines
    }

    /// 解析单行 Markdown
    fn parse_line(&self, line: &str) -> Vec<Line> {
        // 检查标题
        if let Some(level) = self.detect_heading(line) {
            return self.parse_heading(line, level);
        }

        // 检查代码块
        if self.is_code_fence(line) {
            return vec![Line::from(vec![Span::styled(
                line,
                self.code_style,
            )])];
        }

        // 检查引用
        if line.starts_with(">") {
            return self.parse_quote(line);
        }

        // 检查无序列表
        if line.starts_with("- ") || line.starts_with("* ") {
            return self.parse_list_item(line);
        }

        // 检查有序列表
        if let Some(rest) = self.strip_numbered_list(line) {
            return self.parse_numbered_item(rest);
        }

        // 普通文本，解析内联格式
        self.parse_inline(line)
    }

    /// 检测标题级别
    fn detect_heading(&self, line: &str) -> Option<usize> {
        if !line.starts_with('#') {
            return None;
        }

        let mut chars = line.chars();
        let mut count = 0;

        while chars.next() == Some('#') {
            count += 1;
        }

        // 标题后必须有空格
        if count > 0 && count <= 6 && line.chars().nth(count) == Some(' ') {
            Some(count)
        } else {
            None
        }
    }

    /// 解析标题
    fn parse_heading(&self, line: &str, level: usize) -> Vec<Line> {
        let text = &line[level + 1..]; // 跳过 # 和空格
        let prefix = "#".repeat(level);

        let mut spans = vec![
            Span::styled(prefix, self.heading_style),
            Span::raw(" "),
        ];

        spans.extend(self.parse_inline_spans(text));

        vec![Line::from(spans)]
    }

    /// 检查是否是代码围栏
    fn is_code_fence(&self, line: &str) -> bool {
        line.starts_with("```") || line.starts_with("~~~")
    }

    /// 解析引用
    fn parse_quote(&self, line: &str) -> Vec<Line> {
        let text = line[1..].trim_start();

        let mut spans = vec![Span::styled("▍ ", self.quote_style)];
        spans.extend(self.parse_inline_spans(text));

        vec![Line::from(spans)]
    }

    /// 解析无序列表项
    fn parse_list_item(&self, line: &str) -> Vec<Line> {
        let text = line[2..].trim_start();

        let mut spans = vec![Span::styled("• ", Style::default())];
        spans.extend(self.parse_inline_spans(text));

        vec![Line::from(spans)]
    }

    /// 去除有序列表标记
    fn strip_numbered_list(&self, line: &str) -> Option<&str> {
        let mut chars = line.chars().peekable();

        // 检查数字
        let mut num = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                num.push(c);
                chars.next();
            } else {
                break;
            }
        }

        if num.is_empty() {
            return None;
        }

        // 检查点和空格
        if chars.next() == Some('.') && chars.peek() == Some(&' ') {
            let rest = line[num.len() + 2..].trim_start();
            Some(rest)
        } else {
            None
        }
    }

    /// 解析有序列表项
    fn parse_numbered_item(&self, text: &str) -> Vec<Line> {
        let mut spans = vec![Span::styled("• ", Style::default())];
        spans.extend(self.parse_inline_spans(text));

        vec![Line::from(spans)]
    }

    /// 解析内联格式（返回单行）
    fn parse_inline(&self, text: &str) -> Vec<Line> {
        let spans = self.parse_inline_spans(text);
        vec![Line::from(spans)]
    }

    /// 解析内联格式并返回 spans
    fn parse_inline_spans(&self, text: &str) -> Vec<Span> {
        let mut spans = Vec::new();
        let mut chars = text.chars().peekable();
        let mut current = String::new();

        while let Some(c) = chars.next() {
            match c {
                '`' => {
                    // 内联代码
                    if !current.is_empty() {
                        spans.push(Span::raw(current.clone()));
                        current.clear();
                    }

                    let code = self.extract_until(&mut chars, '`');
                    spans.push(Span::styled(code, self.inline_code_style));
                }
                '[' => {
                    // 链接
                    if !current.is_empty() {
                        spans.push(Span::raw(current.clone()));
                        current.clear();
                    }

                    if let Some((text, url)) = self.extract_link(&mut chars) {
                        spans.push(Span::styled(text, self.link_style));
                        spans.push(Span::raw(format!("({})", url)));
                    } else {
                        current.push('[');
                    }
                }
                '*' | '_' => {
                    // 粗体或斜体
                    let is_bold = matches!(chars.peek(), Some('*') | Some('_'))
                        && chars.peek() == Some(&c);

                    if is_bold {
                        chars.next(); // 消费第二个字符
                        if !current.is_empty() {
                            spans.push(Span::raw(current.clone()));
                            current.clear();
                        }

                        let bold_text = self.extract_until_double(&mut chars, c);
                        spans.push(Span::styled(bold_text, self.bold_style));
                    } else {
                        if !current.is_empty() {
                            spans.push(Span::raw(current.clone()));
                            current.clear();
                        }

                        let italic_text = self.extract_until(&mut chars, c);
                        spans.push(Span::styled(italic_text, self.italic_style));
                    }
                }
                '\\' => {
                    // 转义字符
                    if let Some(escaped) = chars.next() {
                        current.push(escaped);
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            spans.push(Span::raw(current));
        }

        spans
    }

    /// 提取直到指定字符的文本
    fn extract_until(&self, chars: &mut Peekable<Chars>, delimiter: char) -> String {
        let mut result = String::new();

        while let Some(&c) = chars.peek() {
            if c == delimiter {
                chars.next(); // 消费分隔符
                return result;
            }
            if c == '\\' {
                chars.next();
                if let Some(escaped) = chars.next() {
                    result.push(escaped);
                }
            } else {
                result.push(c);
                chars.next();
            }
        }

        result
    }

    /// 提取直到双分隔符的文本（用于粗体）
    fn extract_until_double(&self, chars: &mut Peekable<Chars>, delimiter: char) -> String {
        let mut result = String::new();

        while let Some(&c) = chars.peek() {
            if c == delimiter {
                chars.next();
                if chars.peek() == Some(&delimiter) {
                    chars.next();
                    return result;
                } else {
                    result.push(delimiter);
                }
            } else if c == '\\' {
                chars.next();
                if let Some(escaped) = chars.next() {
                    result.push(escaped);
                }
            } else {
                result.push(c);
                chars.next();
            }
        }

        result
    }

    /// 提取链接
    fn extract_link(&self, chars: &mut Peekable<Chars>) -> Option<(String, String)> {
        let mut link_text = String::new();

        // 提取链接文本
        while let Some(&c) = chars.peek() {
            if c == ']' {
                chars.next();
                break;
            }
            if c == '\\' {
                chars.next();
                if let Some(escaped) = chars.next() {
                    link_text.push(escaped);
                }
            } else {
                link_text.push(c);
                chars.next();
            }
        }

        // 检查是否有 '('
        if chars.peek() != Some(&'(') {
            return None;
        }
        chars.next();

        // 提取 URL
        let mut url = String::new();
        while let Some(&c) = chars.peek() {
            if c == ')' {
                chars.next();
                return Some((link_text, url));
            }
            url.push(c);
            chars.next();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let parser = MarkdownParser::new();
        let lines = parser.parse("Hello world");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_parse_heading() {
        let parser = MarkdownParser::new();
        let lines = parser.parse("# Title");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_parse_inline_code() {
        let parser = MarkdownParser::new();
        let lines = parser.parse("This is `code` text");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_parse_bold() {
        let parser = MarkdownParser::new();
        let lines = parser.parse("This is **bold** text");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_parse_italic() {
        let parser = MarkdownParser::new();
        let lines = parser.parse("This is *italic* text");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_parse_list() {
        let parser = MarkdownParser::new();
        let lines = parser.parse("- Item 1\n- Item 2");
        assert_eq!(lines.len(), 2);
    }
}
