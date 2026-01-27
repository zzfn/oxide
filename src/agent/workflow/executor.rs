//! 工作流执行器
//!
//! 负责桥接 CLI 和 WorkflowOrchestrator，提供用户友好的接口。

use super::orchestrator::{OrchestratorConfig, WorkflowOrchestrator};
use super::state::WorkflowPhase;
use crate::agent::SubagentManager;
use anyhow::Result;
use std::sync::Arc;

/// 工作流执行器
///
/// 封装 WorkflowOrchestrator，提供更简洁的 API 用于 CLI 集成。
pub struct WorkflowExecutor {
    orchestrator: WorkflowOrchestrator,
    verbose: bool,
}

impl WorkflowExecutor {
    /// 创建新的工作流执行器
    pub fn new(user_request: String, subagent_manager: Arc<SubagentManager>) -> Self {
        let config = OrchestratorConfig {
            max_iterations: 15,
            verbose: false,
            auto_retry: true,
            max_retries: 3,
        };

        let orchestrator = WorkflowOrchestrator::new(user_request, subagent_manager, Some(config));

        Self {
            orchestrator,
            verbose: true, // CLI 模式默认显示进度
        }
    }

    /// 设置是否显示详细日志
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 执行工作流直到完成
    ///
    /// 返回工作流的最终摘要
    pub fn execute(&self) -> Result<String> {
        // 启动工作流
        self.orchestrator.start()?;

        if self.verbose {
            println!("🚀 启动 PAOR 工作流...\n");
        }

        // 执行循环
        let mut iteration = 0;
        loop {
            iteration += 1;

            // 获取当前状态
            let state = self.orchestrator.get_state()?;

            if self.verbose {
                println!("🔄 迭代 {}/{} | 阶段: {}", iteration, state.max_iterations, state.phase);
            }

            // 执行一次迭代
            let should_continue = self.orchestrator.execute_iteration()?;

            // 检查是否应该继续
            if !should_continue {
                break;
            }

            // 防止无限循环
            if iteration >= 100 {
                if self.verbose {
                    println!("⚠️  达到最大迭代次数限制");
                }
                break;
            }
        }

        // 生成最终摘要
        let summary = self.orchestrator.generate_summary()?;

        if self.verbose {
            println!("\n✅ 工作流执行完成\n");
            println!("{}", summary);
        }

        Ok(summary)
    }

    /// 获取当前状态
    pub fn get_state(&self) -> Result<WorkflowPhase> {
        Ok(self.orchestrator.get_state()?.phase)
    }

    /// 获取工作流进度百分比
    pub fn get_progress(&self) -> Result<f32> {
        let state = self.orchestrator.get_state()?;
        let progress = (state.iteration as f32 / state.max_iterations as f32) * 100.0;
        Ok(progress.min(100.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let executor = WorkflowExecutor::new("Test request".to_string(), subagent_manager);

        let state = executor.get_state().unwrap();
        assert_eq!(state, WorkflowPhase::Idle);
    }

    #[test]
    fn test_executor_verbose() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let executor = WorkflowExecutor::new("Test request".to_string(), subagent_manager)
            .with_verbose(false);

        assert!(!executor.verbose);
    }
}
