//! 进度指示器
//!
//! 异步 Spinner 用于显示处理中状态。

use crossterm::{
    cursor::{Hide, MoveToColumn, Show},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, interval};

/// Spinner 帧动画
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// 进度指示器
pub struct Spinner {
    /// 显示的消息
    message: String,
    /// 是否正在运行
    running: Arc<AtomicBool>,
    /// 任务句柄
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Spinner {
    /// 创建新的 Spinner
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// 启动 Spinner
    pub fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let message = self.message.clone();

        // 隐藏光标
        let _ = execute!(io::stdout(), Hide);

        self.handle = Some(tokio::spawn(async move {
            let mut frame_idx = 0;
            let mut ticker = interval(Duration::from_millis(80));

            while running.load(Ordering::SeqCst) {
                ticker.tick().await;

                let frame = SPINNER_FRAMES[frame_idx % SPINNER_FRAMES.len()];
                let mut stdout = io::stdout();

                let _ = execute!(
                    stdout,
                    MoveToColumn(0),
                    Clear(ClearType::CurrentLine),
                    SetForegroundColor(Color::Cyan),
                    Print(frame),
                    Print(" "),
                    ResetColor,
                    Print(&message),
                );
                let _ = stdout.flush();

                frame_idx += 1;
            }
        }));
    }

    /// 停止 Spinner
    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }

        // 清除 Spinner 行并显示光标
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            Show
        );
        let _ = stdout.flush();
    }

    /// 更新消息
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 停止并显示成功消息
    pub async fn success(&mut self, message: &str) {
        self.stop().await;
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("✓ "),
            ResetColor,
            Print(message),
            Print("\n")
        );
        let _ = stdout.flush();
    }

    /// 停止并显示错误消息
    pub async fn error(&mut self, message: &str) {
        self.stop().await;
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            SetForegroundColor(Color::Red),
            Print("✗ "),
            ResetColor,
            Print(message),
            Print("\n")
        );
        let _ = stdout.flush();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        // 确保光标可见
        let _ = execute!(io::stdout(), Show);
    }
}
