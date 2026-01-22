//! TUI 渲染缓存和性能优化
//!
//! 提供渲染缓存和虚拟滚动支持，提高大消息列表的渲染性能。

use ratatui::text::{Line, Span};
use std::collections::HashMap;

/// 渲染缓存
pub struct RenderCache {
    /// 缓存的渲染行
    lines: HashMap<String, Vec<Line>>,
    /// 最大缓存条目数
    max_entries: usize,
    /// 当前缓存条目数
    current_entries: usize,
}

impl RenderCache {
    /// 创建新的渲染缓存
    pub fn new(max_entries: usize) -> Self {
        Self {
            lines: HashMap::new(),
            max_entries,
            current_entries: 0,
        }
    }

    /// 获取缓存的渲染行
    pub fn get(&self, key: &str) -> Option<&Vec<Line>> {
        self.lines.get(key)
    }

    /// 插入渲染行到缓存
    pub fn insert(&mut self, key: String, lines: Vec<Line>) {
        // 如果缓存已满，移除最旧的条目
        if self.current_entries >= self.max_entries {
            if let Some(first_key) = self.lines.keys().next().cloned() {
                self.lines.remove(&first_key);
                self.current_entries -= 1;
            }
        }

        self.lines.insert(key, lines);
        self.current_entries += 1;
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current_entries = 0;
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        self.current_entries
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.current_entries == 0
    }
}

impl Default for RenderCache {
    fn default() -> Self {
        Self::new(100) // 默认缓存 100 条消息
    }
}

/// 虚拟滚动计算器
pub struct VirtualScroll {
    /// 每行高度（字符）
    pub line_height: usize,
    /// 可见区域高度（行数）
    pub visible_height: usize,
    /// 总内容高度（行数）
    pub total_height: usize,
    /// 当前滚动偏移（行）
    pub scroll_offset: usize,
}

impl VirtualScroll {
    /// 创建新的虚拟滚动
    pub fn new(line_height: usize, visible_height: usize) -> Self {
        Self {
            line_height,
            visible_height,
            total_height: 0,
            scroll_offset: 0,
        }
    }

    /// 设置总内容高度
    pub fn set_total_height(&mut self, total: usize) {
        self.total_height = total;
        // 确保滚动偏移不超过范围
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
    }

    /// 获取最大滚动偏移
    pub fn max_scroll_offset(&self) -> usize {
        self.total_height.saturating_sub(self.visible_height)
    }

    /// 检查是否可以滚动
    pub fn can_scroll_up(&self) -> bool {
        self.scroll_offset > 0
    }

    /// 检查是否可以向下滚动
    pub fn can_scroll_down(&self) -> bool {
        self.scroll_offset < self.max_scroll_offset()
    }

    /// 向上滚动
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// 向下滚动
    pub fn scroll_down(&mut self, amount: usize) {
        let new_offset = self.scroll_offset.saturating_add(amount);
        self.scroll_offset = new_offset.min(self.max_scroll_offset());
    }

    /// 滚动到顶部
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// 滚动到底部
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    /// 获取可见行的起始和结束索引
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.visible_height).min(self.total_height);
        (start, end)
    }

    /// 检查索引是否在可见范围内
    pub fn is_visible(&self, index: usize) -> bool {
        let (start, end) = self.visible_range();
        index >= start && index < end
    }

    /// 计算给定索引的位置
    pub fn position_for_index(&self, index: usize) -> Option<usize> {
        if index < self.total_height {
            Some(index.saturating_sub(self.scroll_offset))
        } else {
            None
        }
    }
}

/// 消息行计算器
pub struct MessageLineCalculator {
    /// 终端宽度（字符）
    pub terminal_width: usize,
    /// 最大消息行数
    pub max_lines: usize,
}

impl MessageLineCalculator {
    /// 创建新的消息行计算器
    pub fn new(terminal_width: usize, max_lines: usize) -> Self {
        Self {
            terminal_width,
            max_lines,
        }
    }

    /// 计算消息的行数
    pub fn count_lines(&self, text: &str) -> usize {
        let mut line_count = 0;

        for line in text.lines() {
            // 计算每行的字符数（考虑 Unicode）
            let line_width = line.chars().count();

            // 计算换行
            let lines_for_text = if line_width == 0 {
                1 // 空行
            } else if line_width <= self.terminal_width {
                1 // 一行能放下
            } else {
                // 需要多行
                (line_width + self.terminal_width - 1) / self.terminal_width
            };

            line_count += lines_for_text;

            // 限制最大行数
            if line_count >= self.max_lines {
                return self.max_lines;
            }
        }

        line_count.min(self.max_lines)
    }

    /// 截断消息到指定行数
    pub fn truncate_to_lines(&self, text: &str, max_lines: usize) -> String {
        let mut result = Vec::new();
        let mut line_count = 0;

        for line in text.lines() {
            let line_width = line.chars().count();
            let lines_for_text = if line_width == 0 {
                1
            } else if line_width <= self.terminal_width {
                1
            } else {
                (line_width + self.terminal_width - 1) / self.terminal_width
            };

            if line_count + lines_for_text > max_lines {
                break;
            }

            result.push(line.to_string());
            line_count += lines_for_text;
        }

        result.join("\n")
    }

    /// 包装文本以适应终端宽度
    pub fn wrap_text(&self, text: &str) -> Vec<String> {
        let mut wrapped = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for c in text.chars() {
            let char_width = if c == '\t' { 4 } else { 1 };

            if c == '\n' {
                wrapped.push(current_line.clone());
                current_line = String::new();
                current_width = 0;
            } else if current_width + char_width > self.terminal_width {
                wrapped.push(current_line);
                current_line = c.to_string();
                current_width = char_width;
            } else {
                current_line.push(c);
                current_width += char_width;
            }
        }

        if !current_line.is_empty() {
            wrapped.push(current_line);
        }

        wrapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_cache() {
        let mut cache = RenderCache::new(2);

        cache.insert("key1".to_string(), vec![Line::from("test1")]);
        cache.insert("key2".to_string(), vec![Line::from("test2")]);

        assert_eq!(cache.len(), 2);
        assert!(cache.get("key1").is_some());

        // 测试缓存淘汰
        cache.insert("key3".to_string(), vec![Line::from("test3")]);
        assert_eq!(cache.len(), 2);
        assert!(cache.get("key1").is_none()); // key1 应该被淘汰
    }

    #[test]
    fn test_virtual_scroll() {
        let mut scroll = VirtualScroll::new(1, 10);
        scroll.set_total_height(100);

        assert_eq!(scroll.max_scroll_offset(), 90);
        assert_eq!(scroll.visible_range(), (0, 10));

        scroll.scroll_down(5);
        assert_eq!(scroll.scroll_offset, 5);

        scroll.scroll_to_bottom();
        assert_eq!(scroll.scroll_offset, 90);

        scroll.scroll_to_top();
        assert_eq!(scroll.scroll_offset, 0);
    }

    #[test]
    fn test_message_line_calculator() {
        let calc = MessageLineCalculator::new(80, 1000);

        // 短文本
        assert_eq!(calc.count_lines("Hello"), 1);

        // 空文本
        assert_eq!(calc.count_lines(""), 0);

        // 多行文本
        assert_eq!(calc.count_lines("Line 1\nLine 2\nLine 3"), 3);
    }

    #[test]
    fn test_wrap_text() {
        let calc = MessageLineCalculator::new(10, 1000);

        let wrapped = calc.wrap_text("This is a long text that should be wrapped");
        assert!(wrapped.len() > 1);
    }
}
