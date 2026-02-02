//! 流式渲染状态管理
//!
//! 管理 AI 响应流式输出的状态转换和显示逻辑

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

use super::StatusLine;
use crate::utils;

/// 流式渲染状态
pub struct StreamState {
    /// MultiProgress 管理器
    mp: Option<Arc<MultiProgress>>,
    /// 状态行（工具执行期间需要暂停）
    statusline: Option<StatusLine>,
    /// 行缓冲（用于按行输出文本）
    line_buffer: String,
    /// 是否在思考模式
    is_thinking: bool,
    /// 当前工具执行的进度条
    current_tool_bar: Option<ProgressBar>,
    /// 当前工具信息（用于完成时输出）
    current_tool_info: Option<String>,
    /// Token 计数
    token_count: usize,
    /// 文本累积（用于计算 token）
    accumulated_text: String,
}

impl StreamState {
    /// 创建新的流式渲染状态
    pub fn new(mp: Option<Arc<MultiProgress>>, statusline: Option<StatusLine>) -> Self {
        Self {
            mp,
            statusline,
            line_buffer: String::new(),
            is_thinking: false,
            current_tool_bar: None,
            current_tool_info: None,
            token_count: 0,
            accumulated_text: String::new(),
        }
    }

    /// 输出一行文本
    pub fn println(&self, text: &str) {
        if let Some(mp) = &self.mp {
            let _ = mp.println(text);
        } else {
            println!("{}", text);
        }
    }

    /// 刷新行缓冲
    pub fn flush_buffer(&mut self) {
        if !self.line_buffer.is_empty() {
            self.println(&self.line_buffer);
            self.line_buffer.clear();
        }
    }

    /// 处理流式文本
    pub fn handle_text(&mut self, text: &str) {
        // 退出思考模式
        if self.is_thinking {
            self.flush_buffer();
            self.println("");
            self.is_thinking = false;
        }

        // 累积文本并计算 token
        self.accumulated_text.push_str(text);
        self.token_count = utils::count_tokens(&self.accumulated_text);

        // 更新状态行的 token 显示
        if let Some(ref mut sl) = self.statusline {
            sl.update("Processing", self.token_count);
        }

        // 按行缓冲输出
        for ch in text.chars() {
            if ch == '\n' {
                self.println(&self.line_buffer);
                self.line_buffer.clear();
            } else {
                self.line_buffer.push(ch);
            }
        }
    }

    /// 处理思考内容
    pub fn handle_reasoning(&mut self, reasoning: &str) {
        // 进入思考模式
        if !self.is_thinking {
            self.flush_buffer();
            self.println("\n💭 思考中:");
            self.is_thinking = true;
        }
        self.line_buffer.push_str(reasoning);
    }

    /// 开始工具调用
    pub fn start_tool(&mut self, tool_name: &str, description: String) {
        // 退出思考模式
        if self.is_thinking {
            self.flush_buffer();
            self.println("");
            self.is_thinking = false;
        }

        // 刷新文本缓冲
        self.flush_buffer();

        // 暂停 statusline
        if let Some(ref sl) = self.statusline {
            sl.suspend();
        }

        // 创建工具信息
        let tool_info = format!("{}({})", tool_name, description);

        // 创建进度条
        let bar = if let Some(mp) = &self.mp {
            let b = mp.add(ProgressBar::new_spinner());
            b.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner:.dim} {msg}")
                    .unwrap(),
            );
            b.set_message(format!("{} {}", "⏺".bright_black(), tool_info.clone()));
            b.enable_steady_tick(Duration::from_millis(80));
            b
        } else {
            let b = ProgressBar::new_spinner();
            b.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner:.dim} {msg}")
                    .unwrap(),
            );
            b.set_message(format!("{} {}", "⏺".bright_black(), tool_info.clone()));
            b.enable_steady_tick(Duration::from_millis(80));
            b
        };

        self.current_tool_bar = Some(bar);
        self.current_tool_info = Some(tool_info);
    }

    /// 完成工具调用
    pub fn finish_tool(&mut self) {
        // 清除进度条
        if let Some(bar) = self.current_tool_bar.take() {
            bar.finish_and_clear();
        }

        // 输出永久性完成信息
        if let Some(info) = self.current_tool_info.take() {
            let finished_msg = format!("{} {}", "⏺".green(), info);
            self.println(&finished_msg);
        }

        // 恢复 statusline
        if let Some(ref sl) = self.statusline {
            sl.resume();
        }
    }

    /// 完成流式输出
    pub fn finish(&mut self) {
        // 刷新剩余缓冲
        self.flush_buffer();
        // 输出换行
        self.println("");
    }
}
