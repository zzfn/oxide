//! 命令执行工具: Bash, TaskOutput, TaskStop

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::registry::{Tool, ToolResult, ToolSchema};
use crate::task::{BackgroundTask, TaskManager};

/// Bash 工具参数
#[derive(Debug, Deserialize)]
struct BashParams {
    command: String,
    #[serde(default)]
    _description: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
    #[serde(default)]
    run_in_background: bool,
}

/// TaskOutput 工具参数
#[derive(Debug, Deserialize)]
struct TaskOutputParams {
    task_id: String,
    #[serde(default = "default_true")]
    block: bool,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    120000 // 2 分钟
}

/// TaskStop 工具参数
#[derive(Debug, Deserialize)]
struct TaskStopParams {
    task_id: String,
}

/// Bash 工具 - 命令执行
pub struct BashTool {
    working_dir: PathBuf,
    task_manager: TaskManager,
}

impl BashTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            task_manager: crate::create_task_manager(),
        }
    }

    pub fn with_task_manager(working_dir: PathBuf, task_manager: TaskManager) -> Self {
        Self {
            working_dir,
            task_manager,
        }
    }

    /// 获取任务管理器（用于创建 TaskOutput 和 TaskStop 工具）
    pub fn task_manager(&self) -> TaskManager {
        self.task_manager.clone()
    }

    /// 执行命令（前台）
    async fn execute_foreground(
        &self,
        command: &str,
        timeout_ms: u64,
    ) -> anyhow::Result<(String, i32)> {
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let timeout_duration = Duration::from_millis(timeout_ms);

        let result = timeout(timeout_duration, async {
            let mut child = cmd.spawn()?;

            // 读取 stdout 和 stderr
            let stdout = child.stdout.take().expect("Failed to capture stdout");
            let stderr = child.stderr.take().expect("Failed to capture stderr");

            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            let mut output = String::new();

            // 交替读取 stdout 和 stderr
            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                output.push_str(&line);
                                output.push('\n');
                            }
                            Ok(None) => break,
                            Err(e) => return Err(anyhow::anyhow!("读取 stdout 失败: {}", e)),
                        }
                    }
                    line = stderr_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                output.push_str(&line);
                                output.push('\n');
                            }
                            Ok(None) => {},
                            Err(e) => return Err(anyhow::anyhow!("读取 stderr 失败: {}", e)),
                        }
                    }
                }
            }

            let status = child.wait().await?;
            let exit_code = status.code().unwrap_or(-1);

            Ok::<(String, i32), anyhow::Error>((output, exit_code))
        })
        .await;

        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("命令执行超时 ({}ms)", timeout_ms)),
        }
    }

    /// 执行命令（后台）
    async fn execute_background(&self, command: &str) -> anyhow::Result<String> {
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

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "Bash"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "Bash".to_string(),
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

    async fn execute(&self, input: Value) -> anyhow::Result<ToolResult> {
        let params: BashParams = serde_json::from_value(input)?;

        if params.run_in_background {
            // 后台执行
            match self.execute_background(&params.command).await {
                Ok(task_id) => Ok(ToolResult::success(format!(
                    "✓ 后台任务已启动\n  任务 ID: {}\n  命令: {}\n\n使用 TaskOutput 工具查看输出",
                    task_id, params.command
                ))),
                Err(e) => Ok(ToolResult::error(format!("启动后台任务失败: {}", e))),
            }
        } else {
            // 前台执行
            let timeout_ms = params.timeout.unwrap_or(120000);

            match self.execute_foreground(&params.command, timeout_ms).await {
                Ok((output, exit_code)) => {
                    let status = if exit_code == 0 { "✓" } else { "✗" };
                    let output_preview = if output.len() > 5000 {
                        format!("{}... (输出过长，已截断)", &output[..5000])
                    } else {
                        output
                    };

                    Ok(ToolResult::success(format!(
                        "{} 命令执行完成 (退出码: {})\n命令: {}\n\n输出:\n{}",
                        status, exit_code, params.command, output_preview
                    )))
                }
                Err(e) => Ok(ToolResult::error(format!("命令执行失败: {}", e))),
            }
        }
    }
}

/// TaskOutput 工具 - 获取后台任务输出
pub struct TaskOutputTool {
    task_manager: TaskManager,
}

impl TaskOutputTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

#[async_trait]
impl Tool for TaskOutputTool {
    fn name(&self) -> &str {
        "TaskOutput"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "TaskOutput".to_string(),
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

    async fn execute(&self, input: Value) -> anyhow::Result<ToolResult> {
        let params: TaskOutputParams = serde_json::from_value(input)?;

        if params.block {
            // 等待任务完成
            let timeout_duration = Duration::from_millis(params.timeout);
            let start = std::time::Instant::now();

            loop {
                let task = self.task_manager.get_background_task(&params.task_id).await;

                match task {
                    Some(task) if !task.is_running => {
                        let status = if task.exit_code == Some(0) {
                            "✓ 成功"
                        } else {
                            "✗ 失败"
                        };

                        return Ok(ToolResult::success(format!(
                            "任务完成: {}\n退出码: {:?}\n\n输出:\n{}",
                            status,
                            task.exit_code.unwrap_or(-1),
                            task.output
                        )));
                    }
                    Some(_) => {
                        // 任务仍在运行
                        if start.elapsed() > timeout_duration {
                            return Ok(ToolResult::error("等待任务完成超时".to_string()));
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    None => {
                        return Ok(ToolResult::error(format!(
                            "任务不存在: {}",
                            params.task_id
                        )));
                    }
                }
            }
        } else {
            // 立即返回当前状态
            match self.task_manager.get_background_task(&params.task_id).await {
                Some(task) => {
                    let status = if task.is_running {
                        "运行中"
                    } else if task.exit_code == Some(0) {
                        "✓ 成功"
                    } else {
                        "✗ 失败"
                    };

                    Ok(ToolResult::success(format!(
                        "任务状态: {}\n退出码: {:?}\n\n当前输出:\n{}",
                        status,
                        task.exit_code.unwrap_or(-1),
                        task.output
                    )))
                }
                None => Ok(ToolResult::error(format!(
                    "任务不存在: {}",
                    params.task_id
                ))),
            }
        }
    }
}

/// TaskStop 工具 - 停止后台任务
pub struct TaskStopTool {
    task_manager: TaskManager,
}

impl TaskStopTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

#[async_trait]
impl Tool for TaskStopTool {
    fn name(&self) -> &str {
        "TaskStop"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "TaskStop".to_string(),
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

    async fn execute(&self, input: Value) -> anyhow::Result<ToolResult> {
        let params: TaskStopParams = serde_json::from_value(input)?;

        match self.task_manager.get_background_task(&params.task_id).await {
            Some(task) if task.is_running => {
                // 注意：这里只是标记任务为停止，实际的进程终止需要更复杂的实现
                self.task_manager.update_background_task(&params.task_id, |task| {
                    task.is_running = false;
                    task.exit_code = Some(-1);
                }).await;
                Ok(ToolResult::success(format!(
                    "✓ 任务已标记为停止: {}",
                    params.task_id
                )))
            }
            Some(_) => Ok(ToolResult::error("任务已经停止".to_string())),
            None => Ok(ToolResult::error(format!(
                "任务不存在: {}",
                params.task_id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_bash_tool_simple_command() {
        let temp_dir = TempDir::new().unwrap();
        let tool = BashTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "command": "echo 'Hello, World!'"
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert!(result.content.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_bash_tool_with_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let tool = BashTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "command": "sleep 5",
                "timeout": 100
            }))
            .await
            .unwrap();

        assert!(result.is_error);
        assert!(result.content.contains("超时"));
    }

    #[tokio::test]
    async fn test_bash_tool_background() {
        let temp_dir = TempDir::new().unwrap();
        let tool = BashTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "command": "echo 'background task'",
                "run_in_background": true
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert!(result.content.contains("任务 ID"));
    }
}
