//! 流式输出
//!
//! 处理 AI 响应的流式输出。

use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

/// 流式渲染器
pub struct StreamRenderer {
    buffer: String,
    in_code_block: bool,
    delay_ms: u64,
}

impl StreamRenderer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_code_block: false,
            delay_ms: 30,
        }
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn stream_text(&mut self, text: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        for ch in text.chars() {
            print!("{}", ch);
            stdout.flush()?;
            thread::sleep(Duration::from_millis(self.delay_ms));
        }
        Ok(())
    }

    pub fn write(&mut self, text: &str) -> io::Result<()> {
        let mut stdout = io::stdout();

        for ch in text.chars() {
            self.buffer.push(ch);
            if self.buffer.ends_with("```") {
                self.in_code_block = !self.in_code_block;
            }

            if self.buffer.len() > 10 {
                self.buffer.remove(0);
            }

            if self.in_code_block {
                print!("\x1b[33m{}\x1b[0m", ch);
            } else {
                print!("{}", ch);
            }
        }

        stdout.flush()?;
        Ok(())
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.buffer.clear();
        self.in_code_block = false;
        println!();
        io::stdout().flush()?;
        Ok(())
    }

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
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self { tx, rx }
    }

    pub fn sender(&self) -> mpsc::Sender<String> {
        self.tx.clone()
    }

    pub fn into_receiver(self) -> mpsc::Receiver<String> {
        self.rx
    }
}
