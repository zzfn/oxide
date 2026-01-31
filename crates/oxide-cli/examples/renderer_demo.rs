use oxide_cli::render::Renderer;
use std::io;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut renderer = Renderer::new();

    // 显示欢迎信息
    renderer.welcome();

    // 模拟用户输入
    println!("❯ 帮我找到配置文件并修改端口号\n");

    // AI 响应开始（流式）
    renderer.stream_mut().stream_text("⏺ 我来帮你找到配置文件并修改端口号。")?;
    println!("\n");
    thread::sleep(Duration::from_millis(500));

    // 工具调用 1: Glob
    simulate_tool_call(&mut renderer, "Glob", "**/*.toml", vec![
        "正在搜索 **/*.toml",
        "找到 3 个文件",
    ], "找到 3 个文件")?;

    // 工具调用 2: Read
    simulate_tool_call(&mut renderer, "Read", "config.toml", vec![
        "读取文件内容...",
        "文件大小: 1.2KB",
    ], "1.2KB")?;

    // 工具调用 3: Edit
    simulate_tool_call(&mut renderer, "Edit", "config.toml:12", vec![
        "定位目标行...",
        "应用更改...",
        "验证语法...",
    ], "修改 1 行")?;

    // 工具调用 4: Bash
    simulate_tool_call(&mut renderer, "Bash", "systemctl restart app", vec![
        "执行: systemctl restart app",
        "等待服务启动...",
    ], "退出码 0")?;

    // 工具使用摘要
    println!();
    renderer.stream_mut().stream_text("⏺ 使用了 4 个工具，读取 1 个文件，修改 1 个文件")?;
    println!("\n");
    thread::sleep(Duration::from_millis(300));

    // AI 最终响应（流式）
    println!();
    renderer.stream_mut().stream_text("✓ 已将端口从 8080 修改为 3000 并重启服务。")?;
    println!();
    renderer.stream_mut().stream_text("\n配置文件位置: config.toml:12")?;
    println!("\n");

    // 压缩状态
    thread::sleep(Duration::from_millis(500));
    println!("✻ Completed in 8.3s\n");

    Ok(())
}

/// 模拟工具调用（使用渲染器）
fn simulate_tool_call(
    renderer: &mut Renderer,
    tool: &str,
    param: &str,
    steps: Vec<&str>,
    summary: &str,
) -> io::Result<()> {
    // 开始工具调用
    renderer.tool_status_mut().start_tool(tool, param)?;

    // 执行各个步骤
    for step in steps {
        for _ in 0..8 {
            renderer.tool_status_mut().update_executing(step)?;
            thread::sleep(Duration::from_millis(80));
        }
    }

    // 完成工具调用（带统计信息）
    renderer.tool_status_mut().finish_tool(Some(summary))?;

    Ok(())
}
