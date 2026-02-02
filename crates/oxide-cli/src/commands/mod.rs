//! 命令模块
//!
//! 提供快捷命令系统，包括命令注册、分发和内置命令。

pub mod builtin;
pub mod registry;

pub use builtin::{
    ClearCommand, CompactCommand, ConfigCommand, HelpCommand, ModeCommand, QuitCommand,
    ReloadConfigCommand, TasksCommand,
};
pub use registry::{Command, CommandRegistry, CommandResult};

use std::sync::Arc;

/// 创建完整的命令注册表（包含 help 命令）
pub fn create_registry() -> Arc<CommandRegistry> {
    // 辅助函数：注册基础命令（不含 help）
    fn register_base_commands(registry: &mut CommandRegistry) {
        registry.register(Arc::new(ClearCommand));
        registry.register(Arc::new(CompactCommand));
        registry.register(Arc::new(TasksCommand));
        registry.register(Arc::new(ConfigCommand));
        registry.register(Arc::new(ReloadConfigCommand));
        registry.register(Arc::new(QuitCommand));
        registry.register(Arc::new(ModeCommand));
    }

    // 创建第一个 registry（不含 help，供 HelpCommand 引用）
    let mut base_registry = CommandRegistry::new();
    register_base_commands(&mut base_registry);
    let base_registry = Arc::new(base_registry);

    // 创建最终的 registry（包含 help）
    let mut final_registry = CommandRegistry::new();
    register_base_commands(&mut final_registry);
    final_registry.register(Arc::new(HelpCommand::new(base_registry)));

    Arc::new(final_registry)
}
