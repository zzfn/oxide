//! TaskList 工具
//!
//! 列出任务列表中的所有任务。

use super::FileToolError;
use crate::task::manager::{get_task_manager, Task, TaskStatus};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TaskList 工具输入参数
#[derive(Deserialize)]
pub struct TaskListArgs {
    // 无参数，列出所有任务
}

/// 任务摘要
#[derive(Serialize, Debug)]
pub struct TaskSummary {
    /// 任务 ID
    pub id: String,

    /// 任务标题
    pub subject: String,

    /// 任务状态
    pub status: String,

    /// 任务所有者
    pub owner: Option<String>,

    /// 未完成的阻塞任务 ID 列表
    pub blocked_by: Vec<String>,
}

/// TaskList 工具输出
#[derive(Serialize, Debug)]
pub struct TaskListOutput {
    /// 任务列表
    pub tasks: Vec<TaskSummary>,

    /// 任务总数
    pub total: usize,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,
}

/// TaskList 工具
#[derive(Deserialize, Serialize)]
pub struct TaskListTool;

impl TaskListTool {
    /// 将 TaskStatus 转换为字符串
    fn status_to_string(status: TaskStatus) -> String {
        match status {
            TaskStatus::Pending => "pending".to_string(),
            TaskStatus::InProgress => "in_progress".to_string(),
            TaskStatus::Completed => "completed".to_string(),
            TaskStatus::Failed => "failed".to_string(),
            TaskStatus::Deleted => "deleted".to_string(),
        }
    }

    /// 将 Task 转换为 TaskSummary
    fn task_to_summary(task: &Task, all_tasks: &HashMap<String, Task>) -> TaskSummary {
        TaskSummary {
            id: task.id.clone(),
            subject: task.subject.clone(),
            status: Self::status_to_string(task.status),
            owner: task.owner.clone(),
            blocked_by: task.get_open_blockers(all_tasks),
        }
    }
}

impl Tool for TaskListTool {
    const NAME: &'static str = "task_list";

    type Error = FileToolError;
    type Args = TaskListArgs;
    type Output = TaskListOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_list".to_string(),
            description: r#"Use this tool to list all tasks in the task list.

## When to Use This Tool

- To see what tasks are available to work on (status: 'pending', no owner, not blocked)
- To check overall progress on the project
- To find tasks that are blocked and need dependencies resolved
- After completing a task, to check for newly unblocked work or claim the next available task
- **Prefer working on tasks in ID order** (lowest ID first) when multiple tasks are available

## Output

Returns a summary of each task:
- **id**: Task identifier (use with TaskGet, TaskUpdate)
- **subject**: Brief description of the task
- **status**: 'pending', 'in_progress', 'completed', 'failed', or 'deleted'
- **owner**: Agent ID if assigned, empty if available
- **blockedBy**: List of open task IDs that must be resolved first

Use TaskGet with a specific task ID to view full details including description."#.to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let manager = get_task_manager();

        match manager.list_tasks() {
            Ok(tasks) => {
                // 构建任务映射用于计算阻塞关系
                let tasks_map: HashMap<String, Task> = tasks
                    .iter()
                    .cloned()
                    .map(|t| (t.id.clone(), t))
                    .collect();

                // 过滤掉已删除的任务，转换为摘要
                let summaries: Vec<TaskSummary> = tasks
                    .iter()
                    .filter(|t| t.status != TaskStatus::Deleted)
                    .map(|t| Self::task_to_summary(t, &tasks_map))
                    .collect();

                let total = summaries.len();

                Ok(TaskListOutput {
                    tasks: summaries,
                    total,
                    success: true,
                    message: format!("Found {} task(s)", total),
                })
            }
            Err(e) => Ok(TaskListOutput {
                tasks: Vec::new(),
                total: 0,
                success: false,
                message: format!("Failed to list tasks: {}", e),
            }),
        }
    }
}

/// TaskList 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedTaskListTool {
    inner: TaskListTool,
}

impl WrappedTaskListTool {
    pub fn new() -> Self {
        Self {
            inner: TaskListTool,
        }
    }
}

impl Default for WrappedTaskListTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedTaskListTool {
    const NAME: &'static str = "task_list";

    type Error = FileToolError;
    type Args = TaskListArgs;
    type Output = TaskListOutput;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.inner.call(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_list_args_deserialization() {
        let json = r#"{}"#;
        let _args: TaskListArgs = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_task_summary_serialization() {
        let summary = TaskSummary {
            id: "test-id".to_string(),
            subject: "Test Task".to_string(),
            status: "pending".to_string(),
            owner: None,
            blocked_by: vec!["other-task".to_string()],
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Task"));
        assert!(json.contains("pending"));
        assert!(json.contains("other-task"));
    }

    #[test]
    fn test_status_to_string() {
        assert_eq!(
            TaskListTool::status_to_string(TaskStatus::Pending),
            "pending"
        );
        assert_eq!(
            TaskListTool::status_to_string(TaskStatus::InProgress),
            "in_progress"
        );
        assert_eq!(
            TaskListTool::status_to_string(TaskStatus::Completed),
            "completed"
        );
        assert_eq!(
            TaskListTool::status_to_string(TaskStatus::Failed),
            "failed"
        );
        assert_eq!(
            TaskListTool::status_to_string(TaskStatus::Deleted),
            "deleted"
        );
    }
}
