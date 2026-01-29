use anyhow::Result;
use colored::*;
use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
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
    line_buffer: String,
    in_code_block: bool,
    in_list: bool,
}

impl MarkdownStreamRenderer {
    fn new() -> Self {
        Self {
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
    }

    /// 在滚动区域内打印一行（会滚动）
    /// 布局：[0..height-6] 响应区 | [height-5] 上边框 | [height-4] 输入框 | [height-3] 下边框 | [height-2] 间距 | [height-1] 状态栏
    fn print_in_scroll_region(text: &str) {
        let mut stdout = stdout();
        if let Ok((width, height)) = terminal::size() {
            // 滚动区域底部行号（height-6 是响应区最后一行）
            let scroll_bottom = height.saturating_sub(6);
            // 输入框行
            let input_row = height.saturating_sub(4);

            // 设置滚动区域（1 到 height-5，不包含输入框区域）
            let scroll_region_bottom = height.saturating_sub(5);
            let _ = write!(stdout, "\x1b[1;{}r", scroll_region_bottom);

            // 移动到滚动区域底部
            let _ = queue!(stdout, MoveTo(0, scroll_bottom));

            // 滚动一行
            let _ = queue!(stdout, ScrollUp(1));

            // 移动到滚动区域底部并清除该行
            let _ = queue!(stdout, MoveTo(0, scroll_bottom));
            let _ = queue!(stdout, Clear(ClearType::CurrentLine));

            // 打印文本（截断过长的文本）
            let text_display: String = text.chars().take(width as usize).collect();
            let _ = queue!(stdout, crossterm::style::Print(&text_display));

            // 将光标移回输入框位置（不使用 SavePosition/RestorePosition）
            let _ = queue!(stdout, MoveTo(3, input_row)); // 3 = "│● " 的位置

            let _ = stdout.flush();
        } else {
            // 回退到普通输出
            print!("{}", text);
            let _ = stdout.flush();
        }
    }

    /// 在输入框上边框位置显示 loading（临时覆盖边框）
    fn print_loading_above_input(text: &str) {
        let mut stdout = stdout();
        if let Ok((width, height)) = terminal::size() {
            // loading 显示在输入框上边框位置（height-5）
            let loading_row = height.saturating_sub(5);

            let _ = queue!(stdout, cursor::SavePosition);
            let _ = queue!(stdout, MoveTo(0, loading_row));
            let _ = queue!(stdout, Clear(ClearType::CurrentLine));
            let _ = queue!(stdout, crossterm::style::Print(text));
            // 填充剩余空间，避免显示不完整
            let text_len = text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum::<usize>();
            if text_len < width as usize {
                let _ = queue!(stdout, crossterm::style::Print(" ".repeat(width as usize - text_len)));
            }
            let _ = queue!(stdout, cursor::RestorePosition);
            let _ = stdout.flush();
        } else {
            // 回退到普通输出
            print!("\r{}", text);
            let _ = stdout.flush();
        }
    }

    /// 清除 loading 并重新绘制输入框上边框
    fn clear_loading_and_restore_border() {
        let mut stdout = stdout();
        if let Ok((width, height)) = terminal::size() {
            let top_row = height.saturating_sub(5);
            let inner_width = width.saturating_sub(2) as usize;

            let _ = queue!(stdout, cursor::SavePosition);
            let _ = queue!(stdout, MoveTo(0, top_row));
            let _ = queue!(stdout, Clear(ClearType::CurrentLine));

            // 重新绘制上边框
            let _ = queue!(stdout, crossterm::style::SetForegroundColor(crossterm::style::Color::DarkGrey));
            let _ = queue!(stdout, crossterm::style::Print("╭"));
            let _ = queue!(stdout, crossterm::style::Print("─".repeat(inner_width)));
            let _ = queue!(stdout, crossterm::style::Print("╮"));
            let _ = queue!(stdout, crossterm::style::ResetColor);

            let _ = queue!(stdout, cursor::RestorePosition);
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
    }
}

/// 流式输出结果，包含响应和用户在输出期间输入的内容
pub struct StreamResult {
    pub response: FinalResponse,
    /// 用户在流式输出期间输入的内容
    pub pending_input: String,
}

/// 在输入框中渲染用户输入（流式输出期间）
fn render_pending_input(input: &str, cursor_pos: usize) {
    let mut stdout = stdout();
    if let Ok((width, height)) = terminal::size() {
        let input_row = height.saturating_sub(4);
        let prompt = "● ".blue().to_string();
        let prompt_display_len = 2; // "● " 的显示宽度

        // 直接移动到输入行，不使用 SavePosition（避免栈不平衡）
        let _ = queue!(stdout, MoveTo(0, input_row));
        let _ = queue!(stdout, Clear(ClearType::CurrentLine));

        // 绘制左边框
        let _ = queue!(stdout, crossterm::style::SetForegroundColor(crossterm::style::Color::DarkGrey));
        let _ = queue!(stdout, crossterm::style::Print("│"));
        let _ = queue!(stdout, crossterm::style::ResetColor);

        // 绘制提示符和输入内容
        let _ = queue!(stdout, crossterm::style::Print(&prompt));
        let _ = queue!(stdout, crossterm::style::Print(input));

        // 填充剩余空间
        let content_len = prompt_display_len + input.chars().count();
        let remaining = (width as usize).saturating_sub(content_len + 2); // 2 for borders
        let _ = queue!(stdout, crossterm::style::Print(" ".repeat(remaining)));

        // 绘制右边框
        let _ = queue!(stdout, crossterm::style::SetForegroundColor(crossterm::style::Color::DarkGrey));
        let _ = queue!(stdout, crossterm::style::Print("│"));
        let _ = queue!(stdout, crossterm::style::ResetColor);

        // 将光标移动到输入位置
        let cursor_col = 1 + prompt_display_len + cursor_pos;
        let _ = queue!(stdout, MoveTo(cursor_col as u16, input_row));

        let _ = stdout.flush();
    }
}

/// 自定义流式输出函数，替代 rig 的 stream_to_stdout
/// 去掉 "Response:" 前缀，并在 "● oxide:" 后添加动画效果
/// 支持实时 Markdown 渲染
/// 同时监听用户输入，允许用户在流式输出期间输入下一条消息
pub async fn stream_with_animation<R>(
    stream: &mut StreamingResult<R>,
) -> Result<StreamResult, std::io::Error>
where
    R: Send + 'static,
{
    let mut final_res = FinalResponse::empty();
    let (stop_spinner_tx, mut stop_spinner_rx) = oneshot::channel();
    let mut stop_spinner_tx = Some(stop_spinner_tx);

    // 用户输入缓存
    let mut pending_input = String::new();
    let mut cursor_pos: usize = 0;
    let mut input_dirty = false; // 标记输入是否需要重新渲染

    // 启用 raw mode 以监听键盘输入
    let _ = terminal::enable_raw_mode();

    // 启动动画 spinner（在输入框上方固定位置，带计时）
    let mut spinner_handle = Some(tokio::spawn(async move {
        let mut frame = 0;
        let mut ticker = interval(Duration::from_millis(100));
        let start_time = std::time::Instant::now();
        ticker.tick().await;

        loop {
            tokio::select! {
                _ = &mut stop_spinner_rx => {
                    // 停止 spinner，清除 loading 并恢复边框
                    MarkdownStreamRenderer::clear_loading_and_restore_border();
                    break;
                }
                _ = ticker.tick() => {
                    let spinner = SPINNER_FRAMES[frame % SPINNER_FRAMES.len()];
                    let elapsed = start_time.elapsed().as_secs();
                    MarkdownStreamRenderer::print_loading_above_input(&format!("{} oxide: {}s", spinner.blue(), elapsed));
                    frame += 1;
                }
            }
        }
    }));

    // 等待第一个内容块
    let mut first_content = true;
    let mut renderer = MarkdownStreamRenderer::new();
    let skin = get_mad_skin();

    // 键盘事件检查定时器
    let mut keyboard_ticker = interval(Duration::from_millis(16)); // ~60fps

    loop {
        tokio::select! {
            biased;

            // 优先处理流内容
            content = stream.next() => {
                match content {
                    Some(Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text)))) => {
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
                    Some(Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(r)))) => {
                        if first_content {
                            if let Some(tx) = stop_spinner_tx.take() {
                                let _ = tx.send(());
                            }
                            if let Some(handle) = spinner_handle.take() {
                                let _ = handle.await;
                            }
                            first_content = false;
                        }
                        let reasoning = r.reasoning.join("\n");
                        MarkdownStreamRenderer::print_in_scroll_region(&reasoning.dimmed().to_string());
                    }
                    Some(Ok(MultiTurnStreamItem::FinalResponse(res))) => {
                        final_res = res;
                    }
                    Some(Err(err)) => {
                        let err_msg = err.to_string();
                        if err_msg.contains("PromptCancelled") {
                            if let Some(tx) = stop_spinner_tx.take() {
                                let _ = tx.send(());
                            }
                            if let Some(handle) = spinner_handle.take() {
                                let _ = handle.await;
                            }
                            let _ = terminal::disable_raw_mode();
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Interrupted,
                                "prompt_cancelled",
                            ));
                        }
                        eprintln!("Error: {}", err);
                    }
                    Some(_) => {}
                    None => {
                        // 流结束
                        break;
                    }
                }
            }

            // 定期检查键盘输入
            _ = keyboard_ticker.tick() => {
                // 非阻塞检查键盘事件
                while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                    if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
                        match (code, modifiers) {
                            // Ctrl+C - 不处理，让流继续
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {}
                            // 普通字符输入
                            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                                let byte_pos = pending_input
                                    .char_indices()
                                    .nth(cursor_pos)
                                    .map(|(i, _)| i)
                                    .unwrap_or(pending_input.len());
                                pending_input.insert(byte_pos, c);
                                cursor_pos += 1;
                                input_dirty = true;
                            }
                            // 退格键
                            (KeyCode::Backspace, _) => {
                                if cursor_pos > 0 {
                                    cursor_pos -= 1;
                                    let byte_pos = pending_input
                                        .char_indices()
                                        .nth(cursor_pos)
                                        .map(|(i, _)| i)
                                        .unwrap_or(pending_input.len());
                                    pending_input.remove(byte_pos);
                                    input_dirty = true;
                                }
                            }
                            // 删除键
                            (KeyCode::Delete, _) => {
                                let char_count = pending_input.chars().count();
                                if cursor_pos < char_count {
                                    let byte_pos = pending_input
                                        .char_indices()
                                        .nth(cursor_pos)
                                        .map(|(i, _)| i)
                                        .unwrap_or(pending_input.len());
                                    pending_input.remove(byte_pos);
                                    input_dirty = true;
                                }
                            }
                            // 左箭头
                            (KeyCode::Left, _) => {
                                if cursor_pos > 0 {
                                    cursor_pos -= 1;
                                    input_dirty = true;
                                }
                            }
                            // 右箭头
                            (KeyCode::Right, _) => {
                                let char_count = pending_input.chars().count();
                                if cursor_pos < char_count {
                                    cursor_pos += 1;
                                    input_dirty = true;
                                }
                            }
                            // Home
                            (KeyCode::Home, _) => {
                                cursor_pos = 0;
                                input_dirty = true;
                            }
                            // End
                            (KeyCode::End, _) => {
                                cursor_pos = pending_input.chars().count();
                                input_dirty = true;
                            }
                            _ => {}
                        }
                    }
                }

                // 只在输入有变化时才重新渲染
                if input_dirty {
                    render_pending_input(&pending_input, cursor_pos);
                    input_dirty = false;
                }
            }
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

    // 退出 raw mode
    let _ = terminal::disable_raw_mode();

    Ok(StreamResult {
        response: final_res,
        pending_input,
    })
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
