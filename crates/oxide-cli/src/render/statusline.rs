//! 状态行显示
//!
//! 显示实时更新的任务状态（底部状态行）

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 状态行显示器
pub struct StatusLine {
    /// MultiProgress 管理器
    mp: Arc<MultiProgress>,
    /// 底部状态栏
    status_bar: Option<ProgressBar>,
    /// 开始时间
    start_time: Option<Instant>,
    /// 当前状态文本
    current_status: Option<String>,
    /// Token 计数
    token_count: usize,
}

impl StatusLine {
    /// 创建新的状态行显示器
    pub fn new(mp: Arc<MultiProgress>) -> Self {
        Self {
            mp,
            status_bar: None,
            start_time: None,
            current_status: None,
            token_count: 0,
        }
    }

    /// 获取底部状态栏（用于 insert_before）
    pub fn bar(&self) -> Option<&ProgressBar> {
        self.status_bar.as_ref()
    }

    /// 开始任务
    pub fn start(&mut self, status: &str) {
        self.start_time = Some(Instant::now());
        self.current_status = Some(status.to_string());
        self.token_count = 0;

        // 创建状态栏（添加到 MultiProgress 最后，确保在最下面）
        let bar = self.mp.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );

        // 立即显示初始状态
        self.update_display(&bar, status, 0);
        bar.enable_steady_tick(Duration::from_millis(100));

        self.status_bar = Some(bar);
    }

    /// 更新状态（原地更新）
    pub fn update(&mut self, status: &str, tokens: usize) {
        self.current_status = Some(status.to_string());
        self.token_count = tokens;

        if let Some(bar) = &self.status_bar {
            self.update_display(bar, status, tokens);
        }
    }

    /// 内部方法：更新显示
    fn update_display(&self, bar: &ProgressBar, status: &str, tokens: usize) {
        let elapsed = self
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::from_secs(0));

        let elapsed_str = format_duration(elapsed);

        let msg = format!(
            "{}… (esc to interrupt · {} · ↓ {} tokens)",
            status.bright_white(),
            elapsed_str,
            tokens
        );

        bar.set_message(msg);
    }

    /// 完成任务
    pub fn finish(&mut self) {
        if self.start_time.is_none() {
            return;
        }

        let elapsed = self
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::from_secs(0));

        let elapsed_str = format_duration(elapsed);

        // 显示完成状态
        if let Some(bar) = self.status_bar.take() {
            bar.set_style(ProgressStyle::default_bar().template("{msg}").unwrap());
            bar.finish_with_message(format!(
                "{} Completed in {}",
                "✻".bright_cyan(),
                elapsed_str
            ));
        }

        self.start_time = None;
        self.current_status = None;
        self.token_count = 0;
    }

    /// 清除状态行
    pub fn clear(&mut self) {
        if let Some(bar) = self.status_bar.take() {
            bar.finish_and_clear();
        }
    }

    /// 获取开始时间
    pub fn start_time(&self) -> Option<Instant> {
        self.start_time
    }
}

/// 格式化持续时间
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    if total_secs < 60 {
        format!("{}s", total_secs)
    } else {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {}s", mins, secs)
    }
}
