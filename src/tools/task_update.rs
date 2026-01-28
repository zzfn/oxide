//! TaskUpdate 工具
//!
//! 更新任务列表中的任务。

use super::FileToolError;
use crate::task::manager::{get_task_manager, TaskStatus};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TaskUpdate 工具输入参数
#[derive(Deserialize)]
pub struct TaskUpdateArgs {
    /// 任务 ID（必需）
    #[serde(rename = "taskId")]
    pub task_id: String,

    /// 新状态（可选）
    #[serde(default)]
    pub status: Option<String>,

    /// 新标题（可选）
    #[serde(default)]
    pub subject: Option<String>,

    /// 新描述（可选）
    #[serde(default)]
    pub description: Option<String>,

    /// 进行中显示文本（可选）
    #[serde(default, rename = "activeForm")]
    pub active_form: Option<String>,

    /// 任务所有者（可选）
    #[serde(default)]
    pub owner: Option<String>,

    /// 添加阻塞的任务 ID（可选）
    #[serde(default, rename = "addBlocks")]
    pub add_blocks: Option<Vec<String>>,

    /// 添加被阻塞的任务 ID（可选）
    #[serde(default, rename = "addBlockedBy")]
    pub add_blocked_by: Option<Vec<String>>,

    /// 元数据更新（可选，设置为 null 可删除键）
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// TaskUpdate 工具输出
#[derive(Serialize, Debug)]
pub struct TaskUpdateOutput {
    /// 任务 ID
    pub task_id: String,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,
}

/// TaskUpdate 工具
#[derive(Deserialize, Serialize)]
pub struct TaskUpdateTool;

impl TaskUpdateTool {
    /// 解析状态字符串
    fn parse_status(status: &str) -> Option<TaskStatus> {
        match status.to_lowercase().as_str() {
            "pending" => Some(TaskStatus::Pending),
            "in_progress" => Some(TaskStatus::InProgress),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            "deleted" => Some(TaskStatus::Deleted),
            _ => None,
        }
    }
}

impl Tool for TaskUpdateTool {
    const NAME: &'static str = "task_update";

    type Error = FileToolError;
    type Args = TaskUpdateArgs;
    type Output = TaskUpdateOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_update".to_string(),
            description: r#"Use this tool to update a task in the task list.

## When to Use This Tool

**Mark tasks as resolved:**
- When you have completed the work described in a task
- When a task is no longer needed or has been superseded
- IMPORTANT: Always mark your assigned tasks as resolved when you finish them

**Delete tasks:**
- When a task is no longer relevant or was created in error
- Setting status to `deleted` permanently removes the task

**Update task details:**
- When requirements change or become clearer
- When establishing dependencies between tasks

## Fields You Can Update

- **status**: pending, in_progress, completed, failed, deleted
- **subject**: Change the task title
- **description**: Change the task description
- **activeForm**: Present continuous form shown when in_progress
- **owner**: Change the task owner
- **addBlocks**: Mark tasks that cannot start until this one completes
- **addBlockedBy**: Mark tasks that must complete before this one can start
- **metadata**: Merge metadata keys (set a key to null to delete it)

## Status Workflow

Status progresses: `pending` → `in_progress` → `completed`

ONLY mark a task as completed when you have FULLY accomplished it."#.to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "taskId": {
                        "type": "string",
                        "description": "The ID of the task to update"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["pending", "in_progress", "completed", "failed", "deleted"],
                        "description": "New status for the task"
                    },
                    "subject": {
                        "type": "string",
                        "description": "New subject for the task"
                    },
                    "description": {
                        "type": "string",
                        "description": "New description for the task"
                    },
                    "activeForm": {
                        "type": "string",
                        "description": "Present continuous form shown when in_progress"
                    },
                    "owner": {
                        "type": "string",
                        "description": "New owner for the task"
                    },
                    "addBlocks": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Task IDs that this task blocks"
                    },
                    "addBlockedBy": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Task IDs that block this task"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Metadata keys to merge into the task. Set a key to null to delete it.",
                        "additionalProperties": true
                    }
                },
                "required": ["taskId"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let manager = get_task_manager();

        // 检查任务是否存在
        let task = match manager.get_task(&args.task_id) {
            Ok(Some(t)) => t,
            Ok(None) => {
                return Ok(TaskUpdateOutput {
                    task_id: args.task_id.clone(),
                    success: false,
                    message: format!("Task '{}' not found", args.task_id),
                });
            }
            Err(e) => {
                return Ok(TaskUpdateOutput {
                    task_id: args.task_id.clone(),
                    success: false,
                    message: format!("Failed to get task: {}", e),
                });
            }
        };

        // 处理状态更新
        if let Some(status_str) = &args.status {
            if let Some(status) = Self::parse_status(status_str) {
                if let Err(e) = manager.update_task_status(&args.task_id, status) {
                    return Ok(TaskUpdateOutput {
                        task_id: args.task_id.clone(),
                        success: false,
                        message: format!("Failed to update status: {}", e),
                    });
                }
            } else {
                return Ok(TaskUpdateOutput {
                    task_id: args.task_id.clone(),
                    success: false,
                    message: format!("Invalid status: '{}'", status_str),
                });
            }
        }

        // 处理其他字段更新
        let update_result = manager.update_task(&args.task_id, |task| {
            if let Some(subject) = &args.subject {
                task.subject = subject.clone();
                task.name = subject.clone();
            }
            if let Some(description) = &args.description {
                task.description = description.clone();
            }
            if let Some(active_form) = &args.active_form {
                task.active_form = Some(active_form.clone());
            }
            if let Some(owner) = &args.owner {
                task.owner = Some(owner.clone());
            }
            // 处理元数据更新
            if let Some(metadata) = &args.metadata {
                for (key, value) in metadata {
                    if value.is_null() {
                        task.metadata.remove(key);
                    } else {
                        task.metadata.insert(key.clone(), value.clone());
                    }
                }
            }
        });

        if let Err(e) = update_result {
            return Ok(TaskUpdateOutput {
                task_id: args.task_id.clone(),
                success: false,
                message: format!("Failed to update task: {}", e),
            });
        }

        // 处理依赖关系
        if let Some(blocks) = &args.add_blocks {
            for blocked_id in blocks {
                if let Err(e) = manager.add_blocks(&args.task_id, blocked_id) {
                    return Ok(TaskUpdateOutput {
                        task_id: args.task_id.clone(),
                        success: false,
                        message: format!("Failed to add blocks dependency: {}", e),
                    });
                }
            }
        }

        if let Some(blocked_by) = &args.add_blocked_by {
            for blocking_id in blocked_by {
                if let Err(e) = manager.add_blocked_by(&args.task_id, blocking_id) {
                    return Ok(TaskUpdateOutput {
                        task_id: args.task_id.clone(),
                        success: false,
                        message: format!("Failed to add blockedBy dependency: {}", e),
                    });
                }
            }
        }

        Ok(TaskUpdateOutput {
            task_id: args.task_id.clone(),
            success: true,
            message: format!("Task '{}' updated successfully", task.subject),
        })
    }
}

/// TaskUpdate 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedTaskUpdateTool {
    inner: TaskUpdateTool,
}

impl WrappedTaskUpdateTool {
    pub fn new() -> Self {
        Self {
            inner: TaskUpdateTool,
        }
    }
}

impl Default for WrappedTaskUpdateTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedTaskUpdateTool {
    const NAME: &'static str = "task_update";

    type Error = FileToolError;
    type Args = TaskUpdateArgs;
    type Output = TaskUpdateOutput;

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
    fn test_task_update_args_deserialization() {
        let json = r#"{
            "taskId": "test-id",
            "status": "in_progress",
            "owner": "agent-1"
        }"#;

        let args: TaskUpdateArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.task_id, "test-id");
        assert_eq!(args.status, Some("in_progress".to_string()));
        assert_eq!(args.owner, Some("agent-1".to_string()));
    }

    #[test]
    fn test_task_update_with_dependencies() {
        let json = r#"{
            "taskId": "task-2",
            "addBlockedBy": ["task-1"]
        }"#;

        let args: TaskUpdateArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.task_id, "task-2");
        assert_eq!(args.add_blocked_by, Some(vec!["task-1".to_string()]));
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(
            TaskUpdateTool::parse_status("pending"),
            Some(TaskStatus::Pending)
        );
        assert_eq!(
            TaskUpdateTool::parse_status("in_progress"),
            Some(TaskStatus::InProgress)
        );
        assert_eq!(
            TaskUpdateTool::parse_status("completed"),
            Some(TaskStatus::Completed)
        );
        assert_eq!(
            TaskUpdateTool::parse_status("deleted"),
            Some(TaskStatus::Deleted)
        );
        assert_eq!(TaskUpdateTool::parse_status("invalid"), None);
    }
}
