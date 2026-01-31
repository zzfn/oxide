//! 命令模块
//!
//! 提供快捷命令系统，包括命令注册、分发和内置命令。

pub mod builtin;
pub mod registry;

pub use builtin::{
    ClearCommand, CompactCommand, ConfigCommand, HelpCommand, ModeCommand, QuitCommand,
    ReloadConfigCommand, TasksCommand, ToolsCommand, create_default_registry, register_help_command,
};
pub use registry::{Command, CommandRegistry, CommandResult};

use std::sync::Arc;

/// 创建完整的命令注册表（包含 help 命令）
pub fn create_registry() -> Arc<CommandRegistry> {
    let mut registry = CommandRegistry::new();

    // 注册所有内置命令
    registry.register(Arc::new(ClearCommand));
    registry.register(Arc::new(CompactCommand));
    registry.register(Arc::new(TasksCommand));
    registry.register(Arc::new(ConfigCommand));
    registry.register(Arc::new(ReloadConfigCommand));
    registry.register(Arc::new(QuitCommand));
    registry.register(Arc::new(ModeCommand));
    registry.register(Arc::new(ToolsCommand));

    let registry = Arc::new(registry);

    // 创建一个新的 registry 来包含 help 命令
    let mut final_registry = CommandRegistry::new();
    final_registry.register(Arc::new(ClearCommand));
    final_registry.register(Arc::new(CompactCommand));
    final_registry.register(Arc::new(TasksCommand));
    final_registry.register(Arc::new(ConfigCommand));
    final_registry.register(Arc::new(ReloadConfigCommand));
    final_registry.register(Arc::new(QuitCommand));
    final_registry.register(Arc::new(ModeCommand));
    final_registry.register(Arc::new(ToolsCommand));
    final_registry.register(Arc::new(HelpCommand::new(registry)));

    Arc::new(final_registry)
}
