use crossterm::{
    cursor::{self, MoveTo},
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 状态栏
pub struct StatusBar {
    width: u16,
    height: u16,
    total_tokens: Arc<AtomicU64>,
    session_id: String,
    model_name: String,
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl StatusBar {
    pub fn new(
        total_tokens: Arc<AtomicU64>,
        session_id: String,
        model_name: String,
    ) -> Self {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        Self {
            width,
            height,
            total_tokens,
            session_id,
            model_name,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// 初始化状态栏
    pub fn init(&mut self) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        self.width = width;
        self.height = height;

        // 初始渲染状态栏
        self.render()?;

        Ok(())
    }

    /// 渲染状态栏（保存/恢复光标位置）
    pub fn render(&self) -> anyhow::Result<()> {
        let mut stdout = stdout();
        let tokens = self.total_tokens.load(Ordering::Relaxed);

        // 保存光标位置
        queue!(stdout, cursor::SavePosition)?;

        // 移动到状态栏行（终端底部）
        queue!(stdout, MoveTo(0, self.height - 1))?;

        // 构建状态栏内容
        let cwd = std::env::current_dir()
            .map(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.to_string_lossy().to_string())
            })
            .unwrap_or_else(|_| "?".to_string());

        let info = format!(
            " Tokens: {} | {} | {} | {} ",
            tokens, cwd, self.model_name, &self.session_id[..8.min(self.session_id.len())]
        );

        // 计算填充
        let padding_len = (self.width as usize).saturating_sub(info.len());
        let padding = " ".repeat(padding_len);

        queue!(
            stdout,
            SetBackgroundColor(Color::Rgb { r: 68, g: 68, b: 68 }),
            SetForegroundColor(Color::White),
            Print(&info),
            Print(&padding),
            ResetColor
        )?;

        // 恢复光标位置
        queue!(stdout, cursor::RestorePosition)?;
        stdout.flush()?;

        Ok(())
    }

    /// 启动后台刷新线程
    pub fn start_refresh(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(true, Ordering::Relaxed);
        let running = self.running.clone();
        let total_tokens = self.total_tokens.clone();
        let session_id = self.session_id.clone();
        let model_name = self.model_name.clone();
        let width = self.width;
        let height = self.height;

        let handle = std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(500));

                // 渲染状态栏
                let mut stdout = stdout();
                let tokens = total_tokens.load(Ordering::Relaxed);

                let cwd = std::env::current_dir()
                    .map(|p| {
                        p.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|_| "?".to_string());

                let info = format!(
                    " Tokens: {} | {} | {} | {} ",
                    tokens, cwd, model_name, &session_id[..8.min(session_id.len())]
                );

                let padding_len = (width as usize).saturating_sub(info.len());
                let padding = " ".repeat(padding_len);

                let _ = queue!(stdout, cursor::SavePosition);
                let _ = queue!(stdout, MoveTo(0, height - 1));
                let _ = queue!(
                    stdout,
                    SetBackgroundColor(Color::Rgb { r: 68, g: 68, b: 68 }),
                    SetForegroundColor(Color::White),
                    Print(&info),
                    Print(&padding),
                    ResetColor
                );
                let _ = queue!(stdout, cursor::RestorePosition);
                let _ = stdout.flush();
            }
        });

        self.handle = Some(handle);
    }

    /// 停止后台刷新
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// 清理状态栏
    pub fn cleanup(&mut self) -> anyhow::Result<()> {
        self.stop();

        let mut stdout = stdout();

        // 清除状态栏
        queue!(
            stdout,
            cursor::SavePosition,
            MoveTo(0, self.height - 1),
            Clear(ClearType::CurrentLine),
            cursor::RestorePosition
        )?;
        stdout.flush()?;

        Ok(())
    }
}

impl Drop for StatusBar {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
