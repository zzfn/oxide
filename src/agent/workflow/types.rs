//! 工作流类型定义
//!
//! 定义任务、计划、观察和反思等核心数据结构。

use crate::agent::types::AgentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// 执行类型
///
/// 定义任务可以通过哪种方式执行
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionType {
    /// 工具调用（指定工具名称）
    ToolCall(String),

    /// 子 Agent 委派（指定 Agent 类型）
    SubagentDelegation(AgentType),

    /// 直接 LLM 调用（用于推理、分析等）
    DirectLLM,
}

impl Default for ExecutionType {
    fn default() -> Self {
        ExecutionType::DirectLLM
    }
}

/// 观察分析结果
///
/// 对一轮迭代中收集的观察数据进行分析的结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservationAnalysis {
    /// 总操作数
    pub total_actions: usize,

    /// 成功数
    pub successful: usize,

    /// 失败数
    pub failed: usize,

    /// 关键发现
    pub key_findings: Vec<String>,

    /// 阻塞问题
    pub blockers: Vec<String>,

    /// 进度指标
    pub progress_indicators: Vec<String>,
}

impl ObservationAnalysis {
    /// 创建新的观察分析结果
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加关键发现
    pub fn add_finding(&mut self, finding: String) {
        self.key_findings.push(finding);
    }

    /// 添加阻塞问题
    pub fn add_blocker(&mut self, blocker: String) {
        self.blockers.push(blocker);
    }

    /// 添加进度指标
    pub fn add_progress(&mut self, indicator: String) {
        self.progress_indicators.push(indicator);
    }

    /// 计算成功率
    pub fn success_rate(&self) -> f32 {
        if self.total_actions == 0 {
            1.0
        } else {
            self.successful as f32 / self.total_actions as f32
        }
    }

    /// 是否有阻塞问题
    pub fn has_blockers(&self) -> bool {
        !self.blockers.is_empty()
    }
}

/// 任务 ID
pub type TaskId = String;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    
    /// 正在执行
    Running,
    
    /// 已完成
    Completed,
    
    /// 失败
    Failed,
    
    /// 已取消
    Cancelled,
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: TaskId,

    /// 任务描述
    pub description: String,

    /// 父任务 ID（如果是子任务）
    pub parent_id: Option<TaskId>,

    /// 依赖的任务 ID 列表
    pub dependencies: Vec<TaskId>,

    /// 任务状态
    pub status: TaskStatus,

    /// 执行类型
    pub execution_type: ExecutionType,

    /// 创建时间
    pub created_at: SystemTime,

    /// 开始时间
    pub started_at: Option<SystemTime>,

    /// 完成时间
    pub completed_at: Option<SystemTime>,

    /// 任务结果
    pub result: Option<String>,

    /// 错误信息
    pub error: Option<String>,
}

impl Task {
    /// 创建新任务
    pub fn new(id: TaskId, description: String) -> Self {
        Self {
            id,
            description,
            parent_id: None,
            dependencies: Vec::new(),
            status: TaskStatus::Pending,
            execution_type: ExecutionType::default(),
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }

    /// 设置执行类型
    pub fn with_execution_type(mut self, execution_type: ExecutionType) -> Self {
        self.execution_type = execution_type;
        self
    }
    
    /// 添加依赖
    pub fn with_dependency(mut self, dependency: TaskId) -> Self {
        self.dependencies.push(dependency);
        self
    }
    
    /// 添加父任务
    pub fn with_parent(mut self, parent_id: TaskId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
    
    /// 标记任务开始
    pub fn mark_started(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(SystemTime::now());
    }
    
    /// 标记任务完成
    pub fn mark_completed(&mut self, result: String) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(SystemTime::now());
        self.result = Some(result);
    }
    
    /// 标记任务失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(SystemTime::now());
        self.error = Some(error);
    }
    
    /// 检查任务是否完成（成功或失败）
    pub fn is_finished(&self) -> bool {
        matches!(self.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled)
    }
}

/// 计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// 计划 ID
    pub id: String,
    
    /// 计划描述
    pub description: String,
    
    /// 要执行的任务列表
    pub tasks: Vec<Task>,
    
    /// 估计的步骤数
    pub estimated_steps: usize,
    
    /// 创建时间
    pub created_at: SystemTime,
}

impl Plan {
    /// 创建新计划
    pub fn new(id: String, description: String, tasks: Vec<Task>) -> Self {
        let estimated_steps = tasks.len();
        Self {
            id,
            description,
            tasks,
            estimated_steps,
            created_at: SystemTime::now(),
        }
    }

    /// 从 LLM JSON 响应解析计划
    ///
    /// 支持的 JSON 格式：
    /// ```json
    /// {
    ///   "description": "计划描述",
    ///   "tasks": [
    ///     {
    ///       "id": "task_1",
    ///       "description": "任务描述",
    ///       "execution_type": "tool_call" | "subagent" | "llm",
    ///       "tool_name": "read_file",  // 当 execution_type 为 tool_call 时
    ///       "agent_type": "Explore",   // 当 execution_type 为 subagent 时
    ///       "dependencies": ["task_0"]
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn from_llm_response(response: &str) -> Result<Self, String> {
        // 尝试提取 JSON 块
        let json_str = Self::extract_json(response)?;

        // 解析 JSON
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("JSON 解析失败: {}", e))?;

        // 提取计划描述
        let description = json
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("自动生成的计划")
            .to_string();

        // 提取任务列表
        let tasks_json = json
            .get("tasks")
            .and_then(|v| v.as_array())
            .ok_or("缺少 tasks 字段")?;

        let mut tasks = Vec::new();
        for (idx, task_json) in tasks_json.iter().enumerate() {
            let task_id = task_json
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("task_{}", idx));

            let task_desc = task_json
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("未命名任务")
                .to_string();

            let mut task = Task::new(task_id, task_desc);

            // 解析执行类型
            if let Some(exec_type) = task_json.get("execution_type").and_then(|v| v.as_str()) {
                task.execution_type = match exec_type {
                    "tool_call" | "tool" => {
                        let tool_name = task_json
                            .get("tool_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        ExecutionType::ToolCall(tool_name)
                    }
                    "subagent" | "delegate" => {
                        let agent_type_str = task_json
                            .get("agent_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Explore");
                        let agent_type = match agent_type_str {
                            "Explore" => AgentType::Explore,
                            "Plan" => AgentType::Plan,
                            "CodeReviewer" => AgentType::CodeReviewer,
                            "FrontendDeveloper" => AgentType::FrontendDeveloper,
                            _ => AgentType::Main,
                        };
                        ExecutionType::SubagentDelegation(agent_type)
                    }
                    _ => ExecutionType::DirectLLM,
                };
            }

            // 解析依赖
            if let Some(deps) = task_json.get("dependencies").and_then(|v| v.as_array()) {
                for dep in deps {
                    if let Some(dep_id) = dep.as_str() {
                        task.dependencies.push(dep_id.to_string());
                    }
                }
            }

            tasks.push(task);
        }

        let plan_id = format!("plan_{}", uuid::Uuid::new_v4());
        Ok(Plan::new(plan_id, description, tasks))
    }

    /// 从响应中提取 JSON 块
    fn extract_json(response: &str) -> Result<String, String> {
        // 尝试直接解析
        if response.trim().starts_with('{') {
            return Ok(response.trim().to_string());
        }

        // 尝试从 markdown 代码块中提取
        if let Some(start) = response.find("```json") {
            let content_start = start + 7;
            if let Some(end) = response[content_start..].find("```") {
                return Ok(response[content_start..content_start + end].trim().to_string());
            }
        }

        // 尝试从普通代码块中提取
        if let Some(start) = response.find("```") {
            let content_start = start + 3;
            // 跳过可能的语言标识符
            let content_start = response[content_start..]
                .find('\n')
                .map(|i| content_start + i + 1)
                .unwrap_or(content_start);
            if let Some(end) = response[content_start..].find("```") {
                let json_str = response[content_start..content_start + end].trim();
                if json_str.starts_with('{') {
                    return Ok(json_str.to_string());
                }
            }
        }

        // 尝试找到 JSON 对象
        if let Some(start) = response.find('{') {
            // 找到匹配的结束括号
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
                return Ok(response[start..end].to_string());
            }
        }

        Err("无法从响应中提取 JSON".to_string())
    }
    
    /// 获取待执行的任务
    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.tasks.iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect()
    }
    
    /// 检查计划是否完成
    pub fn is_complete(&self) -> bool {
        self.tasks.iter().all(|t| t.is_finished())
    }
    
    /// 获取完成进度（0.0 到 1.0）
    pub fn progress(&self) -> f32 {
        if self.tasks.is_empty() {
            return 1.0;
        }
        let completed = self.tasks.iter().filter(|t| t.is_finished()).count();
        completed as f32 / self.tasks.len() as f32
    }
}

/// 观察数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// 观察 ID
    pub id: String,
    
    /// 观察类型（tool_execution, sub_agent_result, error, etc.）
    pub observation_type: String,
    
    /// 观察的源（工具名称、子 agent 类型等）
    pub source: String,
    
    /// 输入数据
    pub input: HashMap<String, serde_json::Value>,
    
    /// 输出数据
    pub output: Option<serde_json::Value>,
    
    /// 是否成功
    pub success: bool,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 执行时间（毫秒）
    pub execution_time_ms: Option<u64>,
    
    /// 时间戳
    pub timestamp: SystemTime,
    
    /// 额外的元数据
    pub metadata: HashMap<String, String>,
}

impl Observation {
    /// 创建工具执行观察
    pub fn tool_execution(
        tool_name: String,
        input: HashMap<String, serde_json::Value>,
        output: Option<serde_json::Value>,
        success: bool,
        error: Option<String>,
        execution_time_ms: Option<u64>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            observation_type: "tool_execution".to_string(),
            source: tool_name,
            input,
            output,
            success,
            error,
            execution_time_ms,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// 创建子 agent 结果观察
    pub fn subagent_result(
        agent_type: String,
        input: String,
        output: String,
        success: bool,
    ) -> Self {
        let mut input_map = HashMap::new();
        input_map.insert("request".to_string(), serde_json::json!(input));
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            observation_type: "subagent_result".to_string(),
            source: agent_type,
            input: input_map,
            output: Some(serde_json::json!(output)),
            success,
            error: None,
            execution_time_ms: None,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
}

/// 反思结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// 反思 ID
    pub id: String,
    
    /// 目标是否达成
    pub goal_achieved: bool,
    
    /// 进展评估（0.0 到 1.0）
    pub progress: f32,
    
    /// 反思内容
    pub content: String,
    
    /// 下一步行动建议
    pub next_action: Option<String>,
    
    /// 是否需要用户干预
    pub requires_user_intervention: bool,
    
    /// 遇到的问题
    pub issues: Vec<String>,
    
    /// 时间戳
    pub timestamp: SystemTime,
}

impl Reflection {
    /// 创建新的反思
    pub fn new(
        goal_achieved: bool,
        progress: f32,
        content: String,
        next_action: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            goal_achieved,
            progress,
            content,
            next_action,
            requires_user_intervention: false,
            issues: Vec::new(),
            timestamp: SystemTime::now(),
        }
    }
    
    /// 添加问题
    pub fn with_issue(mut self, issue: String) -> Self {
        self.issues.push(issue);
        self
    }
    
    /// 标记需要用户干预
    pub fn mark_requires_intervention(mut self) -> Self {
        self.requires_user_intervention = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_lifecycle() {
        let mut task = Task::new("task1".to_string(), "Test task".to_string());
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.execution_type, ExecutionType::DirectLLM);
        assert!(!task.is_finished());

        task.mark_started();
        assert_eq!(task.status, TaskStatus::Running);
        assert!(!task.is_finished());

        task.mark_completed("Success".to_string());
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.is_finished());
    }

    #[test]
    fn test_task_with_execution_type() {
        let task = Task::new("task1".to_string(), "Read file".to_string())
            .with_execution_type(ExecutionType::ToolCall("read_file".to_string()));

        assert_eq!(
            task.execution_type,
            ExecutionType::ToolCall("read_file".to_string())
        );
    }

    #[test]
    fn test_plan_progress() {
        let tasks = vec![
            Task::new("t1".to_string(), "Task 1".to_string()),
            Task::new("t2".to_string(), "Task 2".to_string()),
            Task::new("t3".to_string(), "Task 3".to_string()),
        ];

        let mut plan = Plan::new("plan1".to_string(), "Test plan".to_string(), tasks);
        assert_eq!(plan.progress(), 0.0);
        assert!(!plan.is_complete());

        plan.tasks[0].mark_completed("Done".to_string());
        assert!(plan.progress() > 0.0 && plan.progress() < 1.0);

        plan.tasks[1].mark_completed("Done".to_string());
        plan.tasks[2].mark_completed("Done".to_string());
        assert_eq!(plan.progress(), 1.0);
        assert!(plan.is_complete());
    }

    #[test]
    fn test_plan_from_llm_response_json() {
        let json_response = r#"{
            "description": "测试计划",
            "tasks": [
                {
                    "id": "task_1",
                    "description": "读取文件",
                    "execution_type": "tool_call",
                    "tool_name": "read_file"
                },
                {
                    "id": "task_2",
                    "description": "分析代码",
                    "execution_type": "llm",
                    "dependencies": ["task_1"]
                }
            ]
        }"#;

        let plan = Plan::from_llm_response(json_response).unwrap();
        assert_eq!(plan.description, "测试计划");
        assert_eq!(plan.tasks.len(), 2);
        assert_eq!(
            plan.tasks[0].execution_type,
            ExecutionType::ToolCall("read_file".to_string())
        );
        assert_eq!(plan.tasks[1].execution_type, ExecutionType::DirectLLM);
        assert_eq!(plan.tasks[1].dependencies, vec!["task_1".to_string()]);
    }

    #[test]
    fn test_plan_from_llm_response_markdown() {
        let markdown_response = r#"
好的，我来制定一个计划：

```json
{
    "description": "代码分析计划",
    "tasks": [
        {
            "id": "explore",
            "description": "探索代码库",
            "execution_type": "subagent",
            "agent_type": "Explore"
        }
    ]
}
```

这个计划将帮助我们分析代码。
"#;

        let plan = Plan::from_llm_response(markdown_response).unwrap();
        assert_eq!(plan.description, "代码分析计划");
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(
            plan.tasks[0].execution_type,
            ExecutionType::SubagentDelegation(AgentType::Explore)
        );
    }

    #[test]
    fn test_observation_analysis() {
        let mut analysis = ObservationAnalysis::new();
        analysis.total_actions = 10;
        analysis.successful = 8;
        analysis.failed = 2;
        analysis.add_finding("发现了关键函数".to_string());
        analysis.add_blocker("缺少依赖".to_string());

        assert_eq!(analysis.success_rate(), 0.8);
        assert!(analysis.has_blockers());
        assert_eq!(analysis.key_findings.len(), 1);
        assert_eq!(analysis.blockers.len(), 1);
    }
}
