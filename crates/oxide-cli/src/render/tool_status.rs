//! 工具状态显示
//!
//! 支持原地更新的工具执行状态显示

use colored::Colorize;
use std::io::{self, Write};

/// 工具状态
#[derive(Debug, Clone, PartialEq)]
pub enum ToolStatus {
    /// 调用中
    Calling,
    /// 执行中（带描述）
    Executing(String),
    /// 成功
    Success,
    /// 失败
    Error(String),
}

/// 工具状态显示器
pub struct ToolStatusDisplay {
    /// 当前工具名称
    current_tool: Option<String>,
    /// 当前描述
    current_desc: Option<String>,
    /// 当前状态
    current_status: Option<ToolStatus>,
    /// 是否已显示
    is_displayed: bool,
    /// Spinner 帧
    spinner_frames: Vec<&'static str>,
    /// 当前帧索引
    frame_index: usize,
}

impl ToolStatusDisplay {
    /// 创建新的工具状态显示器
    pub fn new() -> Self {
        Self {
            current_tool: None,
            current_desc: None,
            current_status: None,
            is_displayed: false,
            spinner_frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            frame_index: 0,
        }
    }

    /// 开始工具调用（显示初始状态）
    pub fn start_tool(&mut self, tool_name: &str, description: &str) -> io::Result<()> {
        println!();
        println!("⏺ {}({})", tool_name, description);
        self.current_tool = Some(tool_name.to_string());
        self.current_desc = Some(description.to_string());
        self.is_displayed = true;
        Ok(())
    }

    /// 更新执行状态（带 spinner）
    pub fn update_executing(&mut self, step: &str) -> io::Result<()> {
        if !self.is_displayed {
            println!();
        }

        let frame = self.spinner_frames[self.frame_index % self.spinner_frames.len()];
        self.frame_index += 1;

        print!("\r\x1B[2K⎿  {} {}", frame, step);
        io::stdout().flush()?;
        Ok(())
    }

    /// 完成工具调用（带统计信息）
    pub fn finish_tool(&mut self, summary: Option<&str>) -> io::Result<()> {
        if let Some(info) = summary {
            print!("\r\x1B[2K⎿  Done ({})", info);
        } else {
            print!("\r\x1B[2K⎿  Done");
        }
        io::stdout().flush()?;
        println!();
        self.is_displayed = false;
        self.current_tool = None;
        self.current_desc = None;
        self.frame_index = 0;
        Ok(())
    }

    /// 更新工具状态
    pub fn update(&mut self, tool_name: &str, status: ToolStatus) -> io::Result<()> {
        let mut stdout = io::stdout();

        // 如果已经显示过，使用 ANSI 转义序列回到行首并清除行
        if self.is_displayed {
            print!("\r\x1B[2K");
            stdout.flush()?;
        } else {
            // 首次显示，先换行
            println!();
        }

        // 根据状态显示不同的图标和文本
        let (icon, text, color_fn): (&str, String, fn(&str) -> colored::ColoredString) = match status {
            ToolStatus::Calling => (
                "🔧",
                format!("调用工具: {}", tool_name),
                |s| s.bright_yellow(),
            ),
            ToolStatus::Executing(ref desc) => (
                "⚙",
                format!("执行工具: {} - {}", tool_name, desc),
                |s| s.bright_cyan(),
            ),
            ToolStatus::Success => (
                "✓",
                format!("工具 {} 执行成功", tool_name),
                |s| s.green(),
            ),
            ToolStatus::Error(ref err) => (
                "✗",
                format!("工具 {} 执行失败: {}", tool_name, err),
                |s| s.red(),
            ),
        };

        // 显示状态
        print!("{} {}", icon, color_fn(&text));
        stdout.flush()?;

        // 如果是最终状态（成功或失败），换行并重置
        if matches!(status, ToolStatus::Success | ToolStatus::Error(_)) {
            println!();
            self.is_displayed = false;
            self.current_tool = None;
            self.current_status = None;
        } else {
            self.is_displayed = true;
            self.current_tool = Some(tool_name.to_string());
            self.current_status = Some(status);
        }

        Ok(())
    }

    /// 清除当前显示
    pub fn clear(&mut self) -> io::Result<()> {
        if self.is_displayed {
            let mut stdout = io::stdout();
            print!("\r\x1B[2K");
            stdout.flush()?;
            self.is_displayed = false;
            self.current_tool = None;
            self.current_status = None;
        }
        Ok(())
    }
}

impl Default for ToolStatusDisplay {
    fn default() -> Self {
        Self::new()
    }
}
