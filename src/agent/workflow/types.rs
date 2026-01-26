//! 工作流类型定义
//!
//! 定义任务、计划、观察和反思等核心数据结构。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

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
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
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
        assert!(!task.is_finished());
        
        task.mark_started();
        assert_eq!(task.status, TaskStatus::Running);
        assert!(!task.is_finished());
        
        task.mark_completed("Success".to_string());
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.is_finished());
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
}
