//! 工作流引擎使用示例
//!
//! 演示如何使用 WorkflowOrchestrator 和 WorkflowExecutor
//!
//! 注意：完整的工作流执行需要配置有效的 LLM API 密钥。
//! 此示例仅演示工作流的创建和基本状态管理。

use oxide::agent::workflow::{WorkflowOrchestrator, WorkflowPhase, OrchestratorConfig};
use oxide::agent::SubagentManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 PAOR 工作流引擎示例\n");

    // 1. 创建子 agent 管理器
    let subagent_manager = Arc::new(SubagentManager::new());

    // 2. 创建自定义配置
    let config = OrchestratorConfig {
        max_iterations: 10,
        verbose: true,
        auto_retry: true,
        max_retries: 3,
    };

    // 3. 创建工作流编排器
    let orchestrator = WorkflowOrchestrator::new(
        "分析代码库中的所有 TODO 注释并创建任务列表".to_string(),
        subagent_manager,
        Some(config),
    );

    // 4. 获取初始状态
    let state = orchestrator.get_state().await?;
    println!("📋 用户请求:");
    println!("   {}\n", state.user_request);
    println!("📊 初始状态:");
    println!("   阶段: {}", state.phase);
    println!("   迭代: {}", state.iteration);
    println!("   最大迭代: {}", state.max_iterations);
    println!();

    // 5. 启动工作流（进入 Planning 阶段）
    orchestrator.start().await?;
    println!("✅ 工作流已启动\n");

    let state = orchestrator.get_state().await?;
    println!("📊 启动后状态:");
    println!("   阶段: {}", state.phase);
    println!("   迭代: {}", state.iteration);
    println!();

    // 6. 演示观察数据收集
    println!("👁️  演示观察数据收集:");
    let collector = orchestrator.get_observation_collector();

    use std::collections::HashMap;
    collector.add_tool_execution(
        "read_file".to_string(),
        HashMap::from([("path".to_string(), serde_json::json!("src/main.rs"))]),
        Some(serde_json::json!({"content": "// TODO: implement feature"})),
        true,
        None,
        Some(50),
    );

    collector.add_tool_execution(
        "grep_search".to_string(),
        HashMap::from([("pattern".to_string(), serde_json::json!("TODO"))]),
        Some(serde_json::json!({"matches": 5})),
        true,
        None,
        Some(120),
    );

    let summary = collector.summarize();
    println!("   总观察数: {}", summary.total_observations);
    println!("   成功: {}", summary.successful);
    println!("   失败: {}", summary.failed);
    println!("   工具执行: {}", summary.tool_executions);
    println!();

    // 7. 演示工作流阶段
    println!("📋 PAOR 工作流阶段说明:");
    let phases = [
        (WorkflowPhase::Idle, "空闲 - 等待启动"),
        (WorkflowPhase::Planning, "规划 - 分析任务并制定计划"),
        (WorkflowPhase::Acting, "执行 - 执行计划中的任务"),
        (WorkflowPhase::Observing, "观察 - 收集执行结果"),
        (WorkflowPhase::Reflecting, "反思 - 评估进展并决定下一步"),
        (WorkflowPhase::Complete, "完成 - 目标已达成"),
        (WorkflowPhase::Failed, "失败 - 遇到不可恢复错误"),
    ];

    for (phase, desc) in phases {
        let terminal = if phase.is_terminal() { " [终止状态]" } else { "" };
        println!("   {} - {}{}", phase, desc, terminal);
    }
    println!();

    // 8. 生成摘要
    println!("📝 工作流摘要:\n");
    let summary = orchestrator.generate_summary().await?;
    println!("{}", summary);

    println!("\n💡 提示: 要执行完整的工作流，需要配置有效的 LLM API 密钥。");
    println!("   使用 WorkflowExecutor.execute(&agent) 方法执行完整的 PAOR 循环。");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_example() -> anyhow::Result<()> {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator = WorkflowOrchestrator::new(
            "Test request".to_string(),
            subagent_manager,
            None,
        );

        // 验证初始状态
        let state = orchestrator.get_state().await?;
        assert_eq!(state.phase, WorkflowPhase::Idle);

        // 启动工作流
        orchestrator.start().await?;

        let state = orchestrator.get_state().await?;
        assert_eq!(state.phase, WorkflowPhase::Planning);
        assert_eq!(state.iteration, 1);

        Ok(())
    }
}
