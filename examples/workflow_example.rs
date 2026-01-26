//! 工作流引擎使用示例
//!
//! 演示如何使用 WorkflowOrchestrator 执行自主工作流

use oxide::agent::{SubagentManager, WorkflowOrchestrator};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("🚀 Workflow Engine Example\n");
    
    // 1. 创建子 agent 管理器
    let subagent_manager = Arc::new(SubagentManager::new());
    
    // 2. 创建工作流编排器
    let orchestrator = WorkflowOrchestrator::new(
        "Find all TODO comments in the codebase and create a task list".to_string(),
        subagent_manager,
        None, // 使用默认配置
    );
    
    println!("📋 User Request:");
    println!("   {}\n", orchestrator.get_state()?.user_request);
    
    // 3. 启动工作流
    orchestrator.start()?;
    println!("✅ Workflow started\n");
    
    // 4. 执行 PAOR 循环
    let mut iteration = 0;
    let max_display = 5; // 只显示前几次迭代
    
    loop {
        iteration += 1;
        
        // 获取当前状态
        let state = orchestrator.get_state()?;
        
        if iteration <= max_display {
            println!("🔄 Iteration {}: Phase = {}", iteration, state.phase);
        }
        
        // 执行一次迭代
        let should_continue = orchestrator.execute_iteration()?;
        
        // 检查是否应该继续
        if !should_continue {
            break;
        }
        
        // 防止无限循环（示例中限制最多显示几次）
        if iteration >= 100 {
            println!("⚠️  Reached maximum demo iterations");
            break;
        }
    }
    
    println!();
    
    // 5. 获取最终状态
    let final_state = orchestrator.get_state()?;
    println!("📊 Final State:");
    println!("   Phase: {}", final_state.phase);
    println!("   Iterations: {}", final_state.iteration);
    println!("   Duration: {}ms", final_state.elapsed_ms());
    
    if let Some(reason) = &final_state.failure_reason {
        println!("   Failure Reason: {}", reason);
    }
    
    println!();
    
    // 6. 获取观察数据摘要
    let obs_summary = orchestrator.get_observation_collector().summarize();
    println!("👁️  Observations:");
    println!("   Total: {}", obs_summary.total_observations);
    println!("   Successful: {}", obs_summary.successful);
    println!("   Failed: {}", obs_summary.failed);
    println!("   Tool Executions: {}", obs_summary.tool_executions);
    println!("   Subagent Calls: {}", obs_summary.subagent_calls);
    
    println!();
    
    // 7. 生成完整摘要
    println!("📝 Full Summary:\n");
    let summary = orchestrator.generate_summary()?;
    println!("{}", summary);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_example() -> anyhow::Result<()> {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator = WorkflowOrchestrator::new(
            "Test request".to_string(),
            subagent_manager,
            None,
        );
        
        orchestrator.start()?;
        
        // 执行几次迭代
        for _ in 0..3 {
            if !orchestrator.execute_iteration()? {
                break;
            }
        }
        
        let state = orchestrator.get_state()?;
        assert!(state.iteration > 0);
        
        Ok(())
    }
}
