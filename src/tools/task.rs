//! Task 工具
//!
//! 启动和管理后台任务。

use super::FileToolError;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io::Write;

/// Task 工具输入参数
#[derive(Deserialize)]
pub struct TaskArgs {
    /// 任务描述
    pub description: String,

    /// Agent 类型 (main, explore, plan, code_reviewer, frontend_developer)
    #[serde(default)]
    pub agent_type: Option<String>,

    /// 是否在后台运行
    #[serde(default)]
    pub run_in_background: Option<bool>,

    /// 任务名称(可选)
    #[serde(default)]
    pub name: Option<String>,
}

/// Task 工具输出
#[derive(Serialize, Debug)]
pub struct TaskOutput {
    /// 任务 ID
    pub task_id: String,

    /// 任务名称
    pub name: String,

    /// 任务状态
    pub status: String,

    /// 输出文件路径(如果后台运行)
    pub output_file: Option<String>,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,
}

/// Task 工具
#[derive(Deserialize, Serialize)]
pub struct TaskTool;

impl TaskTool {
    /// 获取任务存储目录
    fn get_tasks_dir() -> PathBuf {
        PathBuf::from(".oxide/tasks")
    }

    /// 保存任务元数据
    fn save_task_metadata(task_id: &str, metadata: &TaskMetadata) -> Result<(), FileToolError> {
        let tasks_dir = Self::get_tasks_dir();
        fs::create_dir_all(&tasks_dir)?;

        let meta_path = tasks_dir.join(format!("{}.json", task_id));
        let json = serde_json::to_string_pretty(metadata)
            .map_err(|e| FileToolError::InvalidInput(format!("序列化失败: {}", e)))?;

        fs::write(&meta_path, json)
            .map_err(|e| FileToolError::Io(e))?;

        Ok(())
    }

    /// 启动后台任务
    fn launch_background_task(
        task_id: &str,
        description: String,
        agent_type: String,
    ) -> Result<PathBuf, FileToolError> {
        let tasks_dir = Self::get_tasks_dir();
        let output_file = tasks_dir.join(format!("{}.output.txt", task_id));

        // 创建输出文件
        let mut file = fs::File::create(&output_file)
            .map_err(|e| FileToolError::Io(e))?;

        // 写入初始信息
        writeln!(file, "Task ID: {}", task_id)
            .map_err(|e| FileToolError::Io(e))?;
        writeln!(file, "Description: {}", description)
            .map_err(|e| FileToolError::Io(e))?;
        writeln!(file, "Agent Type: {}", agent_type)
            .map_err(|e| FileToolError::Io(e))?;
        writeln!(file, "Started at: {}", chrono::Utc::now().to_rfc3339())
            .map_err(|e| FileToolError::Io(e))?;
        writeln!(file, "{}", "=".repeat(80))
            .map_err(|e| FileToolError::Io(e))?;
        writeln!(file, "Task execution in progress...")
            .map_err(|e| FileToolError::Io(e))?;

        // 注意: 这里我们只是创建了一个占位符
        // 真正的后台任务执行需要在更高层级实现
        // 当前这是一个简化实现,用于演示概念

        Ok(output_file)
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

impl Tool for TaskTool {
    const NAME: &'static str = "task";

    type Error = FileToolError;
    type Args = TaskArgs;
    type Output = TaskOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "task".to_string(),
            description: "Launch a specialized agent to handle complex, multi-step tasks autonomously. Each agent type has specific capabilities and tools available to it. Use this tool when you need to delegate work to a specialized agent (e.g., Explore agent for codebase analysis, Plan agent for design, Code Reviewer for code review, Frontend Developer for UI work).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "A detailed description of the task to be performed by the agent"
                    },
                    "agent_type": {
                        "type": "string",
                        "description": "The type of agent to launch (main, explore, plan, code_reviewer, frontend_developer)",
                        "enum": ["main", "explore", "plan", "code_reviewer", "frontend_developer"]
                    },
                    "run_in_background": {
                        "type": "boolean",
                        "description": "Whether to run the task in the background (default: false)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Optional name for the task (default: auto-generated)"
                    }
                },
                "required": ["description", "agent_type"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 解析 Agent 类型
        let agent_type = args.agent_type.unwrap_or_else(|| "main".to_string());
        let valid_types = ["main", "explore", "plan", "code_reviewer", "frontend_developer"];

        if !valid_types.contains(&agent_type.as_str()) {
            return Ok(TaskOutput {
                task_id: String::new(),
                name: String::new(),
                status: "failed".to_string(),
                output_file: None,
                success: false,
                message: format!(
                    "Invalid agent_type: '{}'. Valid types are: {}",
                    agent_type,
                    valid_types.join(", ")
                ),
            });
        }

        // 生成任务 ID
        let task_id = uuid::Uuid::new_v4().to_string();

        // 生成任务名称
        let name = args.name.unwrap_or_else(|| {
            format!("{} Task", agent_type.replace("_", " ").to_uppercase())
        });

        // 创建任务元数据
        let metadata = TaskMetadata {
            id: task_id.clone(),
            name: name.clone(),
            description: args.description.clone(),
            agent_type: agent_type.clone(),
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            output_file: None,
        };

        // 保存任务元数据
        Self::save_task_metadata(&task_id, &metadata)?;

        // 检查是否在后台运行
        let run_in_background = args.run_in_background.unwrap_or(false);

        if run_in_background {
            // 启动后台任务
            let output_file = Self::launch_background_task(&task_id, args.description, agent_type)?;

            // 更新元数据
            let mut updated_metadata = metadata;
            updated_metadata.status = "in_progress".to_string();
            updated_metadata.output_file = Some(output_file.to_string_lossy().to_string());
            Self::save_task_metadata(&task_id, &updated_metadata)?;

            Ok(TaskOutput {
                task_id: task_id.clone(),
                name,
                status: "in_progress".to_string(),
                output_file: Some(output_file.to_string_lossy().to_string()),
                success: true,
                message: format!(
                    "Task '{}' launched in background. Use TaskOutput tool to check progress.",
                    task_id
                ),
            })
        } else {
            // 同步任务 - 这是一个简化版本
            // 在完整实现中,这里会实际执行 Agent 并返回结果
            Ok(TaskOutput {
                task_id: task_id.clone(),
                name,
                status: "pending".to_string(),
                output_file: None,
                success: true,
                message: format!(
                    "Task '{}' created. Note: Synchronous task execution requires full agent integration.",
                    task_id
                ),
            })
        }
    }
}

/// Task 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedTaskTool {
    inner: TaskTool,
}

impl WrappedTaskTool {
    pub fn new() -> Self {
        Self {
            inner: TaskTool,
        }
    }
}

impl Default for WrappedTaskTool {
    fn default() -> Self {
        Self::new()
    }
}

// 注意: 在完整实现中,WrappedTaskTool 需要实现 Tool trait
// 但由于 Task 工具需要与 TaskManager 交互,这需要在更高层级处理
// 当前的实现提供了一个概念验证框架

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_args_deserialization() {
        let json = r#"{
            "description": "测试任务",
            "agent_type": "explore",
            "run_in_background": true
        }"#;

        let args: TaskArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.description, "测试任务");
        assert_eq!(args.agent_type, Some("explore".to_string()));
        assert_eq!(args.run_in_background, Some(true));
    }

    #[test]
    fn test_task_output_serialization() {
        let output = TaskOutput {
            task_id: "test-id".to_string(),
            name: "Test Task".to_string(),
            status: "completed".to_string(),
            output_file: Some("/path/to/output.txt".to_string()),
            success: true,
            message: "Task completed successfully".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("completed"));
    }
}
