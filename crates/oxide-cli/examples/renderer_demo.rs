//! 使用 indicatif MultiProgress 的渲染演示
//!
//! 底部状态行始终保持在最下面，显示运行时间

use oxide_cli::render::Renderer;
use std::io;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut renderer = Renderer::new();

    // 显示欢迎信息
    renderer.welcome();

    // 模拟用户输入
    renderer.println("❯ 帮我找到配置文件并修改端口号\n")?;

    // 开始状态行（底部始终显示）
    renderer.statusline_mut().start("Thinking");

    // AI 响应开始
    thread::sleep(Duration::from_millis(500));
    renderer.println(format!(
        "\n{} 我来帮你找到配置文件并修改端口号。\n",
        "⏺"
    ))?;

    // 工具调用 1: Glob
    simulate_tool_call(
        &mut renderer,
        "Glob",
        "**/*.toml",
        &["正在搜索 **/*.toml", "找到 3 个文件"],
        "找到 3 个文件",
    )?;

    // 工具调用 2: Read
    simulate_tool_call(
        &mut renderer,
        "Read",
        "config.toml",
        &["读取文件内容...", "文件大小: 1.2KB"],
        "1.2KB",
    )?;

    // 工具调用 3: Edit
    simulate_tool_call(
        &mut renderer,
        "Edit",
        "config.toml:12",
        &["定位目标行...", "应用更改...", "验证语法..."],
        "修改 1 行",
    )?;

    // 工具调用 4: Bash
    simulate_tool_call(
        &mut renderer,
        "Bash",
        "systemctl restart app",
        &["执行: systemctl restart app", "等待服务启动..."],
        "退出码 0",
    )?;

    // 工具使用摘要
    renderer.statusline_mut().update("Summarizing", 1200);
    thread::sleep(Duration::from_millis(500));

    renderer.println(format!(
        "\n{} 使用了 4 个工具，读取 1 个文件，修改 1 个文件\n",
        "⏺"
    ))?;

    // AI 最终响应
    renderer.statusline_mut().update("Responding", 1350);
    thread::sleep(Duration::from_millis(300));

    renderer.success("已将端口从 8080 修改为 3000 并重启服务。");
    renderer.println("配置文件位置: config.toml:12\n")?;

    // 完成
    thread::sleep(Duration::from_millis(500));
    renderer.statusline_mut().finish();

    println!();
    Ok(())
}

/// 模拟工具调用
fn simulate_tool_call(
    renderer: &mut Renderer,
    tool: &str,
    param: &str,
    steps: &[&str],
    summary: &str,
) -> io::Result<()> {
    // 获取底部状态栏，用于 insert_before
    let status_bar = renderer.statusline_mut().bar().cloned();

    // 开始工具调用（在状态行上方）
    if let Some(bar) = &status_bar {
        renderer
            .tool_status_mut()
            .start_tool_before(tool, param, bar)?;
    } else {
        renderer.tool_status_mut().start_tool(tool, param)?;
    }

    // 执行各个步骤
    for step in steps {
        renderer.tool_status_mut().update_executing(step)?;
        // 更新底部状态行的 token 计数
        if let Some(start) = renderer.statusline_mut().start_time() {
            let tokens = (start.elapsed().as_millis() / 3) as usize;
            renderer.statusline_mut().update("Processing", tokens);
        }
        thread::sleep(Duration::from_millis(400));
    }

    // 完成工具调用
    renderer.tool_status_mut().finish_tool(Some(summary))?;

    Ok(())
}
