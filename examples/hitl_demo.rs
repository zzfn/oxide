//! HITL Gatekeeper 使用示例
//!
//! 展示如何在工具调用中集成 HITL 功能

use oxide::agent::{HitlIntegration, HitlResult, build_operation_context, ToolCallRequest};
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🤖 HITL Gatekeeper 演示\n".bright_cyan().bold());

    // 1. 创建 HITL 集成实例（不需要 API Key！）
    let hitl = HitlIntegration::new()?;
    println!("{} 信任分数: {:.2}\n",
        "✓".green(),
        hitl.trust_score().await
    );

    // 2. 示例 1：安全的只读操作（自动批准）
    println!("{}", "【示例 1】执行 git status（只读操作）".bright_yellow());
    let request = ToolCallRequest {
        tool_name: "shell_execute".to_string(),
        args: serde_json::json!({
            "command": "git status"
        }),
        context: build_operation_context(
            vec!["read_file".to_string()],
            Some("查看代码状态".to_string()),
            true,
            Some("main".to_string()),
        ),
    };

    match hitl.evaluate_and_confirm(request).await? {
        HitlResult::Approved => println!("  {} 已批准\n", "✓".green()),
        HitlResult::Rejected => println!("  {} 已拒绝\n", "✗".red()),
    }

    // 3. 示例 2：删除文件（需要确认）
    println!("{}", "【示例 2】删除文件（需要确认）".bright_yellow());
    let request = ToolCallRequest {
        tool_name: "delete_file".to_string(),
        args: serde_json::json!({
            "file_path": "/tmp/test.txt"
        }),
        context: build_operation_context(
            vec!["read_file".to_string(), "edit_file".to_string()],
            Some("清理临时文件".to_string()),
            false,
            None,
        ),
    };

    match hitl.evaluate_and_confirm(request).await? {
        HitlResult::Approved => {
            println!("  {} 用户确认删除\n", "✓".green());
            hitl.record_success("delete_file /tmp/test.txt".to_string()).await;
        }
        HitlResult::Rejected => {
            println!("  {} 用户取消删除\n", "✗".red());
            hitl.record_rejection().await;
        }
    }

    // 4. 显示更新后的信任分数
    println!("{} 信任分数: {:.2}\n",
        "✓".green(),
        hitl.trust_score().await
    );

    // 5. 示例 3：编辑文件（可能自动批准，取决于信任分数）
    println!("{}", "【示例 3】编辑文件（信任度高时自动批准）".bright_yellow());
    let request = ToolCallRequest {
        tool_name: "edit_file".to_string(),
        args: serde_json::json!({
            "file_path": "src/main.rs",
            "patch": "@@ -1,1 +1,2 @@\n-fn main() {{\n+fn main() {{\n+    println!(\"Hello\");\n"
        }),
        context: build_operation_context(
            vec![
                "read_file".to_string(),
                "edit_file".to_string(),
                "delete_file /tmp/test.txt".to_string(),
            ],
            Some("添加日志输出".to_string()),
            true,
            Some("feature-branch".to_string()),
        ),
    };

    match hitl.evaluate_and_confirm(request).await? {
        HitlResult::Approved => println!("  {} 已批准\n", "✓".green()),
        HitlResult::Rejected => println!("  {} 已拒绝\n", "✗".red()),
    }

    println!("{}", "演示完成！".bright_cyan().bold());
    Ok(())
}
