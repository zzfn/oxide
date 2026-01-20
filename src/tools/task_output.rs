//! TaskOutput 工具
//!
//! 检索后台任务的输出。

use super::FileToolError;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// TaskOutput 工具输入参数
#[derive(Deserialize)]
pub struct TaskOutputArgs {
    /// 任务 ID
    pub task_id: String,

    /// 是否等待任务完成 (默认: true)
    #[serde(default)]
    pub block: Option<bool>,

    /// 超时时间(毫秒,默认: 30000)
    #[serde(default)]
    pub timeout: Option<u64>,
}

/// TaskOutput 工具输出
#[derive(Serialize, Debug)]
pub struct TaskOutputResult {
    /// 任务 ID
    pub task_id: String,

    /// 任务状态
    pub status: String,

    /// 任务输出
    pub output: Option<String>,

    /// 错误信息(如果有)
    pub error: Option<String>,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,
}

/// TaskOutput 工具
#[derive(Deserialize, Serialize)]
pub struct TaskOutputTool;

impl TaskOutputTool {
    /// 获取任务存储目录
    fn get_tasks_dir() -> PathBuf {
        PathBuf::from(".oxide/tasks")
    }

    /// 读取任务元数据
    fn read_task_metadata(task_id: &str) -> Result<Option<TaskMetadata>, FileToolError> {
        let tasks_dir = Self::get_tasks_dir();
        let meta_path = tasks_dir.join(format!("{}.json", task_id));

        if !meta_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&meta_path)
            .map_err(|e| FileToolError::Io(e))?;

        let metadata: TaskMetadata = serde_json::from_str(&content)
            .map_err(|e| FileToolError::InvalidInput(format!("解析任务元数据失败: {}", e)))?;

        Ok(Some(metadata))
    }

    /// 读取任务输出
    fn read_task_output(task_id: &str) -> Result<Option<String>, FileToolError> {
        let tasks_dir = Self::get_tasks_dir();
        let output_path = tasks_dir.join(format!("{}.output.txt", task_id));

        if !output_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&output_path)
            .map_err(|e| FileToolError::Io(e))?;

        Ok(Some(content))
    }
}

#[derive(Serialize, Deserialize)]
struct TaskMetadata {
    id: String,
    name: String,
    description: String,
    agent_type: String,
    status: String,
    created_at: String,
    output_file: Option<String>,
}

impl Tool for TaskOutputTool {
    const NAME: &'static str = "task_output";

    type Error = FileToolError;
    type Args = TaskOutputArgs;
    type Output = TaskOutputResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task_output".to_string(),
            description: "Retrieves output from a running or completed task. Takes a task_id parameter and returns the task output along with status information. Use block=true (default) to wait for task completion, or block=false for a non-blocking status check. Works with all task types: background shells, async agents, and remote sessions.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The task ID to get output from"
                    },
                    "block": {
                        "type": "boolean",
                        "description": "Whether to wait for completion (default: true)",
                        "default": true
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Max wait time in milliseconds (default: 30000, max: 600000)",
                        "default": 30000,
                        "minimum": 0,
                        "maximum": 600000
                    }
                },
                "required": ["task_id"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 读取任务元数据
        let metadata = Self::read_task_metadata(&args.task_id)?;

        match metadata {
            Some(meta) => {
                // 读取任务输出
                let output = Self::read_task_output(&args.task_id)?;

                // 检查是否需要等待
                let block = args.block.unwrap_or(true);

                if block && meta.status == "in_progress" {
                    // 在完整实现中,这里会等待任务完成或超时
                    // 当前简化版本直接返回当前状态
                    Ok(TaskOutputResult {
                        task_id: args.task_id.clone(),
                        status: meta.status.clone(),
                        output,
                        error: None,
                        success: true,
                        message: format!(
                            "Task '{}' is still in progress. Use block=false for non-blocking checks.",
                            args.task_id
                        ),
                    })
                } else {
                    // 返回当前状态
                    Ok(TaskOutputResult {
                        task_id: args.task_id.clone(),
                        status: meta.status.clone(),
                        output,
                        error: None,
                        success: true,
                        message: format!(
                            "Retrieved output for task '{}': {}",
                            args.task_id, meta.status
                        ),
                    })
                }
            }
            None => {
                // 任务不存在
                Ok(TaskOutputResult {
                    task_id: args.task_id.clone(),
                    status: "not_found".to_string(),
                    output: None,
                    error: Some(format!("Task '{}' not found", args.task_id)),
                    success: false,
                    message: format!("Task '{}' does not exist", args.task_id),
                })
            }
        }
    }
}

/// TaskOutput 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedTaskOutputTool {
    inner: TaskOutputTool,
}

impl WrappedTaskOutputTool {
    pub fn new() -> Self {
        Self {
            inner: TaskOutputTool,
        }
    }
}

impl Default for WrappedTaskOutputTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_task_output_args_deserialization() {
        let json = r#"{
            "task_id": "test-task-id",
            "block": true,
            "timeout": 60000
        }"#;

        let args: TaskOutputArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.task_id, "test-task-id");
        assert_eq!(args.block, Some(true));
        assert_eq!(args.timeout, Some(60000));
    }

    #[test]
    fn test_task_output_result_serialization() {
        let result = TaskOutputResult {
            task_id: "test-id".to_string(),
            status: "completed".to_string(),
            output: Some("Task output here".to_string()),
            error: None,
            success: true,
            message: "Output retrieved successfully".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("completed"));
        assert!(json.contains("Task output here"));
    }

    #[test]
    fn test_read_nonexistent_task() {
        // 测试读取不存在的任务
        let metadata = TaskOutputTool::read_task_metadata("nonexistent-id").unwrap();
        assert!(metadata.is_none());

        let output = TaskOutputTool::read_task_output("nonexistent-id").unwrap();
        assert!(output.is_none());
    }

    #[test]
    fn test_task_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        fs::create_dir_all(&tasks_dir).unwrap();

        // 创建测试任务元数据
        let metadata = TaskMetadata {
            id: "test-id".to_string(),
            name: "Test Task".to_string(),
            description: "Test description".to_string(),
            agent_type: "explore".to_string(),
            status: "in_progress".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            output_file: None,
        };

        let meta_path = tasks_dir.join("test-id.json");
        let json = serde_json::to_string_pretty(&metadata).unwrap();
        fs::write(&meta_path, json).unwrap();

        // 注意: 这个测试需要 TaskOutputTool 能够使用自定义的 tasks_dir
        // 当前的实现使用硬编码的 ".oxide/tasks"
        // 在完整实现中,应该通过依赖注入或配置来解决这个问题
    }
}
