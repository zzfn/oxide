//! 渲染模块
//!
//! 提供 Markdown 渲染、进度指示器和流式输出功能。

pub mod markdown;
pub mod spinner;
pub mod statusline;
pub mod stream;
pub mod tool_status;

pub use markdown::MarkdownRenderer;
pub use spinner::Spinner;
pub use statusline::StatusLine;
pub use stream::{StreamChannel, StreamRenderer};
pub use tool_status::{ToolStatus, ToolStatusDisplay};

use colored::Colorize;
use indicatif::MultiProgress;
use std::io::{self, Write};
use std::sync::Arc;

/// 统一渲染接口
pub struct Renderer {
    /// MultiProgress 管理器（所有输出通过它管理）
    mp: Arc<MultiProgress>,
    markdown: MarkdownRenderer,
    stream: StreamRenderer,
    tool_status: ToolStatusDisplay,
    statusline: StatusLine,
}

impl Renderer {
    /// 创建新的渲染器
    pub fn new() -> Self {
        let mp = Arc::new(MultiProgress::new());
        Self {
            markdown: MarkdownRenderer::new(),
            stream: StreamRenderer::new(),
            tool_status: ToolStatusDisplay::new(Arc::clone(&mp)),
            statusline: StatusLine::new(Arc::clone(&mp)),
            mp,
        }
    }

    /// 获取 MultiProgress 管理器
    pub fn multi_progress(&self) -> &Arc<MultiProgress> {
        &self.mp
    }

    /// 输出一行文本（通过 MultiProgress，确保不破坏进度条）
    pub fn println<S: AsRef<str>>(&self, msg: S) -> io::Result<()> {
        self.mp.println(msg)?;
        Ok(())
    }

    /// 获取流式渲染器的可变引用
    pub fn stream_mut(&mut self) -> &mut StreamRenderer {
        &mut self.stream
    }

    /// 获取工具状态显示器的可变引用
    pub fn tool_status_mut(&mut self) -> &mut ToolStatusDisplay {
        &mut self.tool_status
    }

    /// 获取状态行的可变引用
    pub fn statusline_mut(&mut self) -> &mut StatusLine {
        &mut self.statusline
    }

    /// 显示欢迎信息
    pub fn welcome(&self) {
        let _ = self.mp.println("");
        let _ = self.mp.println(format!(
            "{}",
            "╭─────────────────────────────────────╮".bright_cyan()
        ));
        let _ = self.mp.println(format!(
            "{}",
            "│         Oxide - AI 编程助手         │".bright_cyan()
        ));
        let _ = self.mp.println(format!(
            "{}",
            "╰─────────────────────────────────────╯".bright_cyan()
        ));
        let _ = self.mp.println("");
        let _ = self.mp.println(format!("  {} 输入问题开始对话", "•".green()));
        let _ = self.mp.println(format!(
            "  {} 输入 {} 查看帮助",
            "•".green(),
            "/help".yellow()
        ));
        let _ = self.mp.println(format!(
            "  {} 按 {} 两次退出",
            "•".green(),
            "Ctrl+C".yellow()
        ));
        let _ = self.mp.println("");
    }

    /// 显示分隔线
    pub fn separator(&self) {
        let _ = self.mp.println(format!("{}", "─".repeat(40).bright_black()));
    }

    /// 显示助手响应
    pub fn assistant_response(&self, content: &str) {
        let _ = self.mp.println("");
        let _ = self.mp.println(format!("{}", "Assistant".bright_blue().bold()));
        self.markdown.print(content);
        let _ = self.mp.println("");
    }

    /// 显示助手响应头部（用于流式输出）
    pub fn assistant_header(&self) {
        let _ = self.mp.println("");
        let _ = self
            .mp
            .println(format!("{} ", "Assistant".bright_blue().bold()));
    }

    /// 显示用户输入提示
    pub fn user_prompt(&self) {
        print!("{} ", ">".bright_green().bold());
        let _ = std::io::stdout().flush();
    }

    /// 显示错误信息
    pub fn error(&self, message: &str) {
        let _ = self
            .mp
            .println(format!("{} {}", "Error:".red().bold(), message));
    }

    /// 显示警告信息
    pub fn warning(&self, message: &str) {
        let _ = self
            .mp
            .println(format!("{} {}", "Warning:".yellow().bold(), message));
    }

    /// 显示信息
    pub fn info(&self, message: &str) {
        let _ = self
            .mp
            .println(format!("{} {}", "Info:".blue().bold(), message));
    }

    /// 显示成功信息
    pub fn success(&self, message: &str) {
        let _ = self
            .mp
            .println(format!("{} {}", "✓".green().bold(), message));
    }

    /// 渲染 Markdown 文本
    pub fn markdown(&self, content: &str) {
        self.markdown.print(content);
    }

    /// 获取 Markdown 渲染器的引用
    pub fn markdown_renderer(&self) -> &MarkdownRenderer {
        &self.markdown
    }

    /// 显示工具执行开始
    pub fn tool_execution(&self, tool_name: &str) {
        let _ = self.mp.println(format!(
            "  {} 执行工具: {}",
            "⚙".bright_yellow(),
            tool_name.bright_cyan()
        ));
    }

    /// 显示工具执行成功
    pub fn tool_success(&self, tool_name: &str) {
        let _ = self.mp.println(format!(
            "  {} 工具 {} 执行成功",
            "✓".green(),
            tool_name.bright_cyan()
        ));
    }

    /// 显示工具执行错误
    pub fn tool_error(&self, tool_name: &str, error: &str) {
        let _ = self.mp.println(format!(
            "  {} 工具 {} 执行失败: {}",
            "✗".red(),
            tool_name.bright_cyan(),
            error
        ));
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
