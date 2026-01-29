//! Oxide CLI 入口

use anyhow::Result;

mod repl;
mod render;
mod commands;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Oxide - AI 编程助手");
    println!("输入 /help 获取帮助，Ctrl+C 退出");

    // TODO: 初始化并启动 REPL
    Ok(())
}
