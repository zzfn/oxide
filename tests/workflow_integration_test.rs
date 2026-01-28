//! 工作流引擎集成测试
//!
//! 测试 PAOR 循环的基本行为（不需要真实 LLM 调用的测试）

use oxide::agent::SubagentManager;
use oxide::agent::workflow::{
    WorkflowOrchestrator, WorkflowExecutor, WorkflowPhase,
    OrchestratorConfig,
};
use std::sync::Arc;

#[tokio::test]
async fn test_workflow_basic_creation() {
    // 创建基本的工作流编排器
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test task: analyze codebase".to_string(),
        subagent_manager,
        None,
    );

    // 验证初始状态
    let state = orchestrator.get_state().await.unwrap();
    assert_eq!(state.iteration, 0);
    assert_eq!(state.phase, WorkflowPhase::Idle);
}

#[tokio::test]
async fn test_workflow_start() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test task".to_string(),
        subagent_manager,
        None,
    );

    // 启动工作流
    orchestrator.start().await.unwrap();

    let state = orchestrator.get_state().await.unwrap();
    assert_eq!(state.phase, WorkflowPhase::Planning);
    assert_eq!(state.iteration, 1);
}

#[tokio::test]
async fn test_workflow_cannot_start_twice() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test task".to_string(),
        subagent_manager,
        None,
    );

    // 第一次启动应该成功
    orchestrator.start().await.unwrap();

    // 第二次启动应该失败
    let result = orchestrator.start().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_workflow_config() {
    let config = OrchestratorConfig {
        max_iterations: 5,
        verbose: true,
        auto_retry: false,
        max_retries: 0,
    };

    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test task".to_string(),
        subagent_manager,
        Some(config),
    );

    let state = orchestrator.get_state().await.unwrap();
    assert_eq!(state.max_iterations, 5);
}

#[tokio::test]
async fn test_observation_collection() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test observation collection".to_string(),
        subagent_manager,
        None,
    );

    // 手动添加一些观察数据
    let collector = orchestrator.get_observation_collector();

    use std::collections::HashMap;
    collector.add_tool_execution(
        "test_tool".to_string(),
        HashMap::new(),
        Some(serde_json::json!({"result": "success"})),
        true,
        None,
        Some(100),
    );

    // 验证观察数据
    let all = collector.get_all();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].source, "test_tool");
    assert!(all[0].success);

    let summary = collector.summarize();
    assert_eq!(summary.total_observations, 1);
    assert_eq!(summary.successful, 1);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.tool_executions, 1);
}

#[tokio::test]
async fn test_workflow_executor_creation() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let executor = WorkflowExecutor::new(
        "Test request".to_string(),
        subagent_manager,
    );

    let state = executor.get_state().await.unwrap();
    assert_eq!(state, WorkflowPhase::Idle);
}

#[tokio::test]
async fn test_workflow_executor_verbose() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let executor = WorkflowExecutor::new(
        "Test request".to_string(),
        subagent_manager,
    ).with_verbose(false);

    // 验证 executor 创建成功
    let state = executor.get_state().await.unwrap();
    assert_eq!(state, WorkflowPhase::Idle);
}

#[test]
fn test_workflow_phase_terminal() {
    assert!(!WorkflowPhase::Idle.is_terminal());
    assert!(!WorkflowPhase::Planning.is_terminal());
    assert!(!WorkflowPhase::Acting.is_terminal());
    assert!(!WorkflowPhase::Observing.is_terminal());
    assert!(!WorkflowPhase::Reflecting.is_terminal());
    assert!(WorkflowPhase::Complete.is_terminal());
    assert!(WorkflowPhase::Failed.is_terminal());
}

#[test]
fn test_workflow_phase_display() {
    assert_eq!(format!("{}", WorkflowPhase::Idle), "Idle");
    assert_eq!(format!("{}", WorkflowPhase::Planning), "Planning");
    assert_eq!(format!("{}", WorkflowPhase::Acting), "Acting");
    assert_eq!(format!("{}", WorkflowPhase::Observing), "Observing");
    assert_eq!(format!("{}", WorkflowPhase::Reflecting), "Reflecting");
    assert_eq!(format!("{}", WorkflowPhase::Complete), "Complete");
    assert_eq!(format!("{}", WorkflowPhase::Failed), "Failed");
}

// 注意：以下测试需要真实的 LLM API 调用，因此被标记为 ignore
// 运行这些测试需要设置有效的 API 密钥
// 使用 `cargo test -- --ignored` 来运行这些测试

#[tokio::test]
#[ignore = "需要真实的 LLM API 调用"]
async fn test_workflow_full_execution() {
    // 这个测试需要真实的 Agent 来执行完整的工作流
    // 由于需要 API 密钥，默认跳过
    println!("此测试需要配置有效的 API 密钥");
}
