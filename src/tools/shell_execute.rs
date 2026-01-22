use super::FileToolError;
use super::git_guard::GitGuard;
use super::commit_linter::CommitLinter;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize)]
pub struct ShellExecuteArgs {
    pub command: String,
}

#[derive(Serialize, Debug)]
pub struct ShellExecuteOutput {
    pub command: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct ShellExecuteTool;

impl Tool for ShellExecuteTool {
    const NAME: &'static str = "shell_execute";

    type Error = FileToolError;
    type Args = ShellExecuteArgs;
    type Output = ShellExecuteOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "shell_execute".to_string(),
            description: "Execute a shell command and return the output. Use with caution as this can modify the system.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command to execute."
                    }
                },
                "required": ["command"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let command = &args.command;

        // Execute the command using cmd on Windows or sh on Unix
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command]).output()
        } else {
            Command::new("sh").args(["-c", command]).output()
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let success = output.status.success();
                let exit_code = output.status.code();

                Ok(ShellExecuteOutput {
                    command: command.clone(),
                    success,
                    stdout,
                    stderr,
                    exit_code,
                })
            }
            Err(e) => Err(FileToolError::Io(e)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WrappedShellExecuteTool {
    inner: ShellExecuteTool,
}

impl WrappedShellExecuteTool {
    pub fn new() -> Self {
        Self {
            inner: ShellExecuteTool,
        }
    }
}

impl Tool for WrappedShellExecuteTool {
    const NAME: &'static str = "shell_execute";

    type Error = FileToolError;
    type Args = <ShellExecuteTool as Tool>::Args;
    type Output = <ShellExecuteTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!();
        println!("{} {}({})", "●".bright_green(), "Exec", args.command);

        // Git 安全检查
        Self::check_git_safety(&args.command);

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                if output.success {
                    let stdout_lines = output.stdout.lines().count();
                    if stdout_lines > 0 {
                        println!(
                            "  └─ {} ... +{} lines output",
                            "Command succeeded".dimmed(),
                            stdout_lines
                        );
                    } else {
                        println!("  └─ {}", "Command succeeded".dimmed());
                    }
                } else {
                    let stderr_lines = output.stderr.lines().count();
                    println!(
                        "  └─ {} (exit: {})",
                        format!("Command failed, {} lines stderr", stderr_lines).red(),
                        output.exit_code.unwrap_or(-1)
                    );
                }
            }
            Err(e) => {
                println!("  └─ {}", format!("Error: {}", e).red());
            }
        }
        println!();
        result
    }
}

impl WrappedShellExecuteTool {
    /// 检查 Git 命令的安全性
    fn check_git_safety(command: &str) {
        let command_lower = command.trim().to_lowercase();

        // 检查是否是 Git 命令
        if !command_lower.starts_with("git ") {
            return;
        }

        // 尝试创建 Git Guard
        let guard = match GitGuard::new() {
            Ok(g) => g,
            Err(_) => return, // 不在 Git 仓库中，跳过检查
        };

        // 检查特定的 Git 命令
        if command_lower.contains("git push") {
            // 检查是否在主分支上
            guard.warn_if_pushing_to_main();

            // 检查是否有 --force 标志
            if command_lower.contains("--force") || command_lower.contains("-f") {
                println!();
                println!(
                    "{} {}",
                    "🚨".bright_red(),
                    "警告: 强制推送将会重写 Git 历史".bright_red().bold()
                );
                println!(
                    "  这可能导致: {}",
                    "其他协作者的提交丢失、分支冲突".bright_yellow()
                );
                println!(
                    "  如果确实需要, 请考虑使用: {}",
                    "git push --force-with-lease".bright_cyan()
                );
                println!();
            }
        } else if command_lower.contains("git commit") {
            // 验证 commit 消息
            Self::validate_commit_message(command);

            // 检查 Git 状态
            let safety = guard.check_safety();
            match safety {
                super::git_guard::GitSafety::UncommittedChanges => {
                    // 对于 commit 命令，这是正常的，不需要警告
                }
                super::git_guard::GitSafety::OnMainBranch { branch_name } => {
                    println!();
                    println!(
                        "{} {}",
                        "⚠️ ".bright_yellow(),
                        "注意: 即将在主分支上提交".bright_yellow().bold()
                    );
                    println!("  当前分支: {}", branch_name.bright_white());
                    println!();
                }
                _ => {}
            }
        } else if command_lower.contains("git checkout") || command_lower.contains("git switch") {
            // 检查是否有未提交的更改
            if let super::git_guard::GitSafety::UncommittedChanges = guard.check_safety() {
                println!();
                println!(
                    "{} {}",
                    "⚠️ ".bright_yellow(),
                    "警告: 切换分支前有未提交的更改".bright_yellow().bold()
                );
                println!(
                    "  建议: {} 或 {}",
                    "git stash".bright_cyan(),
                    "git commit".bright_cyan()
                );
                println!();
            }
        }
    }

    /// 验证 commit 消息格式
    fn validate_commit_message(command: &str) {
        // 检查是否包含 -m 参数（用于指定 commit 消息）
        let parts: Vec<&str> = command.split(' ').collect();
        let mut message_index = None;

        for (i, part) in parts.iter().enumerate() {
            if *part == "-m" || part.starts_with("-m=") {
                if *part == "-m" && i + 1 < parts.len() {
                    message_index = Some(i + 1);
                } else if part.starts_with("-m=") {
                    // 提取 -m="message" 格式中的消息
                    let msg = part.strip_prefix("-m=").unwrap_or("");
                    Self::check_commit_format(msg);
                    return;
                }
                break;
            }
        }

        if let Some(idx) = message_index {
            if let Some(&message) = parts.get(idx) {
                // 去除可能的引号
                let message = message.trim_matches('"').trim_matches('\'');
                Self::check_commit_format(message);
            }
        }
    }

    /// 检查 commit 消息格式
    fn check_commit_format(message: &str) {
        let linter = match CommitLinter::new() {
            Ok(l) => l,
            Err(_) => return, // 如果 linter 创建失败，跳过检查
        };

        let result = linter.validate(message);

        // 显示验证结果
        if !result.valid {
            println!();
            println!(
                "{} {}",
                "✗".bright_red(),
                "Commit 消息格式无效".bright_red()
            );
            for error in &result.errors {
                println!("  {}", error.dimmed());
            }
            println!();
        } else if !result.warnings.is_empty() {
            println!();
            println!(
                "{} {}",
                "⚠️".bright_yellow(),
                "Commit 消息格式建议".bright_yellow()
            );
            for warning in &result.warnings {
                println!("  {}", warning.dimmed());
            }
            println!();
        } else {
            // 验证通过，显示简洁的成功信息
            let type_str = result.commit_type.as_deref().unwrap_or("unknown");
            println!(
                "  └─ {}",
                format!("✓ Commit 格式: {}", type_str).dimmed()
            );
        }
    }
}
