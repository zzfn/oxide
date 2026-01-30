//! TaskList 工具 - 列出所有任务

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::task::{TaskError, TaskManager, TaskStatus};

/// TaskList 工具参数
#[derive(Debug, Deserialize)]
pub struct TaskListArgs {
    // 目前不需要参数，但保留结构以便未来扩展
}

/// 任务摘要信息
#[derive(Debug, Serialize)]
pub struct TaskSummary {
    /// 任务 ID
    pub id: String,
    /// 任务标题
    pub subject: String,
    /// 任务状态
    pub status: String,
    /// 任务所有者
    pub owner: Option<String>,
    /// 阻塞此任务的任务列表
    pub blocked_by: Vec<String>,
}

/// TaskList 工具输出
#[derive(Debug, Serialize)]
pub struct TaskListOutput {
    /// 任务列表
    pub tasks: Vec<TaskSummary>,
    /// 任务总数
    pub total: usize,
}

/// TaskList 工具 - 列出所有任务
#[derive(Clone)]
pub struct RigTaskListTool {
    task_manager: TaskManager,
}

impl RigTaskListTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl Tool for RigTaskListTool {
    const NAME: &'static str = "TaskList";

    type Error = TaskError;
    type Args = TaskListArgs;
    type Output = TaskListOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "列出所有任务的摘要信息，包括 ID、标题、状态、所有者和依赖关系。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let tasks = self.task_manager.list_tasks().await;

        let summaries: Vec<TaskSummary> = tasks
            .iter()
            .filter(|task| task.status != TaskStatus::Deleted)
            .map(|task| {
                let status = match task.status {
                    TaskStatus::Pending => "pending",
                    TaskStatus::InProgress => "in_progress",
                    TaskStatus::Completed => "completed",
                    TaskStatus::Deleted => "deleted",
                };

                TaskSummary {
                    id: task.id.clone(),
                    subject: task.subject.clone(),
                    status: status.to_string(),
                    owner: task.owner.clone(),
                    blocked_by: task.blocked_by.clone(),
                }
            })
            .collect();

        let total = summaries.len();

        Ok(TaskListOutput {
            tasks: summaries,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_list_tool() {
        let manager = TaskManager::new();

        // 创建几个任务
        manager
            .create_task("任务1".to_string(), "描述1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_task("任务2".to_string(), "描述2".to_string(), None)
            .await
            .unwrap();

        let tool = RigTaskListTool::new(manager);

        let result = tool.call(TaskListArgs {}).await.unwrap();

        assert_eq!(result.total, 2);
        assert_eq!(result.tasks.len(), 2);
        assert_eq!(result.tasks[0].subject, "任务1");
        assert_eq!(result.tasks[1].subject, "任务2");
    }

    #[tokio::test]
    async fn test_task_list_excludes_deleted() {
        let manager = TaskManager::new();

        let task_id = manager
            .create_task("任务1".to_string(), "描述1".to_string(), None)
            .await
            .unwrap();

        manager
            .create_task("任务2".to_string(), "描述2".to_string(), None)
            .await
            .unwrap();

        // 删除第一个任务
        manager
            .update_task_status(&task_id, TaskStatus::Deleted)
            .await
            .unwrap();

        let tool = RigTaskListTool::new(manager);
        let result = tool.call(TaskListArgs {}).await.unwrap();

        // 应该只返回一个任务（未删除的）
        assert_eq!(result.total, 1);
        assert_eq!(result.tasks[0].subject, "任务2");
    }
}
