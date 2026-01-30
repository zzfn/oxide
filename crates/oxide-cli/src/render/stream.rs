//! 流式输出
//!
//! 处理 AI 响应的流式输出，支持逐字符显示。

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{self, Write};
use tokio::sync::mpsc;

/// 流式渲染器
pub struct StreamRenderer {
    /// 缓冲区
    buffer: String,
    /// 是否在代码块中
    in_code_block: bool,
}

impl StreamRenderer {
    /// 创建新的流式渲染器
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_code_block: false,
        }
    }

    /// 处理流式文本片段
    pub fn write(&mut self, text: &str) -> io::Result<()> {
        let mut stdout = io::stdout();

        for ch in text.chars() {
            // 检测代码块边界
            self.buffer.push(ch);
            if self.buffer.ends_with("```") {
                self.in_code_block = !self.in_code_block;
            }

            // 限制缓冲区大小
            if self.buffer.len() > 10 {
                self.buffer.remove(0);
            }

            // 输出字符
            if self.in_code_block {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Yellow),
                    Print(ch),
                    ResetColor
                )?;
            } else {
                execute!(stdout, Print(ch))?;
            }
        }

        stdout.flush()?;
        Ok(())
    }

    /// 完成流式输出
    pub fn finish(&mut self) -> io::Result<()> {
        self.buffer.clear();
        self.in_code_block = false;
        let mut stdout = io::stdout();
        execute!(stdout, ResetColor)?;
        println!();
        stdout.flush()?;
        Ok(())
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.in_code_block = false;
    }
}

impl Default for StreamRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// 流式输出通道
pub struct StreamChannel {
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
}

impl StreamChannel {
    /// 创建新的流式通道
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self { tx, rx }
    }

    /// 获取发送端
    pub fn sender(&self) -> mpsc::Sender<String> {
        self.tx.clone()
    }

    /// 消费接收端
    pub fn into_receiver(self) -> mpsc::Receiver<String> {
        self.rx
    }
}
