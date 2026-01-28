//! 工作流编排器
//!
//! 实现 Plan-Act-Observe-Reflect (PAOR) 循环的核心逻辑。

use super::observation::ObservationCollector;
use super::state::{WorkflowPhase, WorkflowState};
use super::types::{ExecutionType, ObservationAnalysis, Plan, Reflection, Task, TaskId, TaskStatus};
use crate::agent::builder::AgentEnum;
use crate::agent::SubagentManager;
use anyhow::Result;
use rig::completion::Prompt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    subagent_manager: Arc<SubagentManager>,

    /// 当前计划
    current_plan: Arc<RwLock<Option<Plan>>>,

    /// 反思历史
    reflections: Arc<RwLock<Vec<Reflection>>>,

    /// 配置
    config: OrchestratorConfig,

    /// 任务注册表（ID -> Task）
    task_registry: Arc<RwLock<HashMap<TaskId, Task>>>,

    /// 观察分析结果
    observation_analysis: Arc<RwLock<Option<ObservationAnalysis>>>,

    /// 最终响应内容（用于返回给用户）
    final_response: Arc<RwLock<Option<String>>>,
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
            observation_analysis: Arc::new(RwLock::new(None)),
            final_response: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动工作流
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if state.phase != WorkflowPhase::Idle {
            anyhow::bail!("Workflow is not in Idle state");
        }

        state.transition_to(WorkflowPhase::Planning);
        Ok(())
    }

    /// 异步执行一次完整的 PAOR 循环迭代
    ///
    /// 返回值表示是否应该继续循环
    pub async fn execute_iteration_async(&self, agent: &AgentEnum) -> Result<bool> {
        // 检查是否应该终止
        {
            let mut state = self.state.write().await;

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
            let state = self.state.read().await;
            state.phase
        };

        match current_phase {
            WorkflowPhase::Idle => {
                // 如果还在 Idle，启动工作流
                self.start().await?;
                Ok(true)
            }

            WorkflowPhase::Planning => {
                self.execute_planning_phase_async(agent).await?;
                Ok(true)
            }

            WorkflowPhase::Acting => {
                self.execute_acting_phase_async(agent).await?;
                Ok(true)
            }

            WorkflowPhase::Observing => {
                self.execute_observing_phase().await?;
                Ok(true)
            }

            WorkflowPhase::Reflecting => {
                // Reflecting 阶段会决定下一步动作
                let should_continue = self.execute_reflecting_phase_async(agent).await?;
                Ok(should_continue)
            }

            WorkflowPhase::Complete | WorkflowPhase::Failed => Ok(false),
        }
    }

    /// 异步执行计划阶段
    async fn execute_planning_phase_async(&self, agent: &AgentEnum) -> Result<()> {
        if self.config.verbose {
            println!("📋 进入规划阶段...");
        }

        let (user_request, reflections_summary) = {
            let state = self.state.read().await;
            let reflections = self.reflections.read().await;

            let reflections_summary = if reflections.is_empty() {
                String::new()
            } else {
                let summaries: Vec<String> = reflections
                    .iter()
                    .map(|r| format!("- 进度: {:.0}%, 内容: {}", r.progress * 100.0, r.content))
                    .collect();
                format!("\n\n## 历史反思\n{}", summaries.join("\n"))
            };

            (state.user_request.clone(), reflections_summary)
        };

        // 构建规划提示词
        let planning_prompt = format!(
            r#"你是一个任务规划专家。请分析以下用户请求，并生成一个结构化的执行计划。

## 用户请求
{}
{}

## 输出要求
请以 JSON 格式输出计划，格式如下：
```json
{{
  "description": "计划的整体描述",
  "tasks": [
    {{
      "id": "task_1",
      "description": "任务描述",
      "execution_type": "llm",
      "dependencies": []
    }}
  ]
}}
```

execution_type 可选值：
- "llm": 直接使用 LLM 推理（默认）
- "tool_call": 需要调用工具，需指定 "tool_name"
- "subagent": 委派给子 Agent，需指定 "agent_type"（可选：Explore, Plan, CodeReviewer）

请确保任务之间的依赖关系正确，并按执行顺序排列。"#,
            user_request, reflections_summary
        );

        // 调用 LLM 生成计划
        let response = self.call_llm(agent, &planning_prompt).await?;

        // 解析计划
        let plan = match Plan::from_llm_response(&response) {
            Ok(p) => p,
            Err(e) => {
                if self.config.verbose {
                    println!("⚠️  计划解析失败: {}，使用默认计划", e);
                }
                // 使用默认计划
                self.generate_default_plan(&user_request)
            }
        };

        // 注册任务
        {
            let mut registry = self.task_registry.write().await;
            for task in &plan.tasks {
                registry.insert(task.id.clone(), task.clone());
            }
        }

        // 保存计划
        {
            let mut current_plan = self.current_plan.write().await;
            *current_plan = Some(plan);
        }

        // 转换到 Acting 阶段
        {
            let mut state = self.state.write().await;
            state.transition_to(WorkflowPhase::Acting);
        }

        Ok(())
    }

    /// 异步执行执行阶段
    async fn execute_acting_phase_async(&self, agent: &AgentEnum) -> Result<()> {
        if self.config.verbose {
            println!("🎬 进入执行阶段...");
        }

        // 获取可执行的任务（依赖已满足的 Pending 任务）
        let executable_tasks = self.get_executable_tasks().await;

        if executable_tasks.is_empty() {
            if self.config.verbose {
                println!("  没有可执行的任务");
            }
            // 转换到 Observing 阶段
            let mut state = self.state.write().await;
            state.transition_to(WorkflowPhase::Observing);
            return Ok(());
        }

        // 执行每个任务
        for task in executable_tasks {
            if self.config.verbose {
                println!("  执行任务: {} - {}", task.id, task.description);
            }

            // 标记任务开始
            self.update_task_status(&task.id, TaskStatus::Running).await;

            let start_time = std::time::Instant::now();

            // 根据执行类型执行任务
            let result = match &task.execution_type {
                ExecutionType::ToolCall(tool_name) => {
                    self.execute_tool_task(agent, &task, tool_name).await
                }
                ExecutionType::SubagentDelegation(agent_type) => {
                    self.execute_subagent_task(&task, *agent_type).await
                }
                ExecutionType::DirectLLM => self.execute_llm_task(agent, &task).await,
            };

            let execution_time = start_time.elapsed().as_millis() as u64;

            // 记录观察数据
            match &result {
                Ok(output) => {
                    self.update_task_status(&task.id, TaskStatus::Completed).await;
                    self.observation_collector.add_tool_execution(
                        format!("{:?}", task.execution_type),
                        HashMap::new(),
                        Some(serde_json::json!(output)),
                        true,
                        None,
                        Some(execution_time),
                    );
                }
                Err(e) => {
                    self.update_task_status(&task.id, TaskStatus::Failed).await;
                    self.observation_collector.add_tool_execution(
                        format!("{:?}", task.execution_type),
                        HashMap::new(),
                        None,
                        false,
                        Some(e.to_string()),
                        Some(execution_time),
                    );
                }
            }
        }

        // 转换到 Observing 阶段
        {
            let mut state = self.state.write().await;
            state.transition_to(WorkflowPhase::Observing);
        }

        Ok(())
    }

    /// 执行观察阶段
    async fn execute_observing_phase(&self) -> Result<()> {
        if self.config.verbose {
            println!("👁️  进入观察阶段...");
        }

        // 收集本轮迭代的观察数据
        let observations = self.observation_collector.get_all();
        let summary = self.observation_collector.summarize();

        // 分析观察数据
        let mut analysis = ObservationAnalysis::new();
        analysis.total_actions = summary.total_observations;
        analysis.successful = summary.successful;
        analysis.failed = summary.failed;

        // 提取关键发现
        for obs in &observations {
            if obs.success {
                if let Some(output) = &obs.output {
                    let output_str = output.to_string();
                    if output_str.len() > 10 {
                        analysis.add_progress(format!("{} 执行成功", obs.source));
                    }
                }
            } else if let Some(error) = &obs.error {
                analysis.add_blocker(format!("{} 失败: {}", obs.source, error));
            }
        }

        // 保存分析结果
        {
            let mut obs_analysis = self.observation_analysis.write().await;
            *obs_analysis = Some(analysis);
        }

        // 转换到 Reflecting 阶段
        {
            let mut state = self.state.write().await;
            state.transition_to(WorkflowPhase::Reflecting);
        }

        Ok(())
    }

    /// 异步执行反思阶段
    ///
    /// 返回值表示是否应该继续循环
    async fn execute_reflecting_phase_async(&self, agent: &AgentEnum) -> Result<bool> {
        if self.config.verbose {
            println!("🤔 进入反思阶段...");
        }

        let (user_request, plan_summary, obs_summary, iteration) = {
            let state = self.state.read().await;
            let plan = self.current_plan.read().await;
            let obs_analysis = self.observation_analysis.read().await;

            let plan_summary = plan
                .as_ref()
                .map(|p| {
                    let task_summaries: Vec<String> = p
                        .tasks
                        .iter()
                        .map(|t| format!("- [{}] {}: {:?}", t.id, t.description, t.status))
                        .collect();
                    format!(
                        "计划: {}\n任务:\n{}",
                        p.description,
                        task_summaries.join("\n")
                    )
                })
                .unwrap_or_else(|| "无计划".to_string());

            let obs_summary = obs_analysis
                .as_ref()
                .map(|a| {
                    format!(
                        "总操作: {}, 成功: {}, 失败: {}\n关键发现: {:?}\n阻塞问题: {:?}",
                        a.total_actions,
                        a.successful,
                        a.failed,
                        a.key_findings,
                        a.blockers
                    )
                })
                .unwrap_or_else(|| "无观察数据".to_string());

            (
                state.user_request.clone(),
                plan_summary,
                obs_summary,
                state.iteration,
            )
        };

        // 构建反思提示词
        let reflection_prompt = format!(
            r#"你是一个任务评估专家。请评估当前任务的执行进展。

## 原始请求
{}

## 当前计划
{}

## 观察数据
{}

## 当前迭代
第 {} 轮

## 输出要求
请以 JSON 格式输出评估结果：
```json
{{
  "goal_achieved": true/false,
  "progress": 0.0-1.0,
  "content": "评估内容描述",
  "next_action": "下一步建议（如果未完成）",
  "requires_user_intervention": true/false,
  "issues": ["问题1", "问题2"]
}}
```

请根据观察数据判断：
1. 目标是否已达成
2. 当前进度百分比
3. 是否需要用户干预
4. 下一步应该做什么"#,
            user_request, plan_summary, obs_summary, iteration
        );

        // 调用 LLM 生成反思
        let response = self.call_llm(agent, &reflection_prompt).await?;

        // 解析反思结果
        let reflection = self.parse_reflection_response(&response);

        // 保存反思
        {
            let mut reflections = self.reflections.write().await;
            reflections.push(reflection.clone());
        }

        // 根据反思结果决定下一步
        let mut state = self.state.write().await;

        if reflection.goal_achieved {
            // 保存最终响应
            {
                let mut final_resp = self.final_response.write().await;
                *final_resp = Some(reflection.content.clone());
            }
            state.mark_complete();
            return Ok(false);
        }

        if reflection.requires_user_intervention {
            state.mark_requires_intervention(reflection.issues.join("; "));
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

    /// 调用 LLM
    async fn call_llm(&self, agent: &AgentEnum, prompt: &str) -> Result<String> {
        match agent {
            AgentEnum::Anthropic(a) => {
                let response = a.prompt(prompt).await?;
                Ok(response)
            }
            AgentEnum::OpenAI(a) => {
                let response = a.prompt(prompt).await?;
                Ok(response)
            }
        }
    }

    /// 获取可执行的任务
    async fn get_executable_tasks(&self) -> Vec<Task> {
        let plan = self.current_plan.read().await;
        let registry = self.task_registry.read().await;

        let Some(plan) = plan.as_ref() else {
            return Vec::new();
        };

        plan.tasks
            .iter()
            .filter(|task| {
                // 只选择 Pending 状态的任务
                if task.status != TaskStatus::Pending {
                    return false;
                }

                // 检查依赖是否都已完成
                task.dependencies.iter().all(|dep_id| {
                    registry
                        .get(dep_id)
                        .map(|dep| dep.status == TaskStatus::Completed)
                        .unwrap_or(true)
                })
            })
            .cloned()
            .collect()
    }

    /// 更新任务状态
    async fn update_task_status(&self, task_id: &str, status: TaskStatus) {
        let mut registry = self.task_registry.write().await;
        if let Some(task) = registry.get_mut(task_id) {
            task.status = status;
            if status == TaskStatus::Running {
                task.mark_started();
            }
        }

        // 同时更新计划中的任务状态
        let mut plan = self.current_plan.write().await;
        if let Some(p) = plan.as_mut() {
            for task in &mut p.tasks {
                if task.id == task_id {
                    task.status = status;
                    break;
                }
            }
        }
    }

    /// 执行工具任务
    async fn execute_tool_task(
        &self,
        agent: &AgentEnum,
        task: &Task,
        tool_name: &str,
    ) -> Result<String> {
        // 构建工具调用提示词
        let prompt = format!(
            "请使用 {} 工具完成以下任务：\n\n{}",
            tool_name, task.description
        );

        self.call_llm(agent, &prompt).await
    }

    /// 执行子 Agent 任务
    async fn execute_subagent_task(
        &self,
        task: &Task,
        agent_type: crate::agent::types::AgentType,
    ) -> Result<String> {
        self.subagent_manager
            .delegate(agent_type, &task.description)
            .await
    }

    /// 执行 LLM 任务
    async fn execute_llm_task(&self, agent: &AgentEnum, task: &Task) -> Result<String> {
        self.call_llm(agent, &task.description).await
    }

    /// 生成默认计划
    fn generate_default_plan(&self, user_request: &str) -> Plan {
        let task = Task::new(
            "task_1".to_string(),
            format!("分析并完成请求: {}", user_request),
        )
        .with_execution_type(ExecutionType::DirectLLM);

        Plan::new(
            format!("plan_{}", uuid::Uuid::new_v4()),
            "自动生成的默认计划".to_string(),
            vec![task],
        )
    }

    /// 解析反思响应
    fn parse_reflection_response(&self, response: &str) -> Reflection {
        // 尝试从响应中提取 JSON
        if let Some(json_str) = Self::extract_json_from_response(response) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                let goal_achieved = json
                    .get("goal_achieved")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let progress = json
                    .get("progress")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5) as f32;

                let content = json
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("反思完成")
                    .to_string();

                let next_action = json
                    .get("next_action")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let requires_intervention = json
                    .get("requires_user_intervention")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let issues: Vec<String> = json
                    .get("issues")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let mut reflection = Reflection::new(goal_achieved, progress, content, next_action);
                reflection.requires_user_intervention = requires_intervention;
                reflection.issues = issues;

                return reflection;
            }
        }

        // 默认反思
        Reflection::new(
            false,
            0.5,
            "无法解析反思结果，继续执行".to_string(),
            Some("继续下一轮迭代".to_string()),
        )
    }

    /// 从响应中提取 JSON
    fn extract_json_from_response(response: &str) -> Option<String> {
        // 尝试从 markdown 代码块中提取
        if let Some(start) = response.find("```json") {
            let content_start = start + 7;
            if let Some(end) = response[content_start..].find("```") {
                return Some(response[content_start..content_start + end].trim().to_string());
            }
        }

        // 尝试找到 JSON 对象
        if let Some(start) = response.find('{') {
            let mut depth = 0;
            let mut end = start;
            for (i, c) in response[start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = start + i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth == 0 && end > start {
                return Some(response[start..end].to_string());
            }
        }

        None
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> Result<WorkflowState> {
        Ok(self.state.read().await.clone())
    }

    /// 获取观察收集器
    pub fn get_observation_collector(&self) -> &ObservationCollector {
        &self.observation_collector
    }

    /// 获取所有反思
    pub async fn get_reflections(&self) -> Result<Vec<Reflection>> {
        Ok(self.reflections.read().await.clone())
    }

    /// 获取最终响应
    pub async fn get_final_response(&self) -> Option<String> {
        self.final_response.read().await.clone()
    }

    /// 生成最终摘要
    pub async fn generate_summary(&self) -> Result<String> {
        let state = self.get_state().await?;
        let summary = self.observation_collector.summarize();
        let reflections = self.get_reflections().await?;

        let mut output = String::new();
        output.push_str("# 工作流摘要\n\n");
        output.push_str(&format!("**状态**: {}\n", state.phase));
        output.push_str(&format!(
            "**迭代次数**: {}/{}\n",
            state.iteration, state.max_iterations
        ));
        output.push_str(&format!("**耗时**: {}ms\n\n", state.elapsed_ms()));

        output.push_str("## 观察数据\n");
        output.push_str(&format!("- 总计: {}\n", summary.total_observations));
        output.push_str(&format!("- 成功: {}\n", summary.successful));
        output.push_str(&format!("- 失败: {}\n", summary.failed));
        output.push_str(&format!("- 工具执行: {}\n", summary.tool_executions));
        output.push_str(&format!("- 子Agent调用: {}\n\n", summary.subagent_calls));

        if !reflections.is_empty() {
            output.push_str("## 反思历史\n");
            for (i, reflection) in reflections.iter().enumerate() {
                output.push_str(&format!(
                    "{}. 进度: {:.0}% - {}\n",
                    i + 1,
                    reflection.progress * 100.0,
                    reflection.content
                ));
            }
        }

        if let Some(reason) = &state.failure_reason {
            output.push_str(&format!("\n**失败原因**: {}\n", reason));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator =
            WorkflowOrchestrator::new("Test request".to_string(), subagent_manager, None);

        let state = orchestrator.get_state().await.unwrap();
        assert_eq!(state.phase, WorkflowPhase::Idle);
        assert_eq!(state.iteration, 0);
    }

    #[tokio::test]
    async fn test_orchestrator_start() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator =
            WorkflowOrchestrator::new("Test request".to_string(), subagent_manager, None);

        orchestrator.start().await.unwrap();

        let state = orchestrator.get_state().await.unwrap();
        assert_eq!(state.phase, WorkflowPhase::Planning);
    }

    #[test]
    fn test_extract_json_from_response() {
        let response = r#"
好的，这是我的分析：

```json
{
    "goal_achieved": true,
    "progress": 1.0
}
```

完成了。
"#;

        let json = WorkflowOrchestrator::extract_json_from_response(response);
        assert!(json.is_some());
        let json_str = json.unwrap();
        assert!(json_str.contains("goal_achieved"));
    }

    #[test]
    fn test_parse_reflection_response() {
        let subagent_manager = Arc::new(SubagentManager::new());
        let orchestrator =
            WorkflowOrchestrator::new("Test".to_string(), subagent_manager, None);

        let response = r#"```json
{
    "goal_achieved": true,
    "progress": 1.0,
    "content": "任务完成",
    "next_action": null,
    "requires_user_intervention": false,
    "issues": []
}
```"#;

        let reflection = orchestrator.parse_reflection_response(response);
        assert!(reflection.goal_achieved);
        assert_eq!(reflection.progress, 1.0);
        assert_eq!(reflection.content, "任务完成");
    }
}
