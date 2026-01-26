//! 工作流编排器
//!
//! 实现 Plan-Act-Observe-Reflect (PAOR) 循环的核心逻辑。

use super::observation::ObservationCollector;
use super::state::{WorkflowPhase, WorkflowState};
use super::types::{Plan, Reflection, Task, TaskId};
use crate::agent::SubagentManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 工作流编排器配置
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// 最大迭代次数
    pub max_iterations: u32,
    
    /// 是否启用详细日志
    pub verbose: bool,
    
    /// 是否自动重试失败的任务
    pub auto_retry: bool,
    
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 15,
            verbose: false,
            auto_retry: true,
            max_retries: 3,
        }
    }
}

/// 工作流编排器
/// 
/// 负责管理整个 PAOR 循环的执行流程。
pub struct WorkflowOrchestrator {
    /// 工作流状态
    state: Arc<RwLock<WorkflowState>>,
    
    /// 观察数据收集器
    observation_collector: ObservationCollector,
    
    /// 子 agent 管理器
    #[allow(dead_code)]
    subagent_manager: Arc<SubagentManager>,
    
    /// 当前计划
    current_plan: Arc<RwLock<Option<Plan>>>,
    
    /// 反思历史
    reflections: Arc<RwLock<Vec<Reflection>>>,
    
    /// 配置
    config: OrchestratorConfig,
    
    /// 任务注册表（ID -> Task）
    #[allow(dead_code)]
    task_registry: Arc<RwLock<HashMap<TaskId, Task>>>,
}

impl WorkflowOrchestrator {
    /// 创建新的工作流编排器
    pub fn new(
        user_request: String,
        subagent_manager: Arc<SubagentManager>,
        config: Option<OrchestratorConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();
        let state = WorkflowState::new(user_request, config.max_iterations);
        
        Self {
            state: Arc::new(RwLock::new(state)),
            observation_collector: ObservationCollector::new(),
            subagent_manager,
            current_plan: Arc::new(RwLock::new(None)),
            reflections: Arc::new(RwLock::new(Vec::new())),
            config,
            task_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 启动工作流
    pub fn start(&self) -> Result<()> {
        let mut state = self.state.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
        
        if state.phase != WorkflowPhase::Idle {
            anyhow::bail!("Workflow is not in Idle state");
        }
        
        state.transition_to(WorkflowPhase::Planning);
        Ok(())
    }
    
    /// 执行一次完整的 PAOR 循环迭代
    /// 
    /// 返回值表示是否应该继续循环
    pub fn execute_iteration(&self) -> Result<bool> {
        // 检查是否应该终止
        {
            let mut state = self.state.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;

            if state.phase.is_terminal() {
                return Ok(false);
            }

            if state.has_reached_max_iterations() {
                state.mark_requires_intervention(
                    "Maximum iterations reached without achieving goal".to_string(),
                );
                return Ok(false);
            }
        }
        
        // 执行当前阶段
        let current_phase = {
            let state = self.state.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
            state.phase
        };
        
        match current_phase {
            WorkflowPhase::Idle => {
                // 如果还在 Idle，启动工作流
                self.start()?;
                Ok(true)
            }
            
            WorkflowPhase::Planning => {
                self.execute_planning_phase()?;
                Ok(true)
            }
            
            WorkflowPhase::Acting => {
                self.execute_acting_phase()?;
                Ok(true)
            }
            
            WorkflowPhase::Observing => {
                self.execute_observing_phase()?;
                Ok(true)
            }
            
            WorkflowPhase::Reflecting => {
                // Reflecting 阶段会决定下一步动作
                let should_continue = self.execute_reflecting_phase()?;
                Ok(should_continue)
            }
            
            WorkflowPhase::Complete | WorkflowPhase::Failed => {
                Ok(false)
            }
        }
    }
    
    /// 执行计划阶段
    fn execute_planning_phase(&self) -> Result<()> {
        if self.config.verbose {
            println!("📋 Entering Planning phase...");
        }
        
        // TODO: 实际的计划生成逻辑
        // 这里应该调用 LLM 来分析用户请求并生成计划
        // 目前先使用一个简单的占位符实现
        
        let user_request = {
            let state = self.state.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
            state.user_request.clone()
        };
        
        // 创建一个示例计划
        let plan = self.generate_plan(&user_request)?;
        
        // 保存计划
        {
            let mut current_plan = self.current_plan.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire plan lock"))?;
            *current_plan = Some(plan);
        }
        
        // 转换到 Acting 阶段
        {
            let mut state = self.state.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
            state.transition_to(WorkflowPhase::Acting);
        }
        
        Ok(())
    }
    
    /// 执行执行阶段
    fn execute_acting_phase(&self) -> Result<()> {
        if self.config.verbose {
            println!("🎬 Entering Acting phase...");
        }
        
        // TODO: 实际的任务执行逻辑
        // 这里应该根据计划执行工具调用或委派子任务
        
        // 转换到 Observing 阶段
        {
            let mut state = self.state.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
            state.transition_to(WorkflowPhase::Observing);
        }
        
        Ok(())
    }
    
    /// 执行观察阶段
    fn execute_observing_phase(&self) -> Result<()> {
        if self.config.verbose {
            println!("👁️  Entering Observing phase...");
        }
        
        // TODO: 收集和整理观察数据
        // 这里应该从 observation_collector 中获取数据并进行分析
        
        // 转换到 Reflecting 阶段
        {
            let mut state = self.state.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
            state.transition_to(WorkflowPhase::Reflecting);
        }
        
        Ok(())
    }
    
    /// 执行反思阶段
    /// 
    /// 返回值表示是否应该继续循环
    fn execute_reflecting_phase(&self) -> Result<bool> {
        if self.config.verbose {
            println!("🤔 Entering Reflecting phase...");
        }
        
        // TODO: 实际的反思逻辑
        // 这里应该调用 LLM 来评估进展并决定下一步
        
        let reflection = self.generate_reflection()?;
        
        // 保存反思
        {
            let mut reflections = self.reflections.write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire reflections lock"))?;
            reflections.push(reflection.clone());
        }
        
        // 根据反思结果决定下一步
        let mut state = self.state.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
        
        if reflection.goal_achieved {
            state.mark_complete();
            return Ok(false);
        }
        
        if reflection.requires_user_intervention {
            state.mark_requires_intervention(
                reflection.issues.join("; ")
            );
            return Ok(false);
        }
        
        if state.has_reached_max_iterations() {
            state.mark_failed("Maximum iterations reached without achieving goal".to_string());
            return Ok(false);
        }
        
        // 继续下一轮迭代
        state.transition_to(WorkflowPhase::Planning);
        Ok(true)
    }
    
    /// 生成计划（占位符实现）
    fn generate_plan(&self, user_request: &str) -> Result<Plan> {
        // TODO: 实际应该调用 LLM 来生成计划
        // 这里只是一个简单的示例
        
        let task1 = Task::new(
            "task_1".to_string(),
            format!("Analyze request: {}", user_request),
        );
        
        let task2 = Task::new(
            "task_2".to_string(),
            "Execute the plan".to_string(),
        ).with_dependency("task_1".to_string());
        
        Ok(Plan::new(
            "plan_1".to_string(),
            "Auto-generated plan".to_string(),
            vec![task1, task2],
        ))
    }
    
    /// 生成反思（占位符实现）
    fn generate_reflection(&self) -> Result<Reflection> {
        // TODO: 实际应该调用 LLM 来生成反思
        // 这里只是一个简单的示例
        
        let iteration = {
            let state = self.state.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))?;
            state.iteration
        };
        
        // 简单示例：第一次迭代后就认为完成
        let goal_achieved = iteration >= 1;
        let progress = if goal_achieved { 1.0 } else { 0.5 };
        
        Ok(Reflection::new(
            goal_achieved,
            progress,
            "Reflection placeholder".to_string(),
            Some("Continue execution".to_string()),
        ))
    }
    
    /// 获取当前状态
    pub fn get_state(&self) -> Result<WorkflowState> {
        self.state.read()
            .map(|s| s.clone())
            .map_err(|_| anyhow::anyhow!("Failed to acquire state lock"))
    }
    
    /// 获取观察收集器
    pub fn get_observation_collector(&self) -> &ObservationCollector {
        &self.observation_collector
    }
    
    /// 获取所有反思
    pub fn get_reflections(&self) -> Result<Vec<Reflection>> {
        self.reflections.read()
            .map(|r| r.clone())
            .map_err(|_| anyhow::anyhow!("Failed to acquire reflections lock"))
    }
    
    /// 生成最终摘要
    pub fn generate_summary(&self) -> Result<String> {
        let state = self.get_state()?;
        let summary = self.observation_collector.summarize();
        let reflections = self.get_reflections()?;
        
        let mut output = String::new();
        output.push_str(&format!("# Workflow Summary\n\n"));
        output.push_str(&format!("**Status**: {}\n", state.phase));
        output.push_str(&format!("**Iterations**: {}/{}\n", state.iteration, state.max_iterations));
        output.push_str(&format!("**Duration**: {}ms\n\n", state.elapsed_ms()));
        
        output.push_str(&format!("## Observations\n"));
        output.push_str(&format!("- Total: {}\n", summary.total_observations));
        output.push_str(&format!("- Successful: {}\n", summary.successful));
        output.push_str(&format!("- Failed: {}\n", summary.failed));
        output.push_str(&format!("- Tool Executions: {}\n", summary.tool_executions));
        output.push_str(&format!("- Subagent Calls: {}\n\n", summary.subagent_calls));
        
        if !reflections.is_empty() {
            output.push_str(&format!("## Reflections\n"));
            for (i, reflection) in reflections.iter().enumerate() {
                output.push_str(&format!("{}. Progress: {:.0}% - {}\n", 
                    i + 1, 
                    reflection.progress * 100.0,
                    reflection.content
                ));
            }
        }
        
        if let Some(reason) = &state.failure_reason {
            output.push_str(&format!("\n**Failure Reason**: {}\n", reason));
        }
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_orchestrator_creation() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator = WorkflowOrchestrator::new(
            "Test request".to_string(),
            subagent_manager,
            None,
        );
        
        let state = orchestrator.get_state().unwrap();
        assert_eq!(state.phase, WorkflowPhase::Idle);
        assert_eq!(state.iteration, 0);
    }
    
    #[test]
    fn test_orchestrator_start() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator = WorkflowOrchestrator::new(
            "Test request".to_string(),
            subagent_manager,
            None,
        );
        
        orchestrator.start().unwrap();
        
        let state = orchestrator.get_state().unwrap();
        assert_eq!(state.phase, WorkflowPhase::Planning);
    }
    
    #[test]
    fn test_orchestrator_iteration() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator = WorkflowOrchestrator::new(
            "Test request".to_string(),
            subagent_manager,
            Some(OrchestratorConfig {
                verbose: false,
                ..Default::default()
            }),
        );
        
        // 第一次迭代应该从 Idle 开始
        let should_continue = orchestrator.execute_iteration().unwrap();
        assert!(should_continue);
        
        let state = orchestrator.get_state().unwrap();
        assert_eq!(state.phase, WorkflowPhase::Planning);
    }
}
