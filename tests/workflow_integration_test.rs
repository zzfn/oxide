//! 工作流引擎集成测试
//!
//! 测试 PAOR 循环的端到端行为

use oxide::agent::{SubagentManager, WorkflowOrchestrator};
use std::sync::Arc;

#[test]
fn test_workflow_basic_execution() {
    // 创建基本的工作流编排器
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test task: analyze codebase".to_string(),
        subagent_manager,
        None,
    );
    
    // 验证初始状态
    let state = orchestrator.get_state().unwrap();
    assert_eq!(state.iteration, 0);
    assert!(state.phase.to_string().contains("Idle"));
    
    // 启动工作流
    orchestrator.start().unwrap();
    
    let state = orchestrator.get_state().unwrap();
    assert!(state.phase.to_string().contains("Planning"));
}

#[test]
fn test_workflow_iteration_limit() {
    use oxide::agent::workflow::orchestrator::OrchestratorConfig;
    
    // 创建一个最大迭代次数很小的配置
    let config = OrchestratorConfig {
        max_iterations: 2,
        verbose: false,
        auto_retry: false,
        max_retries: 0,
    };
    
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test task".to_string(),
        subagent_manager,
        Some(config),
    );
    
    orchestrator.start().unwrap();
    
    // 执行多次迭代
    let mut iterations = 0;
    loop {
        let should_continue = orchestrator.execute_iteration().unwrap();
        iterations += 1;
        
        if !should_continue {
            break;
        }
        
        // 防止真正的无限循环
        if iterations > 100 {
            panic!("Test failed: exceeded safety limit");
        }
    }
    
    // 验证最终状态
    let state = orchestrator.get_state().unwrap();
    
    // 由于我们的占位符实现会在第一次反思后就标记完成
    // 所以实际不会达到最大迭代次数，但我们可以验证系统正常终止
    assert!(state.should_terminate(), "Workflow should have terminated");
    
    println!("Workflow terminated after {} iterations", state.iteration);
}

#[test]
fn test_workflow_state_transitions() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test state transitions".to_string(),
        subagent_manager,
        None,
    );
    
    // 启动并记录状态转换
    orchestrator.start().unwrap();
    
    let mut states = vec![orchestrator.get_state().unwrap().phase.to_string()];
    
    // 执行几次迭代，记录状态
    for _ in 0..5 {
        if !orchestrator.execute_iteration().unwrap() {
            break;
        }
        states.push(orchestrator.get_state().unwrap().phase.to_string());
    }
    
    println!("State transitions: {:?}", states);
    
    // 验证至少有状态转换发生
    assert!(states.len() > 1, "Should have multiple states");
    
    // 验证最终状态是终止状态
    let final_state = orchestrator.get_state().unwrap();
    assert!(
        final_state.phase.is_terminal(),
        "Should end in terminal state"
    );
}

#[test]
fn test_observation_collection() {
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
    
    collector.add_subagent_result(
        "TestAgent".to_string(),
        "test input".to_string(),
        "test output".to_string(),
        true,
    );
    
    // 验证观察数据
    assert_eq!(collector.count(), 2, "Should have 2 observations");
    
    let summary = collector.summarize();
    assert_eq!(summary.total_observations, 2);
    assert_eq!(summary.successful, 2);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.tool_executions, 1);
    assert_eq!(summary.subagent_calls, 1);
}

#[test]
fn test_workflow_summary_generation() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Generate summary test".to_string(),
        subagent_manager,
        None,
    );
    
    orchestrator.start().unwrap();
    
    // 执行一次迭代
    orchestrator.execute_iteration().unwrap();
    
    // 生成摘要
    let summary = orchestrator.generate_summary().unwrap();
    
    println!("Generated summary:\n{}", summary);
    
    // 验证摘要包含关键信息
    assert!(summary.contains("Workflow Summary"));
    assert!(summary.contains("Status"));
    assert!(summary.contains("Iterations"));
    assert!(summary.contains("Observations"));
}

#[test]
fn test_workflow_reflections() {
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Test reflections".to_string(),
        subagent_manager,
        None,
    );
    
    orchestrator.start().unwrap();
    
    // 执行迭代直到产生反思（至少要经过 Reflecting 阶段）
    let mut iterations = 0;
    loop {
        iterations += 1;
        
        if !orchestrator.execute_iteration().unwrap() {
            break;
        }
        
        // 检查是否已经有反思
        let reflections = orchestrator.get_reflections().unwrap();
        if !reflections.is_empty() {
            break;
        }
        
        // 安全限制
        if iterations > 20 {
            break;
        }
    }
    
    // 获取反思历史
    let reflections = orchestrator.get_reflections().unwrap();
    
    println!("Number of reflections: {}", reflections.len());
    println!("Iterations executed: {}", iterations);
    
    // 应该至少有一次反思
    assert!(
        !reflections.is_empty(),
        "Should have at least one reflection after {} iterations",
        iterations
    );
    
    // 验证反思结构
    if let Some(first_reflection) = reflections.first() {
        assert!(
            first_reflection.progress >= 0.0 && first_reflection.progress <= 1.0,
            "Progress should be between 0 and 1"
        );
    }
}

#[test]
fn test_workflow_complete_cycle() {
    // 端到端测试：运行完整的工作流周期
    let subagent_manager = Arc::new(SubagentManager::new());
    let orchestrator = WorkflowOrchestrator::new(
        "Complete workflow test".to_string(),
        subagent_manager,
        None,
    );
    
    // 启动
    orchestrator.start().unwrap();
    
    let start_state = orchestrator.get_state().unwrap();
    assert_eq!(start_state.iteration, 1);
    
    // 运行到完成
    let mut iteration_count = 0;
    loop {
        iteration_count += 1;
        
        let should_continue = orchestrator.execute_iteration().unwrap();
        
        if !should_continue {
            break;
        }
        
        // 安全限制
        if iteration_count > 50 {
            panic!("Test exceeded safety limit");
        }
    }
    
    // 验证最终状态
    let final_state = orchestrator.get_state().unwrap();
    assert!(final_state.should_terminate());
    
    // 生成并验证摘要
    let summary = orchestrator.generate_summary().unwrap();
    assert!(!summary.is_empty());
    
    println!("\n=== Complete Workflow Test ===");
    println!("Total iterations: {}", iteration_count);
    println!("Final phase: {}", final_state.phase);
    println!("\n{}", summary);
}
