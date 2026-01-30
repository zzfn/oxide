//! 渲染模块
//!
//! 提供 Markdown 渲染、进度指示器和流式输出功能。

pub mod markdown;
pub mod spinner;
pub mod stream;

pub use markdown::MarkdownRenderer;
pub use spinner::Spinner;
pub use stream::{StreamChannel, StreamRenderer};

use colored::Colorize;
use std::io::Write;

/// 统一渲染接口
pub struct Renderer {
    markdown: MarkdownRenderer,
}

impl Renderer {
    /// 创建新的渲染器
    pub fn new() -> Self {
        Self {
            markdown: MarkdownRenderer::new(),
        }
    }

    /// 显示欢迎信息
    pub fn welcome(&self) {
        println!();
        println!("{}", "╭─────────────────────────────────────╮".bright_cyan());
        println!("{}", "│         Oxide - AI 编程助手         │".bright_cyan());
        println!("{}", "╰─────────────────────────────────────╯".bright_cyan());
        println!();
        println!("  {} 输入问题开始对话", "•".green());
        println!("  {} 输入 {} 查看帮助", "•".green(), "/help".yellow());
        println!("  {} 按 {} 两次退出", "•".green(), "Ctrl+C".yellow());
        println!();
    }

    /// 显示分隔线
    pub fn separator(&self) {
        println!("{}", "─".repeat(40).bright_black());
    }

    /// 显示助手响应
    pub fn assistant_response(&self, content: &str) {
        println!();
        println!("{}", "Assistant".bright_blue().bold());
        self.markdown.print(content);
        println!();
    }

    /// 显示助手响应头部（用于流式输出）
    pub fn assistant_header(&self) {
        println!();
        print!("{} ", "Assistant".bright_blue().bold());
        let _ = std::io::stdout().flush();
    }

    /// 显示用户输入提示
    pub fn user_prompt(&self) {
        print!("{} ", ">".bright_green().bold());
        let _ = std::io::stdout().flush();
    }

    /// 显示错误信息
    pub fn error(&self, message: &str) {
        println!("{} {}", "Error:".red().bold(), message);
    }

    /// 显示警告信息
    pub fn warning(&self, message: &str) {
        println!("{} {}", "Warning:".yellow().bold(), message);
    }

    /// 显示信息
    pub fn info(&self, message: &str) {
        println!("{} {}", "Info:".blue().bold(), message);
    }

    /// 显示成功信息
    pub fn success(&self, message: &str) {
        println!("{} {}", "✓".green().bold(), message);
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
        println!("  {} 执行工具: {}", "⚙".bright_yellow(), tool_name.bright_cyan());
    }

    /// 显示工具执行成功
    pub fn tool_success(&self, tool_name: &str) {
        println!("  {} 工具 {} 执行成功", "✓".green(), tool_name.bright_cyan());
    }

    /// 显示工具执行错误
    pub fn tool_error(&self, tool_name: &str, error: &str) {
        println!("  {} 工具 {} 执行失败: {}", "✗".red(), tool_name.bright_cyan(), error);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
