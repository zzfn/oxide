use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("用户: 帮我找到配置文件并修改端口号\n");

    // 模拟工具调用流程
    simulate_tool_call("Glob", "查找配置文件", vec![
        "正在搜索 **/*.toml",
        "找到 3 个文件",
    ])?;

    simulate_tool_call("Read", "读取 config.toml", vec![
        "读取文件内容...",
        "文件大小: 1.2KB",
    ])?;

    simulate_tool_call("Edit", "修改端口配置", vec![
        "定位目标行...",
        "应用更改...",
        "验证语法...",
    ])?;

    simulate_tool_call("Bash", "重启服务", vec![
        "执行: systemctl restart app",
        "等待服务启动...",
    ])?;

    // 流式输出 AI 响应
    println!();
    stream_text("✓ 已将端口从 8080 修改为 3000 并重启服务")?;
    println!("\n");

    Ok(())
}

fn stream_text(text: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    for ch in text.chars() {
        print!("{}", ch);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(30));
    }
    Ok(())
}

fn simulate_tool_call(tool: &str, desc: &str, steps: Vec<&str>) -> io::Result<()> {
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    // 调用阶段
    print!("\r\x1B[2K🔧 调用工具: {} - {}", tool, desc);
    io::stdout().flush()?;
    thread::sleep(Duration::from_millis(300));

    // 执行阶段 - 带 spinner
    for step in steps {
        for _ in 0..5 {
            for frame in frames {
                print!("\r\x1B[2K{} ⚙ 执行工具: {} - {}", frame, tool, step);
                io::stdout().flush()?;
                thread::sleep(Duration::from_millis(80));
            }
        }
    }

    // 完成
    print!("\r\x1B[2K✓ 工具 {} 执行成功", tool);
    io::stdout().flush()?;
    thread::sleep(Duration::from_millis(200));
    println!();

    Ok(())
}
