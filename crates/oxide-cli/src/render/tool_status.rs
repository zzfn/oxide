//! 工具状态显示
//!
//! 支持原地更新的工具执行状态显示

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io;
use std::sync::Arc;
use std::time::Duration;

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
    /// MultiProgress 管理器
    mp: Arc<MultiProgress>,
    /// 当前工具进度条
    current_bar: Option<ProgressBar>,
    /// 当前工具名称
    current_tool: Option<String>,
    /// Spinner 帧
    spinner_frames: Vec<&'static str>,
    /// 当前帧索引
    frame_index: usize,
}

impl ToolStatusDisplay {
    /// 创建新的工具状态显示器
    pub fn new(mp: Arc<MultiProgress>) -> Self {
        Self {
            mp,
            current_bar: None,
            current_tool: None,
            spinner_frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            frame_index: 0,
        }
    }

    /// 开始工具调用（显示初始状态）
    pub fn start_tool(&mut self, tool_name: &str, description: &str) -> io::Result<()> {
        // 先输出工具调用信息
        self.mp
            .println(format!("\n{} {}({})", "⏺".green(), tool_name, description))?;

        // 创建进度条
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&self.spinner_frames)
                .template("⎿  {spinner} {msg}")
                .unwrap(),
        );
        bar.enable_steady_tick(Duration::from_millis(80));

        self.current_bar = Some(self.mp.add(bar));
        self.current_tool = Some(tool_name.to_string());
        self.frame_index = 0;

        Ok(())
    }

    /// 开始工具调用（在指定进度条之前插入）
    pub fn start_tool_before(
        &mut self,
        tool_name: &str,
        description: &str,
        before: &ProgressBar,
    ) -> io::Result<()> {
        // 先输出工具调用信息
        self.mp
            .println(format!("\n{} {}({})", "⏺".green(), tool_name, description))?;

        // 创建进度条（在指定位置之前插入）
        let bar = self
            .mp
            .insert_before(before, ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&self.spinner_frames)
                .template("⎿  {spinner:.yellow} {msg}")
                .unwrap(),
        );
        bar.enable_steady_tick(Duration::from_millis(80));

        self.current_bar = Some(bar);
        self.current_tool = Some(tool_name.to_string());
        self.frame_index = 0;

        Ok(())
    }

    /// 更新执行状态（带 spinner）
    pub fn update_executing(&mut self, step: &str) -> io::Result<()> {
        if let Some(bar) = &self.current_bar {
            bar.set_message(step.to_string());
        }
        Ok(())
    }

    /// 完成工具调用（带统计信息）
    pub fn finish_tool(&mut self, summary: Option<&str>) -> io::Result<()> {
        if let Some(bar) = self.current_bar.take() {
            let msg = if let Some(info) = summary {
                format!("{} ({})", "Done".bright_black(), info)
            } else {
                "Done".bright_black().to_string()
            };
            bar.finish_with_message(msg);
        }

        self.current_tool = None;
        self.frame_index = 0;
        Ok(())
    }

    /// 更新工具状态
    pub fn update(&mut self, tool_name: &str, status: ToolStatus) -> io::Result<()> {
        // 根据状态显示不同的图标和文本
        let text = match status {
            ToolStatus::Calling => format!("调用工具: {}", tool_name),
            ToolStatus::Executing(ref desc) => format!("执行工具: {} - {}", tool_name, desc),
            ToolStatus::Success => format!("工具 {} 执行成功", tool_name),
            ToolStatus::Error(ref err) => format!("工具 {} 执行失败: {}", tool_name, err),
        };

        self.mp.println(text)?;
        Ok(())
    }

    /// 清除当前显示
    pub fn clear(&mut self) -> io::Result<()> {
        if let Some(bar) = self.current_bar.take() {
            bar.finish_and_clear();
        }
        self.current_tool = None;
        Ok(())
    }
}
