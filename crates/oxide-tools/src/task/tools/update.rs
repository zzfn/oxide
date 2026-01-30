//! TaskUpdate 工具 - 更新任务

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::task::{TaskError, TaskManager, TaskStatus};

/// TaskUpdate 工具参数
#[derive(Debug, Deserialize)]
pub struct TaskUpdateArgs {
    /// 任务 ID
    #[serde(rename = "taskId")]
    pub task_id: String,
    /// 新状态
    #[serde(default)]
    pub status: Option<String>,
    /// 新标题
    #[serde(default)]
    pub subject: Option<String>,
    /// 新描述
    #[serde(default)]
    pub description: Option<String>,
    /// 新的进行中显示文本
    #[serde(default, rename = "activeForm")]
    pub active_form: Option<String>,
    /// 新所有者
    #[serde(default)]
    pub owner: Option<String>,
    /// 添加阻塞关系（此任务阻塞的任务）
    #[serde(default, rename = "addBlocks")]
    pub add_blocks: Option<Vec<String>>,
    /// 添加被阻塞关系（阻塞此任务的任务）
    #[serde(default, rename = "addBlockedBy")]
    pub add_blocked_by: Option<Vec<String>>,
    /// 元数据更新（设置为 null 的键会被删除）
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// TaskUpdate 工具输出
#[derive(Debug, Serialize)]
pub struct TaskUpdateOutput {
    /// 任务 ID
    pub task_id: String,
    /// 更新的字段
    pub updated_fields: Vec<String>,
    /// 成功消息
    pub message: String,
}

/// TaskUpdate 工具 - 更新任务
#[derive(Clone)]
pub struct RigTaskUpdateTool {
    task_manager: TaskManager,
}

impl RigTaskUpdateTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl Tool for RigTaskUpdateTool {
    const NAME: &'static str = "TaskUpdate";

    type Error = TaskError;
    type Args = TaskUpdateArgs;
    type Output = TaskUpdateOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "更新任务的状态、内容、所有者、依赖关系或元数据。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "taskId": {
                        "type": "string",
                        "description": "任务 ID"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["pending", "in_progress", "completed", "deleted"],
                        "description": "新状态"
                    },
                    "subject": {
                        "type": "string",
                        "description": "新标题"
                    },
                    "description": {
                        "type": "string",
                        "description": "新描述"
                    },
                    "activeForm": {
                        "type": "string",
                        "description": "新的进行中显示文本"
                    },
                    "owner": {
                        "type": "string",
                        "description": "新所有者"
                    },
                    "addBlocks": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "添加此任务阻塞的任务 ID 列表"
                    },
                    "addBlockedBy": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "添加阻塞此任务的任务 ID 列表"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "元数据更新（设置为 null 的键会被删除）"
                    }
                },
                "required": ["taskId"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut updated_fields = Vec::new();

        // 更新状态
        if let Some(status_str) = &args.status {
            let status = match status_str.as_str() {
                "pending" => TaskStatus::Pending,
                "in_progress" => TaskStatus::InProgress,
                "completed" => TaskStatus::Completed,
                "deleted" => TaskStatus::Deleted,
                _ => {
                    return Err(TaskError::Other(format!(
                        "无效的状态: {}",
                        status_str
                    )))
                }
            };

            self.task_manager
                .update_task_status(&args.task_id, status)
                .await?;
            updated_fields.push("status".to_string());
        }

        // 更新内容
        if args.subject.is_some() || args.description.is_some() || args.active_form.is_some() {
            self.task_manager
                .update_task_content(
                    &args.task_id,
                    args.subject.clone(),
                    args.description.clone(),
                    args.active_form.clone(),
                )
                .await?;

            if args.subject.is_some() {
                updated_fields.push("subject".to_string());
            }
            if args.description.is_some() {
                updated_fields.push("description".to_string());
            }
            if args.active_form.is_some() {
                updated_fields.push("activeForm".to_string());
            }
        }

        // 更新所有者
        if args.owner.is_some() {
            self.task_manager
                .update_task_owner(&args.task_id, args.owner.clone())
                .await?;
            updated_fields.push("owner".to_string());
        }

        // 添加依赖关系
        if args.add_blocks.is_some() || args.add_blocked_by.is_some() {
            let blocks = args.add_blocks.unwrap_or_default();
            let blocked_by = args.add_blocked_by.unwrap_or_default();

            self.task_manager
                .add_dependency(&args.task_id, blocks.clone(), blocked_by.clone())
                .await?;

            if !blocks.is_empty() {
                updated_fields.push("blocks".to_string());
            }
            if !blocked_by.is_empty() {
                updated_fields.push("blockedBy".to_string());
            }
        }

        // 更新元数据
        if let Some(metadata) = args.metadata {
            self.task_manager
                .update_task_metadata(&args.task_id, metadata)
                .await?;
            updated_fields.push("metadata".to_string());
        }

        let message = if updated_fields.is_empty() {
            format!("No changes made to task #{}", args.task_id)
        } else {
            format!(
                "Updated task #{} {}",
                args.task_id,
                updated_fields.join(", ")
            )
        };

        Ok(TaskUpdateOutput {
            task_id: args.task_id,
            updated_fields,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_update_status() {
        let manager = TaskManager::new();

        let task_id = manager
            .create_task("测试任务".to_string(), "描述".to_string(), None)
            .await
            .unwrap();

        let tool = RigTaskUpdateTool::new(manager.clone());

        let result = tool
            .call(TaskUpdateArgs {
                task_id: task_id.clone(),
                status: Some("in_progress".to_string()),
                subject: None,
                description: None,
                active_form: None,
                owner: None,
                add_blocks: None,
                add_blocked_by: None,
                metadata: None,
            })
            .await
            .unwrap();

        assert!(result.updated_fields.contains(&"status".to_string()));

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_task_update_dependencies() {
        let manager = TaskManager::new();

        let task1 = manager
            .create_task("任务1".to_string(), "描述1".to_string(), None)
            .await
            .unwrap();
        let task2 = manager
            .create_task("任务2".to_string(), "描述2".to_string(), None)
            .await
            .unwrap();

        let tool = RigTaskUpdateTool::new(manager.clone());

        // task2 依赖 task1
        tool.call(TaskUpdateArgs {
            task_id: task2.clone(),
            status: None,
            subject: None,
            description: None,
            active_form: None,
            owner: None,
            add_blocks: None,
            add_blocked_by: Some(vec![task1.clone()]),
            metadata: None,
        })
        .await
        .unwrap();

        let task = manager.get_task(&task2).await.unwrap();
        assert_eq!(task.blocked_by.len(), 1);
        assert_eq!(task.blocked_by[0], task1);
    }

    #[tokio::test]
    async fn test_task_update_metadata() {
        let manager = TaskManager::new();

        let task_id = manager
            .create_task("测试".to_string(), "描述".to_string(), None)
            .await
            .unwrap();

        let tool = RigTaskUpdateTool::new(manager.clone());

        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), json!("high"));
        metadata.insert("assignee".to_string(), json!("Alice"));

        tool.call(TaskUpdateArgs {
            task_id: task_id.clone(),
            status: None,
            subject: None,
            description: None,
            active_form: None,
            owner: None,
            add_blocks: None,
            add_blocked_by: None,
            metadata: Some(metadata),
        })
        .await
        .unwrap();

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.metadata.get("priority").unwrap(), "high");
        assert_eq!(task.metadata.get("assignee").unwrap(), "Alice");
    }
}
