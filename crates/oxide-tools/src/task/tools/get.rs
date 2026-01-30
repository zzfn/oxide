//! TaskGet 工具 - 获取任务详情

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::task::{TaskError, TaskManager};

/// TaskGet 工具参数
#[derive(Debug, Deserialize)]
pub struct TaskGetArgs {
    /// 任务 ID
    #[serde(rename = "taskId")]
    pub task_id: String,
}

/// TaskGet 工具输出
#[derive(Debug, Serialize)]
pub struct TaskGetOutput {
    /// 任务 ID
    pub id: String,
    /// 任务标题
    pub subject: String,
    /// 详细描述
    pub description: String,
    /// 进行中显示文本
    pub active_form: Option<String>,
    /// 任务状态
    pub status: String,
    /// 任务所有者
    pub owner: Option<String>,
    /// 此任务阻塞的任务列表
    pub blocks: Vec<String>,
    /// 阻塞此任务的任务列表
    pub blocked_by: Vec<String>,
    /// 元数据
    pub metadata: serde_json::Value,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// TaskGet 工具 - 获取任务详情
#[derive(Clone)]
pub struct RigTaskGetTool {
    task_manager: TaskManager,
}

impl RigTaskGetTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl Tool for RigTaskGetTool {
    const NAME: &'static str = "TaskGet";

    type Error = TaskError;
    type Args = TaskGetArgs;
    type Output = TaskGetOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取任务的完整详情，包括描述、元数据、依赖关系等所有信息。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "taskId": {
                        "type": "string",
                        "description": "任务 ID"
                    }
                },
                "required": ["taskId"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let task = self.task_manager.get_task(&args.task_id).await?;

        let status = match task.status {
            crate::task::TaskStatus::Pending => "pending",
            crate::task::TaskStatus::InProgress => "in_progress",
            crate::task::TaskStatus::Completed => "completed",
            crate::task::TaskStatus::Deleted => "deleted",
        };

        Ok(TaskGetOutput {
            id: task.id,
            subject: task.subject,
            description: task.description,
            active_form: task.active_form,
            status: status.to_string(),
            owner: task.owner,
            blocks: task.blocks,
            blocked_by: task.blocked_by,
            metadata: serde_json::to_value(task.metadata).unwrap_or(json!({})),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_get_tool() {
        let manager = TaskManager::new();

        let task_id = manager
            .create_task(
                "测试任务".to_string(),
                "这是一个测试任务的详细描述".to_string(),
                Some("正在测试".to_string()),
            )
            .await
            .unwrap();

        let tool = RigTaskGetTool::new(manager);

        let result = tool
            .call(TaskGetArgs {
                task_id: task_id.clone(),
            })
            .await
            .unwrap();

        assert_eq!(result.id, task_id);
        assert_eq!(result.subject, "测试任务");
        assert_eq!(result.description, "这是一个测试任务的详细描述");
        assert_eq!(result.active_form, Some("正在测试".to_string()));
        assert_eq!(result.status, "pending");
    }

    #[tokio::test]
    async fn test_task_get_not_found() {
        let manager = TaskManager::new();
        let tool = RigTaskGetTool::new(manager);

        let result = tool
            .call(TaskGetArgs {
                task_id: "nonexistent".to_string(),
            })
            .await;

        assert!(result.is_err());
    }
}
