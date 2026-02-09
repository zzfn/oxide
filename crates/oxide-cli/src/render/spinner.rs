//! 进度指示器
//!
//! 异步 Spinner 用于显示处理中状态。

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, interval};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// 进度指示器
pub struct Spinner {
    message: String,
    running: Arc<AtomicBool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let message = self.message.clone();

        // 隐藏光标
        print!("\x1b[?25l");
        let _ = io::stdout().flush();

        self.handle = Some(tokio::spawn(async move {
            let mut frame_idx = 0;
            let mut ticker = interval(Duration::from_millis(80));

            while running.load(Ordering::SeqCst) {
                ticker.tick().await;

                let frame = SPINNER_FRAMES[frame_idx % SPINNER_FRAMES.len()];
                let mut stdout = io::stdout();

                // 移到行首，清除行，打印 spinner
                print!("\r\x1b[2K\x1b[36m{}\x1b[0m {}", frame, message);
                let _ = stdout.flush();

                frame_idx += 1;
            }
        }));
    }

    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }

        // 清除行并显示光标
        print!("\r\x1b[2K\x1b[?25h");
        let _ = io::stdout().flush();
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn success(&mut self, message: &str) {
        self.stop().await;
        println!("\x1b[32m✓\x1b[0m {}", message);
    }

    pub async fn error(&mut self, message: &str) {
        self.stop().await;
        println!("\x1b[31m✗\x1b[0m {}", message);
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        // 确保光标可见
        print!("\x1b[?25h");
        let _ = io::stdout().flush();
    }
}
