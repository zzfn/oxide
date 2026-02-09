use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, StatefulWidget, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget};

use super::approval_overlay::ApprovalOverlay;
use super::exec_cell::ActiveExecCell;
use super::history_cell::HistoryCell;
use super::markdown_render::MarkdownStreamCollector;

/// 聊天视图主 widget
pub struct ChatWidget {
    /// 已提交的历史 cells
    history: Vec<HistoryCell>,
    /// 当前活跃的流式文本收集器
    stream_collector: Option<MarkdownStreamCollector>,
    /// 已收集的流式行（待提交到 history）
    stream_lines: Vec<Line<'static>>,
    /// 活跃的工具调用
    active_exec: Option<ActiveExecCell>,
    /// 审批弹窗
    approval: Option<ApprovalOverlay>,
    /// 滚动偏移（从底部算起）
    scroll_offset: usize,
    /// 全部渲染行的缓存
    rendered_lines_cache: Vec<Line<'static>>,
    /// 缓存是否需要刷新
    cache_dirty: bool,
    /// 终端宽度（用于缓存失效）
    last_width: u16,
}

impl ChatWidget {
    pub fn new() -> Self {
        let mut widget = Self {
            history: Vec::new(),
            stream_collector: None,
            stream_lines: Vec::new(),
            active_exec: None,
            approval: None,
            scroll_offset: 0,
            rendered_lines_cache: Vec::new(),
            cache_dirty: true,
            last_width: 0,
        };
        widget.push_cell(HistoryCell::Welcome);
        widget
    }

    pub fn push_cell(&mut self, cell: HistoryCell) {
        self.history.push(cell);
        self.cache_dirty = true;
        self.scroll_offset = 0; // auto-scroll to bottom on new content
    }

    /// 开始流式助手消息
    pub fn begin_stream(&mut self, width: Option<usize>) {
        // Add assistant header
        self.push_cell(HistoryCell::Separator);
        self.stream_collector = Some(MarkdownStreamCollector::new(width));
        self.stream_lines.clear();
    }

    /// 追加流式文本
    pub fn push_stream_text(&mut self, text: &str) {
        if let Some(collector) = &mut self.stream_collector {
            collector.push(text);
            collector.commit_lines();
            let new_lines = collector.take_lines();
            self.stream_lines.extend(new_lines);
        }
        self.cache_dirty = true;
        self.scroll_offset = 0;
    }

    /// 结束流式消息，提交完整内容
    pub fn end_stream(&mut self, full_text: &str) {
        if let Some(mut collector) = self.stream_collector.take() {
            collector.flush();
            let _ = collector.take_lines();
        }
        self.stream_lines.clear();
        if !full_text.is_empty() {
            self.push_cell(HistoryCell::AssistantMessage {
                text: full_text.to_string(),
            });
        }
        self.cache_dirty = true;
    }

    /// 开始工具调用
    pub fn begin_tool_call(&mut self, name: String, arguments: String) {
        self.active_exec = Some(ActiveExecCell::new(name, arguments));
        self.cache_dirty = true;
    }

    /// 完成工具调用
    pub fn end_tool_call(&mut self, name: String, success: bool, _summary: Option<String>) {
        if let Some(exec) = self.active_exec.take() {
            let output_text = if exec.output_lines.is_empty() {
                None
            } else {
                Some(exec.output_lines.join("\n"))
            };
            self.push_cell(HistoryCell::ToolCall {
                name,
                arguments: exec.arguments,
                output: output_text,
                success: Some(success),
                elapsed: Some(exec.start_time.elapsed()),
            });
        }
        self.cache_dirty = true;
    }

    /// 推送审批请求
    pub fn push_approval(&mut self, overlay: ApprovalOverlay) {
        self.approval = Some(overlay);
    }

    /// 获取审批弹窗
    pub fn approval_mut(&mut self) -> Option<&mut ApprovalOverlay> {
        self.approval.as_mut()
    }

    /// 弹出审批弹窗
    pub fn take_approval(&mut self) -> Option<ApprovalOverlay> {
        self.approval.take()
    }

    pub fn has_approval(&self) -> bool {
        self.approval.is_some()
    }

    /// Tick spinner for active exec
    pub fn tick(&mut self) {
        if let Some(exec) = &mut self.active_exec {
            exec.tick_spinner();
            self.cache_dirty = true;
        }
    }

    /// 向上滚动
    pub fn scroll_up(&mut self, lines: usize) {
        let total = self.total_rendered_lines();
        self.scroll_offset = (self.scroll_offset + lines).min(total.saturating_sub(1));
    }

    /// 向下滚动
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    fn total_rendered_lines(&self) -> usize {
        self.rendered_lines_cache.len()
    }

    /// 重建渲染行缓存
    fn rebuild_cache(&mut self, width: u16) {
        if !self.cache_dirty && self.last_width == width {
            return;
        }
        self.last_width = width;

        let mut all_lines: Vec<Line<'static>> = Vec::new();

        // History cells
        for cell in &self.history {
            let cell_lines = cell.render_lines(width);
            all_lines.extend(cell_lines);
        }

        // Active stream lines (assistant header + collected lines)
        if self.stream_collector.is_some() || !self.stream_lines.is_empty() {
            // Emit assistant header if not already in history
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(Span::styled(
                "Assistant",
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            )));
            all_lines.push(Line::from(""));
            for line in &self.stream_lines {
                all_lines.push(line.clone());
            }
        }

        // Active exec cell
        if let Some(exec) = &self.active_exec {
            let exec_lines = exec.render_lines(width);
            all_lines.extend(exec_lines);
        }

        self.rendered_lines_cache = all_lines;
        self.cache_dirty = false;
    }

    /// 渲染聊天区域（不含 bottom pane）
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.rebuild_cache(area.width);

        let total = self.rendered_lines_cache.len();
        let visible_height = area.height as usize;

        // 计算显示范围（从底部算起）
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(visible_height);

        let visible_lines: Vec<Line<'static>> = self.rendered_lines_cache[start..end]
            .iter()
            .cloned()
            .collect();

        let paragraph = Paragraph::new(visible_lines);
        paragraph.render(area, buf);

        // 滚动条
        if total > visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(total)
                .position(start);
            StatefulWidget::render(scrollbar, area, buf, &mut scrollbar_state);
        }

        // 审批弹窗（覆盖渲染）
        if let Some(approval) = &self.approval {
            approval.render(area, buf);
        }
    }
}
