use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    // 用户输入
    println!("❯ 帮我找到配置文件并修改端口号\n");

    // AI 响应开始（流式）
    stream_text("⏺ 我来帮你找到配置文件并修改端口号。")?;
    println!("\n");
    thread::sleep(Duration::from_millis(500));

    // 工具调用 1: Glob
    tool_call_with_spinner(
        "Glob",
        "查找配置文件",
        vec!["正在搜索 **/*.toml", "找到 3 个文件"],
    )?;

    // 工具调用 2: Read
    tool_call_with_spinner(
        "Read",
        "读取 config.toml",
        vec!["读取文件内容...", "文件大小: 1.2KB"],
    )?;

    // 工具调用 3: Edit
    tool_call_with_spinner(
        "Edit",
        "修改端口配置",
        vec!["定位目标行...", "应用更改...", "验证语法..."],
    )?;

    // 工具调用 4: Bash
    tool_call_with_spinner(
        "Bash",
        "重启服务",
        vec!["执行: systemctl restart app", "等待服务启动..."],
    )?;

    // 工具使用摘要
    println!();
    stream_text("⏺ 使用了 4 个工具，读取 1 个文件，修改 1 个文件")?;
    println!("\n");
    thread::sleep(Duration::from_millis(300));

    // AI 最终响应（流式）
    println!();
    stream_text("✓ 已将端口从 8080 修改为 3000 并重启服务。")?;
    println!();
    stream_text("\n配置文件位置: config.toml:12")?;
    println!("\n");

    // 压缩状态
    thread::sleep(Duration::from_millis(500));
    println!("✻ Completed in 8.3s\n");

    Ok(())
}

/// 流式文本输出
fn stream_text(text: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    for ch in text.chars() {
        print!("{}", ch);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(30));
    }
    Ok(())
}

/// 模拟工具调用（带 spinner）
fn tool_call_with_spinner(tool: &str, desc: &str, steps: Vec<&str>) -> io::Result<()> {
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    // 调用阶段
    println!();
    print!("⏺ {}({})", tool, desc);
    io::stdout().flush()?;
    thread::sleep(Duration::from_millis(200));

    // 执行阶段 - 带 spinner
    for (idx, step) in steps.iter().enumerate() {
        println!();
        let iterations = if idx == steps.len() - 1 { 8 } else { 5 };
        for _ in 0..iterations {
            for frame in frames {
                print!("\r\x1B[2K⎿  {} {}", frame, step);
                io::stdout().flush()?;
                thread::sleep(Duration::from_millis(80));
            }
        }
    }

    // 完成
    print!("\r\x1B[2K⎿  Done");
    io::stdout().flush()?;
    println!();

    Ok(())
}
