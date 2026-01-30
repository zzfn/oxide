//! Bash 执行工具示例
//!
//! 演示 Bash, TaskOutput, TaskStop 工具的使用

use oxide_tools::{BashTool, TaskOutputTool, Tool};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 测试 Bash 执行工具\n");

    let working_dir = env::current_dir()?;
    println!("📁 工作目录: {}\n", working_dir.display());

    // 创建 Bash 工具
    let bash_tool = BashTool::new(working_dir.clone());
    let task_manager = bash_tool.task_manager();

    // 测试 1: 简单命令执行
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⚡ 测试 1: 简单命令执行");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = bash_tool
        .execute(json!({
            "command": "echo 'Hello from Bash!'"
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试 2: 列出文件
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📂 测试 2: 列出当前目录文件");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = bash_tool
        .execute(json!({
            "command": "ls -la | head -10",
            "description": "列出文件"
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试 3: 管道命令
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔧 测试 3: 管道命令");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = bash_tool
        .execute(json!({
            "command": "echo 'apple\nbanana\ncherry' | grep 'a' | wc -l"
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试 4: 超时控制
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⏱️  测试 4: 超时控制");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = bash_tool
        .execute(json!({
            "command": "sleep 3",
            "timeout": 500
        }))
        .await?;

    if result.is_error {
        println!("✓ 正确捕获超时: {}\n", result.content);
    }

    // 测试 5: 后台任务
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 测试 5: 后台任务执行");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = bash_tool
        .execute(json!({
            "command": "for i in 1 2 3; do echo \"Step $i\"; sleep 0.5; done",
            "run_in_background": true
        }))
        .await?;

    println!("{}\n", result.content);

    // 提取任务 ID
    let task_id = result
        .content
        .lines()
        .find(|line| line.contains("任务 ID:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim())
        .unwrap_or("");

    if !task_id.is_empty() {
        // 测试 6: 查看后台任务输出（非阻塞）
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("👀 测试 6: 查看后台任务状态（非阻塞）");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let task_output_tool = TaskOutputTool::new(task_manager.clone());
        let result = task_output_tool
            .execute(json!({
                "task_id": task_id,
                "block": false
            }))
            .await?;

        println!("{}\n", result.content);

        // 测试 7: 等待后台任务完成
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("⏳ 测试 7: 等待后台任务完成");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let result = task_output_tool
            .execute(json!({
                "task_id": task_id,
                "block": true,
                "timeout": 5000
            }))
            .await?;

        println!("{}\n", result.content);
    }

    // 测试 8: 错误处理
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("❌ 测试 8: 错误命令处理");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = bash_tool
        .execute(json!({
            "command": "nonexistent_command_xyz"
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试 9: 工作目录验证
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📍 测试 9: 工作目录验证");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = bash_tool
        .execute(json!({
            "command": "pwd"
        }))
        .await?;

    println!("{}\n", result.content);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎉 所有测试完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
