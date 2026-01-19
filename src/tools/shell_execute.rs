use super::FileToolError;
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
