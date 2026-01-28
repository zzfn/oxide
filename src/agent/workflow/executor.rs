//! 工作流执行器
//!
//! 负责桥接 CLI 和 WorkflowOrchestrator，提供用户友好的接口。

use super::orchestrator::{OrchestratorConfig, WorkflowOrchestrator};
use super::state::WorkflowPhase;
use crate::agent::builder::AgentEnum;
use crate::agent::SubagentManager;
use anyhow::Result;
use std::sync::Arc;

/// 进度回调类型
pub type ProgressCallback = Box<dyn Fn(WorkflowProgress) + Send + Sync>;

/// 工作流进度信息
#[derive(Debug, Clone)]
pub struct WorkflowProgress {
    /// 当前阶段
    pub phase: WorkflowPhase,
    /// 当前迭代
    pub iteration: u32,
    /// 最大迭代
    pub max_iterations: u32,
    /// 进度百分比 (0-100)
    pub percentage: f32,
    /// 状态消息
    pub message: String,
}

impl WorkflowProgress {
    /// 创建新的进度信息
    pub fn new(phase: WorkflowPhase, iteration: u32, max_iterations: u32, message: String) -> Self {
        let percentage = (iteration as f32 / max_iterations as f32) * 100.0;
        Self {
            phase,
            iteration,
            max_iterations,
            percentage: percentage.min(100.0),
            message,
        }
    }
}

/// 工作流执行器
///
/// 封装 WorkflowOrchestrator，提供更简洁的 API 用于 CLI 集成。
pub struct WorkflowExecutor {
    orchestrator: WorkflowOrchestrator,
    verbose: bool,
    progress_callback: Option<ProgressCallback>,
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
            progress_callback: None,
        }
    }

    /// 使用自定义配置创建执行器
    pub fn with_config(
        user_request: String,
        subagent_manager: Arc<SubagentManager>,
        config: OrchestratorConfig,
    ) -> Self {
        let orchestrator = WorkflowOrchestrator::new(user_request, subagent_manager, Some(config));

        Self {
            orchestrator,
            verbose: true,
            progress_callback: None,
        }
    }

    /// 设置是否显示详细日志
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 设置进度回调
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// 异步执行工作流直到完成
    ///
    /// 返回工作流的最终摘要
    pub async fn execute(&self, agent: &AgentEnum) -> Result<WorkflowResult> {
        // 启动工作流
        self.orchestrator.start().await?;

        if self.verbose {
            println!("🚀 启动 PAOR 工作流...\n");
        }

        // 执行循环
        let mut iteration = 0;
        loop {
            iteration += 1;

            // 获取当前状态
            let state = self.orchestrator.get_state().await?;

            // 发送进度通知
            let progress = WorkflowProgress::new(
                state.phase,
                state.iteration,
                state.max_iterations,
                format!("执行阶段: {}", state.phase),
            );

            if let Some(ref callback) = self.progress_callback {
                callback(progress.clone());
            }

            if self.verbose {
                println!(
                    "🔄 迭代 {}/{} | 阶段: {}",
                    state.iteration, state.max_iterations, state.phase
                );
            }

            // 执行一次迭代
            let should_continue = self.orchestrator.execute_iteration_async(agent).await?;

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

        // 获取最终状态
        let final_state = self.orchestrator.get_state().await?;

        // 生成最终摘要
        let summary = self.orchestrator.generate_summary().await?;

        // 获取最终响应
        let final_response = self.orchestrator.get_final_response().await;

        if self.verbose {
            println!("\n✅ 工作流执行完成\n");
            println!("{}", summary);
        }

        Ok(WorkflowResult {
            success: final_state.phase == WorkflowPhase::Complete,
            phase: final_state.phase,
            iterations: final_state.iteration,
            summary,
            final_response,
            failure_reason: final_state.failure_reason,
        })
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> Result<WorkflowPhase> {
        Ok(self.orchestrator.get_state().await?.phase)
    }

    /// 获取工作流进度百分比
    pub async fn get_progress(&self) -> Result<f32> {
        let state = self.orchestrator.get_state().await?;
        let progress = (state.iteration as f32 / state.max_iterations as f32) * 100.0;
        Ok(progress.min(100.0))
    }

    /// 获取 Orchestrator 引用（用于高级操作）
    pub fn orchestrator(&self) -> &WorkflowOrchestrator {
        &self.orchestrator
    }
}

/// 工作流执行结果
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    /// 是否成功完成
    pub success: bool,
    /// 最终阶段
    pub phase: WorkflowPhase,
    /// 执行的迭代次数
    pub iterations: u32,
    /// 执行摘要
    pub summary: String,
    /// 最终响应内容
    pub final_response: Option<String>,
    /// 失败原因（如果失败）
    pub failure_reason: Option<String>,
}

impl WorkflowResult {
    /// 获取用于显示的响应内容
    pub fn display_response(&self) -> String {
        if let Some(ref response) = self.final_response {
            response.clone()
        } else if self.success {
            self.summary.clone()
        } else {
            format!(
                "工作流执行失败: {}",
                self.failure_reason
                    .as_ref()
                    .unwrap_or(&"未知原因".to_string())
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let executor = WorkflowExecutor::new("Test request".to_string(), subagent_manager);

        let state = executor.get_state().await.unwrap();
        assert_eq!(state, WorkflowPhase::Idle);
    }

    #[tokio::test]
    async fn test_executor_verbose() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let executor = WorkflowExecutor::new("Test request".to_string(), subagent_manager)
            .with_verbose(false);

        assert!(!executor.verbose);
    }

    #[test]
    fn test_workflow_progress() {
        let progress = WorkflowProgress::new(
            WorkflowPhase::Planning,
            1,
            10,
            "测试进度".to_string(),
        );

        assert_eq!(progress.phase, WorkflowPhase::Planning);
        assert_eq!(progress.iteration, 1);
        assert_eq!(progress.percentage, 10.0);
    }

    #[test]
    fn test_workflow_result_display() {
        let result = WorkflowResult {
            success: true,
            phase: WorkflowPhase::Complete,
            iterations: 3,
            summary: "摘要内容".to_string(),
            final_response: Some("最终响应".to_string()),
            failure_reason: None,
        };

        assert_eq!(result.display_response(), "最终响应");

        let failed_result = WorkflowResult {
            success: false,
            phase: WorkflowPhase::Failed,
            iterations: 5,
            summary: "摘要".to_string(),
            final_response: None,
            failure_reason: Some("超时".to_string()),
        };

        assert!(failed_result.display_response().contains("超时"));
    }
}
