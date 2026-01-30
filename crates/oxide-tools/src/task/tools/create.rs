//! TaskCreate 工具 - 创建新任务

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::task::{TaskError, TaskManager};

/// TaskCreate 工具参数
#[derive(Debug, Deserialize)]
pub struct TaskCreateArgs {
    /// 任务标题（祈使句）
    pub subject: String,
    /// 详细描述
    pub description: String,
    /// 进行中显示文本（现在进行时）
    #[serde(rename = "activeForm")]
    pub active_form: Option<String>,
    /// 元数据
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// TaskCreate 工具输出
#[derive(Debug, Serialize)]
pub struct TaskCreateOutput {
    /// 任务 ID
    pub task_id: String,
    /// 任务标题
    pub subject: String,
    /// 成功消息
    pub message: String,
}

/// TaskCreate 工具 - 创建新任务
#[derive(Clone)]
pub struct RigTaskCreateTool {
    task_manager: TaskManager,
}

impl RigTaskCreateTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl Tool for RigTaskCreateTool {
    const NAME: &'static str = "TaskCreate";

    type Error = TaskError;
    type Args = TaskCreateArgs;
    type Output = TaskCreateOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "创建新任务。用于跟踪复杂的多步骤工作。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "subject": {
                        "type": "string",
                        "description": "任务标题（祈使句形式，如 '实现用户认证'）"
                    },
                    "description": {
                        "type": "string",
                        "description": "详细描述任务内容、验收标准和上下文"
                    },
                    "activeForm": {
                        "type": "string",
                        "description": "进行中显示文本（现在进行时，如 '正在实现用户认证'）"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "任务元数据（可选）"
                    }
                },
                "required": ["subject", "description"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let task_id = self
            .task_manager
            .create_task(args.subject.clone(), args.description, args.active_form)
            .await?;

        // 如果提供了元数据，更新任务
        if let Some(metadata) = args.metadata {
            if let Some(obj) = metadata.as_object() {
                let mut metadata_map = std::collections::HashMap::new();
                for (k, v) in obj {
                    metadata_map.insert(k.clone(), v.clone());
                }
                self.task_manager
                    .update_task_metadata(&task_id, metadata_map)
                    .await?;
            }
        }

        Ok(TaskCreateOutput {
            task_id: task_id.clone(),
            subject: args.subject.clone(),
            message: format!("Task #{} created successfully: {}", task_id, args.subject),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_create_tool() {
        let manager = TaskManager::new();
        let tool = RigTaskCreateTool::new(manager.clone());

        let result = tool
            .call(TaskCreateArgs {
                subject: "测试任务".to_string(),
                description: "这是一个测试任务".to_string(),
                active_form: Some("正在测试".to_string()),
                metadata: None,
            })
            .await
            .unwrap();

        assert_eq!(result.subject, "测试任务");
        assert!(!result.task_id.is_empty());

        // 验证任务已创建
        let task = manager.get_task(&result.task_id).await.unwrap();
        assert_eq!(task.subject, "测试任务");
        assert_eq!(task.description, "这是一个测试任务");
    }

    #[tokio::test]
    async fn test_task_create_with_metadata() {
        let manager = TaskManager::new();
        let tool = RigTaskCreateTool::new(manager.clone());

        let metadata = json!({
            "priority": "high",
            "tags": ["bug", "urgent"]
        });

        let result = tool
            .call(TaskCreateArgs {
                subject: "修复 Bug".to_string(),
                description: "修复登录问题".to_string(),
                active_form: None,
                metadata: Some(metadata),
            })
            .await
            .unwrap();

        let task = manager.get_task(&result.task_id).await.unwrap();
        assert_eq!(task.metadata.get("priority").unwrap(), "high");
    }
}
