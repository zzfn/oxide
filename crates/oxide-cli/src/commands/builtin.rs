//! 内置命令
//!
//! 实现 /help, /clear, /compact, /tasks, /config, /quit 等命令。

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use super::registry::{Command, CommandRegistry, CommandResult};
use crate::app::SharedAppState;

/// 帮助命令
pub struct HelpCommand {
    registry: Arc<CommandRegistry>,
}

impl HelpCommand {
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "显示帮助信息"
    }

    fn usage(&self) -> &str {
        "/help [command]"
    }

    async fn execute(&self, args: &[&str], _state: SharedAppState) -> Result<CommandResult> {
        if let Some(cmd_name) = args.first() {
            // 显示特定命令的帮助
            if let Some(cmd) = self.registry.get(cmd_name) {
                let help = format!(
                    "**/{name}** - {desc}\n\n用法: `{usage}`",
                    name = cmd.name(),
                    desc = cmd.description(),
                    usage = if cmd.usage().is_empty() {
                        format!("/{}", cmd.name())
                    } else {
                        cmd.usage().to_string()
                    }
                );
                return Ok(CommandResult::Message(help));
            } else {
                return Ok(CommandResult::Message(format!("未知命令: {}", cmd_name)));
            }
        }

        // 显示所有命令
        let mut help = String::from("## 可用命令\n\n");
        let mut commands: Vec<_> = self.registry.commands();
        commands.sort_by(|a, b| a.name().cmp(b.name()));

        for cmd in commands {
            help.push_str(&format!(
                "- **/{name}** - {desc}\n",
                name = cmd.name(),
                desc = cmd.description()
            ));
        }

        help.push_str("\n输入 `/help <command>` 查看详细用法。");

        Ok(CommandResult::Message(help))
    }
}

/// 清空命令
pub struct ClearCommand;

#[async_trait]
impl Command for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }

    fn description(&self) -> &str {
        "清空当前会话"
    }

    async fn execute(&self, _args: &[&str], state: SharedAppState) -> Result<CommandResult> {
        let mut state = state.write().await;
        state.clear_session();

        // 清屏
        print!("\x1B[2J\x1B[1;1H");

        Ok(CommandResult::Message("会话已清空。".to_string()))
    }
}

/// 压缩命令
pub struct CompactCommand;

#[async_trait]
impl Command for CompactCommand {
    fn name(&self) -> &str {
        "compact"
    }

    fn description(&self) -> &str {
        "压缩上下文以节省 Token"
    }

    async fn execute(&self, _args: &[&str], _state: SharedAppState) -> Result<CommandResult> {
        // TODO: 实现上下文压缩逻辑
        Ok(CommandResult::Message("上下文压缩功能尚未实现。".to_string()))
    }
}

/// 任务列表命令
pub struct TasksCommand;

#[async_trait]
impl Command for TasksCommand {
    fn name(&self) -> &str {
        "tasks"
    }

    fn description(&self) -> &str {
        "显示后台任务列表"
    }

    async fn execute(&self, _args: &[&str], state: SharedAppState) -> Result<CommandResult> {
        let state = state.read().await;
        let tasks = &state.background_tasks;

        let msg = format!(
            "## 后台任务\n\n- 运行中: {}\n- 已完成: {}",
            tasks.running, tasks.completed
        );

        Ok(CommandResult::Message(msg))
    }
}

/// 配置命令
pub struct ConfigCommand;

#[async_trait]
impl Command for ConfigCommand {
    fn name(&self) -> &str {
        "config"
    }

    fn description(&self) -> &str {
        "查看或修改配置"
    }

    fn usage(&self) -> &str {
        "/config [key] [value]"
    }

    async fn execute(&self, args: &[&str], state: SharedAppState) -> Result<CommandResult> {
        let state = state.read().await;

        if args.is_empty() {
            // 显示当前配置
            let msg = format!(
                "## 当前配置\n\n\
                - 模式: {}\n\
                - 工作目录: {}\n\
                - 会话 ID: {}",
                state.mode.display_name(),
                state.working_dir.display(),
                state.session_id.as_deref().unwrap_or("无")
            );
            return Ok(CommandResult::Message(msg));
        }

        // TODO: 实现配置修改
        Ok(CommandResult::Message("配置修改功能尚未实现。".to_string()))
    }
}

/// 退出命令
pub struct QuitCommand;

#[async_trait]
impl Command for QuitCommand {
    fn name(&self) -> &str {
        "quit"
    }

    fn description(&self) -> &str {
        "退出程序"
    }

    async fn execute(&self, _args: &[&str], _state: SharedAppState) -> Result<CommandResult> {
        Ok(CommandResult::Exit)
    }
}

/// 模式切换命令
pub struct ModeCommand;

#[async_trait]
impl Command for ModeCommand {
    fn name(&self) -> &str {
        "mode"
    }

    fn description(&self) -> &str {
        "切换运行模式"
    }

    fn usage(&self) -> &str {
        "/mode [normal|fast|plan]"
    }

    async fn execute(&self, args: &[&str], state: SharedAppState) -> Result<CommandResult> {
        let mut state = state.write().await;

        if let Some(mode_str) = args.first() {
            match mode_str.to_lowercase().as_str() {
                "normal" | "n" => state.set_mode(crate::app::CliMode::Normal),
                "fast" | "f" => state.set_mode(crate::app::CliMode::Fast),
                "plan" | "p" => state.set_mode(crate::app::CliMode::Plan),
                _ => {
                    return Ok(CommandResult::Message(format!(
                        "未知模式: {}。可用模式: normal, fast, plan",
                        mode_str
                    )));
                }
            }
        } else {
            state.toggle_mode();
        }

        Ok(CommandResult::Message(format!(
            "当前模式: {}",
            state.mode.display_name()
        )))
    }
}

/// 创建默认命令注册表
pub fn create_default_registry() -> Arc<CommandRegistry> {
    // 注册内置命令（除了 help，因为它需要 registry 引用）
    let mut reg = CommandRegistry::new();
    reg.register(Arc::new(ClearCommand));
    reg.register(Arc::new(CompactCommand));
    reg.register(Arc::new(TasksCommand));
    reg.register(Arc::new(ConfigCommand));
    reg.register(Arc::new(QuitCommand));
    reg.register(Arc::new(ModeCommand));

    // 创建最终的 registry 并添加 help 命令
    let final_registry = Arc::new(reg);

    // 由于 HelpCommand 需要 registry 引用，我们需要特殊处理
    // 这里先返回不含 help 的 registry，后续在 REPL 中添加
    final_registry
}

/// 注册 help 命令（需要在 registry 创建后调用）
pub fn register_help_command(registry: &mut CommandRegistry, registry_ref: Arc<CommandRegistry>) {
    registry.register(Arc::new(HelpCommand::new(registry_ref)));
}

/// Tools 命令 - 列出所有可用工具
pub struct ToolsCommand;

#[async_trait]
impl Command for ToolsCommand {
    fn name(&self) -> &str {
        "tools"
    }

    fn description(&self) -> &str {
        "列出所有可用的工具"
    }

    fn usage(&self) -> &str {
        "/tools"
    }

    async fn execute(&self, _args: &[&str], state: SharedAppState) -> Result<CommandResult> {
        let state = state.read().await;

        let Some(ref tool_registry) = state.tool_registry else {
            return Ok(CommandResult::Message("工具注册表未初始化".to_string()));
        };

        let schemas = tool_registry.schemas();

        let mut output = String::from("## 可用工具\n\n");

        for schema in schemas {
            output.push_str(&format!("### {}\n", schema.name));
            output.push_str(&format!("{}\n\n", schema.description));
            output.push_str(&format!("**参数**: \n```json\n{}\n```\n\n",
                serde_json::to_string_pretty(&schema.parameters).unwrap_or_default()));
        }

        Ok(CommandResult::Message(output))
    }
}
