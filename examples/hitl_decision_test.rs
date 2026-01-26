//! HITL 决策节点测试
//!
//! 可视化展示 HITL Gatekeeper 的决策流程

use oxide::agent::{HitlGatekeeper, HitlConfig, ToolCallRequest, build_operation_context, HitlDecision, WarningLevel};
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "═".repeat(80).bright_black());
    println!("{}", "HITL Gatekeeper 决策节点可视化测试".bright_cyan().bold());
    println!("{}", "═".repeat(80).bright_black());
    println!();

    // 创建 Gatekeeper
    let config = HitlConfig::default();
    let gatekeeper = HitlGatekeeper::new(config)?;

    // 测试场景列表
    let test_cases = vec![
        // 场景 1: 只读操作（应该自动批准）
        TestCase {
            name: "读取文件（只读操作）",
            tool_name: "read_file",
            args: serde_json::json!({ "file_path": "src/main.rs" }),
            expected: "自动批准",
            description: "只读操作无任何风险，应该直接通过",
        },

        // 场景 2: 安全命令（应该自动批准）
        TestCase {
            name: "执行 git status（安全命令）",
            tool_name: "shell_execute",
            args: serde_json::json!({ "command": "git status" }),
            expected: "自动批准",
            description: "已知的安全只读命令",
        },

        // 场景 3: 搜索代码（应该自动批准）
        TestCase {
            name: "搜索代码（grep）",
            tool_name: "grep_search",
            args: serde_json::json!({
                "pattern": "fn main",
                "path": "."
            }),
            expected: "自动批准",
            description: "代码搜索是只读操作",
        },

        // 场景 4: 删除文件（应该需要确认）
        TestCase {
            name: "删除文件",
            tool_name: "delete_file",
            args: serde_json::json!({ "file_path": "/tmp/test.txt" }),
            expected: "需要确认",
            description: "删除操作不可逆，需要用户确认",
        },

        // 场景 5: 执行普通命令（应该需要确认）
        TestCase {
            name: "执行 npm install",
            tool_name: "shell_execute",
            args: serde_json::json!({ "command": "npm install" }),
            expected: "需要确认",
            description: "修改 node_modules，需要确认",
        },

        // 场景 6: 危险命令（应该拒绝）
        TestCase {
            name: "执行 rm -rf /（危险命令）",
            tool_name: "shell_execute",
            args: serde_json::json!({ "command": "rm -rf /" }),
            expected: "拒绝执行",
            description: "极其危险的命令，应该直接拒绝",
        },

        // 场景 7: 编辑文件（应该需要确认）
        TestCase {
            name: "编辑文件",
            tool_name: "edit_file",
            args: serde_json::json!({
                "file_path": "src/main.rs",
                "patch": "@@ -1,1 +1,2 @@\n-old\n+new"
            }),
            expected: "需要确认",
            description: "修改文件内容",
        },

        // 场景 8: 写入文件（应该需要确认）
        TestCase {
            name: "写入新文件",
            tool_name: "write_file",
            args: serde_json::json!({
                "file_path": "new_file.rs",
                "content": "fn main() {}"
            }),
            expected: "需要确认",
            description: "创建/覆盖文件",
        },

        // 场景 9: fork bomb（应该拒绝）
        TestCase {
            name: "Fork bomb（恶意命令）",
            tool_name: "shell_execute",
            args: serde_json::json!({ "command": ":(){:|:&};:" }),
            expected: "拒绝执行",
            description: "fork bomb 会耗尽系统资源",
        },

        // 场景 10: 未知工具（默认执行）
        TestCase {
            name: "未知工具",
            tool_name: "unknown_tool",
            args: serde_json::json!({ "param": "value" }),
            expected: "默认执行",
            description: "未知的工具，采用安全策略默认执行",
        },
    ];

    // 显示信任分数
    println!("{} 当前信任分数: {:.2}\n",
        "📊".bright_cyan(),
        gatekeeper.trust_score().await
    );

    // 运行所有测试
    let mut passed: i32 = 0;
    let mut current: i32 = 0;

    for (index, test) in test_cases.iter().enumerate() {
        current += 1;

        println!("{}", "─".repeat(80).bright_black());
        println!("{} {}", format!("测试 {}/{}", index + 1, test_cases.len()).bright_yellow(), test.name.bright_white());
        println!("{}", "─".repeat(80).bright_black());

        println!();
        println!("  工具: {}", test.tool_name.bright_cyan());
        println!("  参数: {}", serde_json::to_string_pretty(&test.args).unwrap_or_default().dimmed());
        println!();
        println!("  描述: {}", test.description.dimmed());
        println!("  预期: {}", test.expected.bright_green());
        println!();

        // 构建请求
        let request = ToolCallRequest {
            tool_name: test.tool_name.to_string(),
            args: test.args.clone(),
            context: build_operation_context(
                vec!["read_file".to_string()],
                Some("测试 HITL 决策".to_string()),
                true,
                Some("main".to_string()),
            ),
        };

        // 执行决策
        let decision = gatekeeper.evaluate_tool_call(request).await?;

        // 显示决策结果
        match decision {
            HitlDecision::ExecuteDirectly { reason } => {
                println!("  {} 决策: {}", "✅".bright_green(), "自动执行".bright_green().bold());
                println!("  {} 理由: {}", "📝".dimmed(), reason.dimmed());

                if test.expected == "自动批准" {
                    println!("  {} 结果: {}", "✓".bright_green(), "通过".bright_green());
                    passed += 1;
                } else {
                    println!("  {} 结果: {}", "✗".bright_red(), "失败".bright_red());
                    println!("  {} 预期: {}", "⚠️".bright_yellow(), test.expected);
                }
            }

            HitlDecision::RequireConfirmation { reason, warning_level } => {
                let icon = match warning_level {
                    WarningLevel::Info => "ℹ️",
                    WarningLevel::Low => "⚠️",
                    WarningLevel::Medium => "⚠️",
                    WarningLevel::High => "🚨",
                    WarningLevel::Critical => "🔴",
                };

                println!("  {} 决策: {}", "⏸️".bright_yellow(), "需要确认".bright_yellow().bold());
                println!("  {} 级别: {} {:?}", "🔔".dimmed(), icon, warning_level);
                println!("  {} 理由: {}", "📝".dimmed(), reason.dimmed());

                if test.expected == "需要确认" {
                    println!("  {} 结果: {}", "✓".bright_green(), "通过".bright_green());
                    passed += 1;
                } else {
                    println!("  {} 结果: {}", "✗".bright_red(), "失败".bright_red());
                    println!("  {} 预期: {}", "⚠️".bright_yellow(), test.expected);
                }
            }

            HitlDecision::Reject { reason, suggestion } => {
                println!("  {} 决策: {}", "🛑".bright_red(), "拒绝执行".bright_red().bold());
                println!("  {} 理由: {}", "📝".dimmed(), reason.dimmed());
                if let Some(suggestion) = suggestion {
                    println!("  {} 建议: {}", "💡".bright_cyan(), suggestion.bright_cyan());
                }

                if test.expected == "拒绝执行" {
                    println!("  {} 结果: {}", "✓".bright_green(), "通过".bright_green());
                    passed += 1;
                } else {
                    println!("  {} 结果: {}", "✗".bright_red(), "失败".bright_red());
                    println!("  {} 预期: {}", "⚠️".bright_yellow(), test.expected);
                }
            }

            HitlDecision::RequireChoice { question, options, .. } => {
                println!("  {} 决策: {}", "❓".bright_blue(), "需要选择".bright_blue().bold());
                println!("  {} 问题: {}", "📝".dimmed(), question.dimmed());
                println!("  {} 选项:", "📋".dimmed());
                for (i, option) in options.iter().enumerate() {
                    println!("    {}. {} - {}", i + 1, option.label, option.description);
                }
            }
        }

        println!();
    }

    // 显示总结
    println!("{}", "═".repeat(80).bright_black());
    println!("{}", "测试总结".bright_cyan().bold());
    println!("{}", "═".repeat(80).bright_black());
    println!();
    println!("  通过: {}/{}", passed.to_string().bright_green(), current);
    println!("  失败: {}/{}", (current - passed).to_string().bright_red(), current);
    let success_rate = passed as f32 / current as f32 * 100.0;
    println!("  成功率: {:.1}%", success_rate);
    println!();

    // 显示决策流程图
    println!("{}", "═".repeat(80).bright_black());
    println!("{}", "决策流程图".bright_cyan().bold());
    println!("{}", "═".repeat(80).bright_black());
    println!();
    println!("{}", r#"
工具调用请求
    ↓
检查 HITL 是否启用？
    ├─ 否 → 执行直接 ✅
    └─ 是 ↓
快速路径检查（只读操作）
    ├─ read_file/glob/grep → 执行直接 ✅
    ├─ git status/ls/pwd → 执行直接 ✅
    └─ 其他 ↓
信任分数检查 (≥ 0.8)
    ├─ 是 + 低风险工具 → 执行直接 ✅
    └─ 否 ↓
规则引擎判断
    ├─ delete_file → 需要确认 ⚠️
    ├─ shell_execute
    │   ├─ 危险命令 → 拒绝 🛑
    │   └─ 普通命令 → 需要确认 ⚠️
    ├─ write_file/edit_file → 需要确认 ⚠️
    └─ 未知工具 → 执行直接 ✅
    "#.dimmed());
    println!();

    Ok(())
}

/// 测试用例
struct TestCase {
    name: &'static str,
    tool_name: &'static str,
    args: serde_json::Value,
    expected: &'static str,
    description: &'static str,
}
