//! 状态行显示
//!
//! 显示实时更新的任务状态（底部状态行）

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 状态行内部状态
struct StatusLineInner {
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
    /// 是否已暂停
    suspended: bool,
}

/// 状态行显示器（可 Clone，线程安全）
#[derive(Clone)]
pub struct StatusLine {
    inner: Arc<Mutex<StatusLineInner>>,
}

impl StatusLine {
    /// 创建新的状态行显示器
    pub fn new(mp: Arc<MultiProgress>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(StatusLineInner {
                mp,
                status_bar: None,
                start_time: None,
                current_status: None,
                token_count: 0,
                suspended: false,
            })),
        }
    }

    /// 获取底部状态栏（用于 insert_before）
    pub fn bar(&self) -> Option<ProgressBar> {
        let inner = self.inner.lock().unwrap();
        inner.status_bar.clone()
    }

    /// 开始任务
    pub fn start(&mut self, status: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.start_time = Some(Instant::now());
        inner.current_status = Some(status.to_string());
        inner.token_count = 0;
        inner.suspended = false;

        // 创建状态栏（添加到 MultiProgress 最后，确保在最下面）
        let bar = inner.mp.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.cyan} {prefix}… (esc to interrupt · {elapsed} · ↓ {msg} tokens)")
                .unwrap(),
        );

        // 立即显示初始状态
        bar.set_prefix(status.bright_white().to_string());
        bar.set_message("0");
        bar.enable_steady_tick(Duration::from_millis(100));

        inner.status_bar = Some(bar);
    }

    /// 暂停显示（工具执行期间隐藏）
    pub fn suspend(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.suspended {
            return;
        }
        inner.suspended = true;
        if let Some(bar) = inner.status_bar.take() {
            bar.finish_and_clear();
        }
    }

    /// 恢复显示
    pub fn resume(&self) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.suspended {
            return;
        }
        inner.suspended = false;

        // 重新创建状态栏
        let bar = inner.mp.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.cyan} {prefix}… (esc to interrupt · {elapsed} · ↓ {msg} tokens)")
                .unwrap(),
        );

        let status = inner.current_status.as_deref().unwrap_or("Processing");
        bar.set_prefix(status.bright_white().to_string());
        bar.set_message(inner.token_count.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));

        inner.status_bar = Some(bar);
    }

    /// 更新状态（原地更新）
    pub fn update(&mut self, status: &str, tokens: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.current_status = Some(status.to_string());
        inner.token_count = tokens;

        if let Some(bar) = &inner.status_bar {
            bar.set_prefix(status.bright_white().to_string());
            bar.set_message(tokens.to_string());
        }
    }

    /// 完成任务
    pub fn finish(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.start_time.is_none() {
            return;
        }

        let elapsed = inner
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::from_secs(0));

        let elapsed_str = format_duration(elapsed);

        // 显示完成状态
        if let Some(bar) = inner.status_bar.take() {
            bar.set_style(ProgressStyle::default_bar().template("{msg}").unwrap());
            bar.finish_with_message(format!(
                "{} Completed in {}",
                "✻".bright_cyan(),
                elapsed_str
            ));
        }

        inner.start_time = None;
        inner.current_status = None;
        inner.token_count = 0;
    }

    /// 清除状态行
    pub fn clear(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(bar) = inner.status_bar.take() {
            bar.finish_and_clear();
        }
    }

    /// 获取开始时间
    pub fn start_time(&self) -> Option<Instant> {
        let inner = self.inner.lock().unwrap();
        inner.start_time
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
