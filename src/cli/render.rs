use anyhow::Result;
use colored::*;
use crossterm::{
    cursor::{self, MoveTo},
    queue,
    terminal::{self, ScrollUp, Clear, ClearType},
};
use futures::StreamExt;
use rig::agent::{FinalResponse, MultiTurnStreamItem, StreamingResult};
use rig::streaming::StreamedAssistantContent;
use std::io::{stdout, Write};
use std::sync::OnceLock;
use std::time::Duration;
use termimad::MadSkin;
use tokio::sync::oneshot;
use tokio::time::interval;

use super::OxideCli;

/// 全局 Markdown 渲染器（线程安全）
static MAD_SKIN: OnceLock<MadSkin> = OnceLock::new();

/// 获取配置好的 MadSkin
fn get_mad_skin() -> &'static MadSkin {
    MAD_SKIN.get_or_init(|| {
        let mut skin = MadSkin::default();

        // 自定义样式
        skin.set_headers_fg(termimad::crossterm::style::Color::Cyan);
        skin.bold.set_fg(termimad::crossterm::style::Color::White);
        skin.italic.set_fg(termimad::crossterm::style::Color::Yellow);
        skin.inline_code.set_fg(termimad::crossterm::style::Color::Green);

        // 设置代码块样式（灰色背景）
        skin.code_block.set_fg(termimad::crossterm::style::Color::AnsiValue(245));
        skin.code_block.set_bg(termimad::crossterm::style::Color::AnsiValue(233));

        skin
    })
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            shutdown_tx: None,
        }
    }

    pub fn start(&mut self, message: &str) {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let message = message.to_string();

        tokio::spawn(async move {
            let mut frame = 0;
            let mut ticker = interval(Duration::from_millis(100));
            ticker.tick().await;

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        // Clear the spinner line
                        print!("\r{}\r", " ".repeat(80));
                        use std::io::Write;
                        std::io::stdout().flush().unwrap();
                        break;
                    }
                    _ = ticker.tick() => {
                        let spinner = SPINNER_FRAMES[frame % SPINNER_FRAMES.len()];
                        print!("\r{} {}", spinner.yellow(), message.dimmed());
                        use std::io::Write;
                        std::io::stdout().flush().unwrap();
                        frame += 1;
                    }
                }
            }
        });
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // Give the spinner task a moment to clean up
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Markdown 流式渲染器
struct MarkdownStreamRenderer {
    buffer: String,
    line_buffer: String,
    in_code_block: bool,
    in_list: bool,
}

impl MarkdownStreamRenderer {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            line_buffer: String::new(),
            in_code_block: false,
            in_list: false,
        }
    }

    /// 处理流式文本并输出
    fn process_text(&mut self, text: &str, skin: &MadSkin) {
        for ch in text.chars() {
            self.line_buffer.push(ch);

            // 检测是否在代码块中
            if self.line_buffer.contains("```") {
                self.in_code_block = !self.in_code_block;
                self.line_buffer.clear();
                continue;
            }

            // 遇到换行符时处理整行
            if ch == '\n' {
                self.flush_line(skin);
            }
        }

        // 将文本添加到缓冲区，用于最终渲染
        self.buffer.push_str(text);
    }

    /// 在滚动区域内打印一行（会滚动）
    /// 布局：[0..height-6] 响应区 | [height-5] 上边框 | [height-4] 输入框 | [height-3] 下边框 | [height-2] 间距 | [height-1] 状态栏
    fn print_in_scroll_region(text: &str) {
        let mut stdout = stdout();
        if let Ok((_, height)) = terminal::size() {
            let scroll_bottom = height.saturating_sub(6);

            let _ = queue!(stdout, cursor::SavePosition);
            let _ = queue!(stdout, MoveTo(0, scroll_bottom));
            let _ = queue!(stdout, ScrollUp(1));
            let _ = queue!(stdout, MoveTo(0, scroll_bottom));
            let _ = queue!(stdout, Clear(ClearType::CurrentLine));
            let _ = queue!(stdout, crossterm::style::Print(text));
            let _ = queue!(stdout, cursor::RestorePosition);
            let _ = stdout.flush();
        } else {
            // 回退到普通输出
            print!("{}", text);
            let _ = stdout.flush();
        }
    }

    /// 在滚动区域底部原地更新（不滚动，用于 spinner 动画）
    fn print_at_scroll_bottom(text: &str) {
        let mut stdout = stdout();
        if let Ok((_, height)) = terminal::size() {
            let scroll_bottom = height.saturating_sub(6);

            let _ = queue!(stdout, cursor::SavePosition);
            let _ = queue!(stdout, MoveTo(0, scroll_bottom));
            let _ = queue!(stdout, Clear(ClearType::CurrentLine));
            let _ = queue!(stdout, crossterm::style::Print(text));
            let _ = queue!(stdout, cursor::RestorePosition);
            let _ = stdout.flush();
        } else {
            // 回退到普通输出
            print!("\r{}", text);
            let _ = stdout.flush();
        }
    }

    /// 刷新当前行到输出
    fn flush_line(&mut self, skin: &MadSkin) {
        let line = self.line_buffer.clone();

        let output = if self.in_code_block {
            // 代码块内直接输出
            line.clone()
        } else {
            // 渲染 Markdown 格式
            // 检测列表项
            if line.trim_start().starts_with("- ") || line.trim_start().starts_with("* ") {
                self.in_list = true;
            } else if !line.trim().is_empty() && !line.trim_start().starts_with("    ") {
                self.in_list = false;
            }

            // 使用 termimad 渲染行
            skin.inline(&line).to_string()
        };

        // 在滚动区域内输出（去掉末尾换行符，因为 ScrollUp 已经处理了）
        let output_trimmed = output.trim_end_matches('\n');
        Self::print_in_scroll_region(output_trimmed);

        self.line_buffer.clear();
    }

    /// 完成流式输出，渲染完整格式
    fn finish(self, skin: &MadSkin) {
        // 刷新剩余内容
        if !self.line_buffer.is_empty() {
            let line = self.line_buffer;
            let rendered = skin.inline(&line);
            Self::print_in_scroll_region(&rendered.to_string());
        }

        // 输出额外的空行分隔
        Self::print_in_scroll_region("");
    }
}

/// 自定义流式输出函数，替代 rig 的 stream_to_stdout
/// 去掉 "Response:" 前缀，并在 "● oxide:" 后添加动画效果
/// 支持实时 Markdown 渲染
pub async fn stream_with_animation<R>(
    stream: &mut StreamingResult<R>,
) -> Result<FinalResponse, std::io::Error>
where
    R: Send + 'static,
{
    let mut final_res = FinalResponse::empty();
    let (stop_spinner_tx, mut stop_spinner_rx) = oneshot::channel();
    let mut stop_spinner_tx = Some(stop_spinner_tx);

    // 启动动画 spinner（在滚动区域内）
    let mut spinner_handle = Some(tokio::spawn(async move {
        let mut frame = 0;
        let mut ticker = interval(Duration::from_millis(100));
        ticker.tick().await;

        loop {
            tokio::select! {
                _ = &mut stop_spinner_rx => {
                    // 停止 spinner，显示静态图标
                    MarkdownStreamRenderer::print_at_scroll_bottom(&format!("● oxide: "));
                    break;
                }
                _ = ticker.tick() => {
                    let spinner = SPINNER_FRAMES[frame % SPINNER_FRAMES.len()];
                    MarkdownStreamRenderer::print_at_scroll_bottom(&format!("{} oxide:", spinner.blue()));
                    frame += 1;
                }
            }
        }
    }));

    // 等待第一个内容块
    let mut first_content = true;
    let mut renderer = MarkdownStreamRenderer::new();
    let skin = get_mad_skin();

    while let Some(content) = stream.next().await {
        match content {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(
                text,
            ))) => {
                if first_content {
                    // 收到第一个文本块，停止 spinner
                    if let Some(tx) = stop_spinner_tx.take() {
                        let _ = tx.send(());
                    }
                    // 等待 spinner 清理完成
                    if let Some(handle) = spinner_handle.take() {
                        let _ = handle.await;
                    }
                    first_content = false;
                }

                // 使用 Markdown 渲染器处理文本
                renderer.process_text(&text.text, skin);
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(
                StreamedAssistantContent::Reasoning(r),
            )) => {
                if first_content {
                    // 收到第一个内容块，停止 spinner
                    if let Some(tx) = stop_spinner_tx.take() {
                        let _ = tx.send(());
                    }
                    if let Some(handle) = spinner_handle.take() {
                        let _ = handle.await;
                    }
                    first_content = false;
                }
                let reasoning = r.reasoning.join("\n");
                // Reasoning 内容在滚动区域输出
                MarkdownStreamRenderer::print_in_scroll_region(&reasoning.dimmed().to_string());
            }
            Ok(MultiTurnStreamItem::FinalResponse(res)) => {
                final_res = res;
            }
            Err(err) => {
                let err_msg = err.to_string();
                if err_msg.contains("PromptCancelled") {
                    if let Some(tx) = stop_spinner_tx.take() {
                        let _ = tx.send(());
                    }
                    if let Some(handle) = spinner_handle.take() {
                        let _ = handle.await;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        "prompt_cancelled",
                    ));
                }
                eprintln!("Error: {}", err);
            }
            _ => {}
        }
    }

    // 完成渲染
    renderer.finish(skin);

    // 如果流式输出结束还没有收到任何内容，停止 spinner
    if first_content {
        if let Some(tx) = stop_spinner_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = spinner_handle.take() {
            let _ = handle.await;
        }
    }

    Ok(final_res)
}

impl OxideCli {
    pub fn show_welcome(&self) -> Result<()> {
        println!("{}", "✨ Welcome to Oxide CLI v0.1.0!".bright_green());
        println!(
            "{} {} | {} {} | {} {}",
            "Session:".dimmed(),
            self.context_manager.session_id(),
            "cwd:".dimmed(),
            std::env::current_dir().unwrap().display(),
            "model:".dimmed(),
            self.model_name
        );
        println!();
        Ok(())
    }

    pub fn show_tips(&self) -> Result<()> {
        println!("{}", "Tips for getting started:".bright_white());
        println!();
        println!(
            "{} Ask questions, edit files, or run commands.",
            "1.".bright_white()
        );
        println!("{} Be specific for the best results.", "2.".bright_white());
        println!("{} Type /help for more information.", "3.".bright_white());
        println!();
        println!(
            "{}",
            "ctrl+c twice within 1s to exit, /help for commands, Tab for completion".dimmed()
        );
        println!();
        Ok(())
    }
}
