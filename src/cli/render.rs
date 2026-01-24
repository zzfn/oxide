use anyhow::Result;
use colored::*;
use futures::StreamExt;
use rig::agent::{FinalResponse, MultiTurnStreamItem, StreamingResult};
use rig::streaming::StreamedAssistantContent;
use std::io::{stdout, Write};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::interval;

use super::OxideCli;

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

/// 自定义流式输出函数，替代 rig 的 stream_to_stdout
/// 去掉 "Response:" 前缀，并在 "● oxide:" 后添加动画效果
pub async fn stream_with_animation<R>(
    stream: &mut StreamingResult<R>,
) -> Result<FinalResponse, std::io::Error>
where
    R: Send + 'static,
{
    let mut final_res = FinalResponse::empty();
    let (stop_spinner_tx, mut stop_spinner_rx) = oneshot::channel();
    let mut stop_spinner_tx = Some(stop_spinner_tx);

    // 启动动画 spinner
    let mut spinner_handle = Some(tokio::spawn(async move {
        let mut frame = 0;
        let mut ticker = interval(Duration::from_millis(100));
        ticker.tick().await;

        loop {
            tokio::select! {
                _ = &mut stop_spinner_rx => {
                    // 清除 spinner 行并显示静态图标
                    print!("\r\x1b[2K"); // 清除整行
                    print!("● oxide: ");
                    stdout().flush().unwrap();
                    break;
                }
                _ = ticker.tick() => {
                    let spinner = SPINNER_FRAMES[frame % SPINNER_FRAMES.len()];
                    print!("\r{} {}", spinner.blue(), "oxide:".dimmed());
                    stdout().flush().unwrap();
                    frame += 1;
                }
            }
        }
    }));

    // 等待第一个内容块
    let mut first_content = true;
    let mut buffer = String::new();

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
                print!("{}", text.text);
                buffer.push_str(&text.text);
                stdout().flush().unwrap();
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
                print!("{}", reasoning);
                stdout().flush().unwrap();
            }
            Ok(MultiTurnStreamItem::FinalResponse(res)) => {
                final_res = res;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
            _ => {}
        }
    }

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
