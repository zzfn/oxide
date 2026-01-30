//! 执行工具的 rig Tool trait 适配: Bash, TaskOutput, TaskStop

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use super::errors::ExecError;
use crate::task::TaskManager;

// ============================================================================
// Bash 工具
// ============================================================================

/// Bash 工具参数
#[derive(Debug, Deserialize)]
pub struct BashArgs {
    /// 要执行的命令
    pub command: String,
    /// 命令描述（可选）
    #[serde(default)]
    pub description: Option<String>,
    /// 超时时间（毫秒）
    #[serde(default)]
    pub timeout: Option<u64>,
    /// 是否在后台运行
    #[serde(default)]
    pub run_in_background: bool,
}

/// Bash 工具输出
#[derive(Debug, Serialize)]
pub struct BashOutput {
    /// 命令输出
    pub output: String,
    /// 退出码（前台执行时）
    pub exit_code: Option<i32>,
    /// 任务 ID（后台执行时）
    pub task_id: Option<String>,
    /// 是否成功
    pub success: bool,
}

/// Bash 工具 - 命令执行
#[derive(Clone)]
pub struct RigBashTool {
    working_dir: PathBuf,
    task_manager: TaskManager,
}

impl RigBashTool {
    pub fn new(working_dir: PathBuf, task_manager: TaskManager) -> Self {
        Self {
            working_dir,
            task_manager,
        }
    }

    async fn execute_foreground(
        &self,
        command: &str,
        timeout_ms: u64,
    ) -> Result<(String, i32), ExecError> {
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let timeout_duration = Duration::from_millis(timeout_ms);

        let result = timeout(timeout_duration, async {
            let mut child = cmd.spawn()?;

            let stdout = child.stdout.take().expect("Failed to capture stdout");
            let stderr = child.stderr.take().expect("Failed to capture stderr");

            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            let mut output = String::new();

            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                output.push_str(&line);
                                output.push('\n');
                            }
                            Ok(None) => break,
                            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                        }
                    }
                    line = stderr_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                output.push_str(&line);
                                output.push('\n');
                            }
                            Ok(None) => {},
                            Err(_) => {},
                        }
                    }
                }
            }

            let status = child.wait().await?;
            let exit_code = status.code().unwrap_or(-1);

            Ok::<(String, i32), std::io::Error>((output, exit_code))
        })
        .await;

        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(ExecError::IoError(e)),
            Err(_) => Err(ExecError::Timeout(timeout_ms)),
        }
    }

    async fn execute_background(&self, command: &str) -> Result<String, ExecError> {
        let task_id = uuid::Uuid::new_v4().to_string();

        // 创建任务记录
        self.task_manager.add_background_task(task_id.clone(), command.to_string()).await;

        // 启动后台任务
        let task_id_clone = task_id.clone();
        let command_clone = command.to_string();
        let working_dir = self.working_dir.clone();
        let task_manager = self.task_manager.clone();

        tokio::spawn(async move {
            let mut cmd = Command::new("bash");
            cmd.arg("-c")
                .arg(&command_clone)
                .current_dir(&working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    let stdout = child.stdout.take().expect("Failed to capture stdout");
                    let stderr = child.stderr.take().expect("Failed to capture stderr");

                    let mut stdout_reader = BufReader::new(stdout).lines();
                    let mut stderr_reader = BufReader::new(stderr).lines();

                    let mut output = String::new();

                    loop {
                        tokio::select! {
                            line = stdout_reader.next_line() => {
                                match line {
                                    Ok(Some(line)) => {
                                        output.push_str(&line);
                                        output.push('\n');
                                    }
                                    Ok(None) => break,
                                    Err(_) => break,
                                }
                            }
                            line = stderr_reader.next_line() => {
                                match line {
                                    Ok(Some(line)) => {
                                        output.push_str(&line);
                                        output.push('\n');
                                    }
                                    Ok(None) => {},
                                    Err(_) => {},
                                }
                            }
                        }
                    }

                    let status = child.wait().await;
                    let exit_code = status.ok().and_then(|s| s.code());

                    // 更新任务状态
                    task_manager.update_background_task(&task_id_clone, |task| {
                        task.output = output;
                        task.is_running = false;
                        task.exit_code = exit_code;
                    }).await;
                }
                Err(e) => {
                    task_manager.update_background_task(&task_id_clone, |task| {
                        task.output = format!("启动命令失败: {}", e);
                        task.is_running = false;
                        task.exit_code = Some(-1);
                    }).await;
                }
            }
        });

        Ok(task_id)
    }
}

impl Tool for RigBashTool {
    const NAME: &'static str = "Bash";

    type Error = ExecError;
    type Args = BashArgs;
    type Output = BashOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "执行 bash 命令，支持前台和后台执行".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "要执行的 bash 命令"
                    },
                    "description": {
                        "type": "string",
                        "description": "命令描述（可选）"
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "超时时间（毫秒），默认 120000 (2分钟)"
                    },
                    "run_in_background": {
                        "type": "boolean",
                        "description": "是否在后台运行，默认 false"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if args.run_in_background {
            let task_id = self.execute_background(&args.command).await?;
            Ok(BashOutput {
                output: format!("后台任务已启动，任务 ID: {}", task_id),
                exit_code: None,
                task_id: Some(task_id),
                success: true,
            })
        } else {
            let timeout_ms = args.timeout.unwrap_or(120000);
            let (output, exit_code) = self.execute_foreground(&args.command, timeout_ms).await?;
            let success = exit_code == 0;

            Ok(BashOutput {
                output,
                exit_code: Some(exit_code),
                task_id: None,
                success,
            })
        }
    }
}

// ============================================================================
// TaskOutput 工具
// ============================================================================

/// TaskOutput 工具参数
#[derive(Debug, Deserialize)]
pub struct TaskOutputArgs {
    /// 任务 ID
    pub task_id: String,
    /// 是否等待任务完成
    #[serde(default = "default_true")]
    pub block: bool,
    /// 等待超时时间（毫秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    120000
}

/// TaskOutput 工具输出
#[derive(Debug, Serialize)]
pub struct TaskOutputOutput {
    /// 任务输出
    pub output: String,
    /// 任务是否仍在运行
    pub is_running: bool,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 是否成功
    pub success: bool,
}

/// TaskOutput 工具 - 获取后台任务输出
#[derive(Clone)]
pub struct RigTaskOutputTool {
    task_manager: TaskManager,
}

impl RigTaskOutputTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl Tool for RigTaskOutputTool {
    const NAME: &'static str = "TaskOutput";

    type Error = ExecError;
    type Args = TaskOutputArgs;
    type Output = TaskOutputOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取后台任务的输出".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "任务 ID"
                    },
                    "block": {
                        "type": "boolean",
                        "description": "是否等待任务完成，默认 true"
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "等待超时时间（毫秒），默认 120000"
                    }
                },
                "required": ["task_id"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if args.block {
            let timeout_duration = Duration::from_millis(args.timeout);
            let start = std::time::Instant::now();

            loop {
                let task = self.task_manager.get_background_task(&args.task_id).await;

                match task {
                    Some(task) if !task.is_running => {
                        let success = task.exit_code == Some(0);
                        return Ok(TaskOutputOutput {
                            output: task.output,
                            is_running: false,
                            exit_code: task.exit_code,
                            success,
                        });
                    }
                    Some(_) => {
                        if start.elapsed() > timeout_duration {
                            return Err(ExecError::WaitTimeout);
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    None => {
                        return Err(ExecError::TaskNotFound(args.task_id));
                    }
                }
            }
        } else {
            match self.task_manager.get_background_task(&args.task_id).await {
                Some(task) => {
                    let success = !task.is_running && task.exit_code == Some(0);
                    Ok(TaskOutputOutput {
                        output: task.output.clone(),
                        is_running: task.is_running,
                        exit_code: task.exit_code,
                        success,
                    })
                }
                None => Err(ExecError::TaskNotFound(args.task_id)),
            }
        }
    }
}

// ============================================================================
// TaskStop 工具
// ============================================================================

/// TaskStop 工具参数
#[derive(Debug, Deserialize)]
pub struct TaskStopArgs {
    /// 任务 ID
    pub task_id: String,
}

/// TaskStop 工具输出
#[derive(Debug, Serialize)]
pub struct TaskStopOutput {
    /// 任务 ID
    pub task_id: String,
    /// 是否成功停止
    pub stopped: bool,
}

/// TaskStop 工具 - 停止后台任务
#[derive(Clone)]
pub struct RigTaskStopTool {
    task_manager: TaskManager,
}

impl RigTaskStopTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl Tool for RigTaskStopTool {
    const NAME: &'static str = "TaskStop";

    type Error = ExecError;
    type Args = TaskStopArgs;
    type Output = TaskStopOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "停止后台任务".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "任务 ID"
                    }
                },
                "required": ["task_id"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match self.task_manager.get_background_task(&args.task_id).await {
            Some(task) if task.is_running => {
                self.task_manager.update_background_task(&args.task_id, |task| {
                    task.is_running = false;
                    task.exit_code = Some(-1);
                }).await;
                Ok(TaskStopOutput {
                    task_id: args.task_id,
                    stopped: true,
                })
            }
            Some(_) => Err(ExecError::TaskAlreadyStopped),
            None => Err(ExecError::TaskNotFound(args.task_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rig_tools::create_task_manager;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rig_bash_tool_simple() {
        let temp_dir = TempDir::new().unwrap();
        let task_manager = create_task_manager();
        let tool = RigBashTool::new(temp_dir.path().to_path_buf(), task_manager);

        let result = tool
            .call(BashArgs {
                command: "echo 'Hello, World!'".to_string(),
                description: None,
                timeout: None,
                run_in_background: false,
            })
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.output.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_rig_bash_tool_background() {
        let temp_dir = TempDir::new().unwrap();
        let task_manager = create_task_manager();
        let tool = RigBashTool::new(temp_dir.path().to_path_buf(), task_manager.clone());

        let result = tool
            .call(BashArgs {
                command: "echo 'background'".to_string(),
                description: None,
                timeout: None,
                run_in_background: true,
            })
            .await
            .unwrap();

        assert!(result.task_id.is_some());

        // 等待任务完成
        tokio::time::sleep(Duration::from_millis(500)).await;

        let output_tool = RigTaskOutputTool::new(task_manager);
        let output = output_tool
            .call(TaskOutputArgs {
                task_id: result.task_id.unwrap(),
                block: false,
                timeout: 1000,
            })
            .await
            .unwrap();

        assert!(!output.is_running);
        assert!(output.output.contains("background"));
    }
}
