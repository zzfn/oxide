//! 命令注册表
//!
//! 管理和分发快捷命令。

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::app::SharedAppState;

/// 命令执行结果
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// 继续运行
    Continue,
    /// 退出程序
    Exit,
    /// 返回消息
    Message(String),
}

/// 命令 trait
#[async_trait]
pub trait Command: Send + Sync {
    /// 命令名称（不含 / 前缀）
    fn name(&self) -> &str;

    /// 命令描述
    fn description(&self) -> &str;

    /// 命令用法
    fn usage(&self) -> &str {
        ""
    }

    /// 执行命令
    async fn execute(&self, args: &[&str], state: SharedAppState) -> Result<CommandResult>;
}

/// 命令注册表
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl CommandRegistry {
    /// 创建新的命令注册表
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// 注册命令
    pub fn register(&mut self, command: Arc<dyn Command>) {
        self.commands.insert(command.name().to_string(), command);
    }

    /// 获取命令
    pub fn get(&self, name: &str) -> Option<Arc<dyn Command>> {
        self.commands.get(name).cloned()
    }

    /// 执行命令
    pub async fn execute(&self, input: &str, state: SharedAppState) -> Result<CommandResult> {
        let input = input.trim();

        // 解析命令和参数
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(CommandResult::Continue);
        }

        let cmd_name = parts[0].trim_start_matches('/');
        let args = &parts[1..];

        // 查找并执行命令
        if let Some(command) = self.get(cmd_name) {
            command.execute(args, state).await
        } else {
            Ok(CommandResult::Message(format!(
                "未知命令: /{}。输入 /help 查看可用命令。",
                cmd_name
            )))
        }
    }

    /// 获取所有命令名称
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// 获取所有命令
    pub fn commands(&self) -> Vec<Arc<dyn Command>> {
        self.commands.values().cloned().collect()
    }

    /// 检查是否为命令输入
    pub fn is_command(input: &str) -> bool {
        input.trim().starts_with('/')
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
