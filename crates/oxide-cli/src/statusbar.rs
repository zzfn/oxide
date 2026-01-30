//! 状态栏
//!
//! 在终端底部显示当前模式、Token 使用统计、工作目录等信息。

use crossterm::{
    cursor::{MoveTo, SavePosition, RestorePosition},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};

use crate::app::SharedAppState;

/// 状态栏渲染器
pub struct StatusBar {
    /// 共享应用状态
    state: SharedAppState,
    /// 是否启用状态栏
    enabled: bool,
}

impl StatusBar {
    /// 创建新的状态栏
    pub fn new(state: SharedAppState) -> Self {
        Self {
            state,
            enabled: true,
        }
    }

    /// 启用状态栏
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用状态栏
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 渲染状态栏
    pub async fn render(&self) -> io::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let state = self.state.read().await;
        let (cols, rows) = terminal::size()?;
        let mut stdout = io::stdout();

        // 保存当前光标位置
        execute!(stdout, SavePosition)?;

        // 移动到最后一行
        execute!(stdout, MoveTo(0, rows - 1))?;

        // 清除该行
        execute!(stdout, Clear(ClearType::CurrentLine))?;

        // 构建状态栏内容
        let mode_str = format!(" [{}] ", state.mode.display_name());
        let token_str = format!(" {} ", state.token_usage.format());
        let dir_str = format!(
            " {} ",
            state
                .working_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(".")
        );

        // 计算任务状态
        let task_str = if state.background_tasks.running > 0 {
            format!(" ⚙ {} ", state.background_tasks.running)
        } else {
            String::new()
        };

        // 计算处理状态
        let processing_str = if state.is_processing { " ● " } else { "" };

        // 渲染状态栏
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
        )?;

        // 模式（高亮）
        let mode_color = match state.mode {
            crate::app::CliMode::Normal => Color::Green,
            crate::app::CliMode::Fast => Color::Yellow,
            crate::app::CliMode::Plan => Color::Cyan,
        };
        execute!(
            stdout,
            SetBackgroundColor(mode_color),
            SetForegroundColor(Color::Black),
            Print(&mode_str),
        )?;

        // Token 使用
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(&token_str),
        )?;

        // 工作目录
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print(&dir_str),
        )?;

        // 任务状态
        if !task_str.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print(&task_str),
            )?;
        }

        // 处理状态
        if !processing_str.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(processing_str),
            )?;
        }

        // 填充剩余空间
        let used_width = mode_str.len() + token_str.len() + dir_str.len() + task_str.len() + processing_str.len();
        if used_width < cols as usize {
            let padding = " ".repeat(cols as usize - used_width);
            execute!(stdout, Print(padding))?;
        }

        // 重置颜色并恢复光标位置
        execute!(stdout, ResetColor, RestorePosition)?;
        stdout.flush()?;

        Ok(())
    }

    /// 清除状态栏
    pub fn clear(&self) -> io::Result<()> {
        let (_, rows) = terminal::size()?;
        let mut stdout = io::stdout();

        execute!(
            stdout,
            SavePosition,
            MoveTo(0, rows - 1),
            Clear(ClearType::CurrentLine),
            RestorePosition
        )?;
        stdout.flush()?;

        Ok(())
    }
}
