//! TaskGet 工具
//!
//! 获取任务的详细信息。

use super::FileToolError;
use crate::task::manager::{get_task_manager, TaskStatus};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TaskGet 工具输入参数
#[derive(Deserialize)]
pub struct TaskGetArgs {
    /// 任务 ID
    #[serde(rename = "taskId")]
    pub task_id: String,
}

/// 任务详情
#[derive(Serialize, Debug)]
pub struct TaskDetail {
    /// 任务 ID
    pub id: String,

    /// 任务标题
    pub subject: String,

    /// 任务描述
    pub description: String,

    /// 任务状态
    pub status: String,

    /// 任务所有者
    pub owner: Option<String>,

    /// 进行中显示文本
    pub active_form: Option<String>,

    /// 阻塞的任务 ID 列表
    pub blocks: Vec<String>,

    /// 被阻塞的任务 ID 列表
    pub blocked_by: Vec<String>,

    /// 自定义元数据
    pub metadata: HashMap<String, serde_json::Value>,

    /// 创建时间
    pub created_at: String,

    /// 最后更新时间
    pub updated_at: String,
}

/// TaskGet 工具输出
#[derive(Serialize, Debug)]
pub struct TaskGetOutput {
    /// 任务详情
    pub task: Option<TaskDetail>,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,
}

/// TaskGet 工具
#[derive(Deserialize, Serialize)]
pub struct TaskGetTool;

impl TaskGetTool {
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
}

impl Tool for TaskGetTool {
    const NAME: &'static str = "task_get";

    type Error = FileToolError;
    type Args = TaskGetArgs;
    type Output = TaskGetOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_get".to_string(),
            description: r#"Use this tool to retrieve a task by its ID from the task list.

## When to Use This Tool

- When you need the full description and context before starting work on a task
- To understand task dependencies (what it blocks, what blocks it)
- After being assigned a task, to get complete requirements

## Output

Returns full task details:
- **subject**: Task title
- **description**: Detailed requirements and context
- **status**: 'pending', 'in_progress', 'completed', 'failed', or 'deleted'
- **blocks**: Tasks waiting on this one to complete
- **blockedBy**: Tasks that must complete before this one can start
- **metadata**: Custom metadata attached to the task
- **createdAt**: When the task was created
- **updatedAt**: When the task was last modified

## Tips

- After fetching a task, verify its blockedBy list is empty before beginning work.
- Use TaskList to see all tasks in summary form."#.to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "taskId": {
                        "type": "string",
                        "description": "The ID of the task to retrieve"
                    }
                },
                "required": ["taskId"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let manager = get_task_manager();

        match manager.get_task(&args.task_id) {
            Ok(Some(task)) => {
                let detail = TaskDetail {
                    id: task.id,
                    subject: task.subject,
                    description: task.description,
                    status: Self::status_to_string(task.status),
                    owner: task.owner,
                    active_form: task.active_form,
                    blocks: task.blocks,
                    blocked_by: task.blocked_by,
                    metadata: task.metadata,
                    created_at: task.created_at.to_rfc3339(),
                    updated_at: task.updated_at.to_rfc3339(),
                };

                Ok(TaskGetOutput {
                    task: Some(detail),
                    success: true,
                    message: "Task retrieved successfully".to_string(),
                })
            }
            Ok(None) => Ok(TaskGetOutput {
                task: None,
                success: false,
                message: format!("Task '{}' not found", args.task_id),
            }),
            Err(e) => Ok(TaskGetOutput {
                task: None,
                success: false,
                message: format!("Failed to get task: {}", e),
            }),
        }
    }
}

/// TaskGet 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedTaskGetTool {
    inner: TaskGetTool,
}

impl WrappedTaskGetTool {
    pub fn new() -> Self {
        Self {
            inner: TaskGetTool,
        }
    }
}

impl Default for WrappedTaskGetTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedTaskGetTool {
    const NAME: &'static str = "task_get";

    type Error = FileToolError;
    type Args = TaskGetArgs;
    type Output = TaskGetOutput;

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
    fn test_task_get_args_deserialization() {
        let json = r#"{"taskId": "test-id"}"#;

        let args: TaskGetArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.task_id, "test-id");
    }

    #[test]
    fn test_task_detail_serialization() {
        let detail = TaskDetail {
            id: "test-id".to_string(),
            subject: "Test Task".to_string(),
            description: "Test description".to_string(),
            status: "pending".to_string(),
            owner: Some("agent-1".to_string()),
            active_form: Some("Testing".to_string()),
            blocks: vec!["task-2".to_string()],
            blocked_by: vec!["task-0".to_string()],
            metadata: HashMap::new(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Task"));
        assert!(json.contains("Test description"));
        assert!(json.contains("agent-1"));
    }

    #[test]
    fn test_task_get_output_not_found() {
        let output = TaskGetOutput {
            task: None,
            success: false,
            message: "Task 'nonexistent' not found".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("not found"));
    }
}
