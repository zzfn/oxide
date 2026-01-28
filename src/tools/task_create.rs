//! TaskCreate 工具
//!
//! 创建新的任务到任务列表中。

use super::FileToolError;
use crate::task::manager::get_task_manager;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TaskCreate 工具输入参数
#[derive(Deserialize)]
pub struct TaskCreateArgs {
    /// 任务标题（必需）
    pub subject: String,

    /// 任务详细描述（必需）
    pub description: String,

    /// 进行中显示文本（可选，如 "Running tests"）
    #[serde(default)]
    pub active_form: Option<String>,

    /// 自定义元数据（可选）
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// TaskCreate 工具输出
#[derive(Serialize, Debug)]
pub struct TaskCreateOutput {
    /// 任务 ID
    pub task_id: String,

    /// 任务标题
    pub subject: String,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,
}

/// TaskCreate 工具
#[derive(Deserialize, Serialize)]
pub struct TaskCreateTool;

impl Tool for TaskCreateTool {
    const NAME: &'static str = "task_create";

    type Error = FileToolError;
    type Args = TaskCreateArgs;
    type Output = TaskCreateOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_create".to_string(),
            description: r#"Use this tool to create a structured task list for your current coding session. This helps you track progress, organize complex tasks, and demonstrate thoroughness to the user.

## When to Use This Tool

Use this tool proactively in these scenarios:
- Complex multi-step tasks - When a task requires 3 or more distinct steps
- Non-trivial and complex tasks - Tasks that require careful planning
- User explicitly requests todo list
- User provides multiple tasks (numbered or comma-separated)

## When NOT to Use This Tool

Skip using this tool when:
- There is only a single, straightforward task
- The task is trivial and tracking provides no benefit
- The task can be completed in less than 3 trivial steps

## Task Fields

- **subject**: A brief, actionable title in imperative form (e.g., "Fix authentication bug")
- **description**: Detailed description of what needs to be done
- **activeForm**: Present continuous form shown when task is in_progress (e.g., "Fixing authentication bug")

All tasks are created with status `pending`."#.to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "subject": {
                        "type": "string",
                        "description": "A brief title for the task in imperative form"
                    },
                    "description": {
                        "type": "string",
                        "description": "A detailed description of what needs to be done"
                    },
                    "active_form": {
                        "type": "string",
                        "description": "Present continuous form shown in spinner when in_progress (e.g., 'Running tests')"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Arbitrary metadata to attach to the task",
                        "additionalProperties": true
                    }
                },
                "required": ["subject", "description"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let manager = get_task_manager();

        match manager.create_task_simple(
            args.subject.clone(),
            args.description,
            args.active_form,
            args.metadata,
        ) {
            Ok(task) => Ok(TaskCreateOutput {
                task_id: task.id,
                subject: task.subject,
                success: true,
                message: format!("Task '{}' created successfully", args.subject),
            }),
            Err(e) => Ok(TaskCreateOutput {
                task_id: String::new(),
                subject: args.subject,
                success: false,
                message: format!("Failed to create task: {}", e),
            }),
        }
    }
}

/// TaskCreate 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedTaskCreateTool {
    inner: TaskCreateTool,
}

impl WrappedTaskCreateTool {
    pub fn new() -> Self {
        Self {
            inner: TaskCreateTool,
        }
    }
}

impl Default for WrappedTaskCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedTaskCreateTool {
    const NAME: &'static str = "task_create";

    type Error = FileToolError;
    type Args = TaskCreateArgs;
    type Output = TaskCreateOutput;

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
    fn test_task_create_args_deserialization() {
        let json = r#"{
            "subject": "Fix bug",
            "description": "Fix the authentication bug",
            "active_form": "Fixing bug"
        }"#;

        let args: TaskCreateArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.subject, "Fix bug");
        assert_eq!(args.description, "Fix the authentication bug");
        assert_eq!(args.active_form, Some("Fixing bug".to_string()));
    }

    #[test]
    fn test_task_create_output_serialization() {
        let output = TaskCreateOutput {
            task_id: "test-id".to_string(),
            subject: "Test Task".to_string(),
            success: true,
            message: "Task created successfully".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Task"));
    }
}
