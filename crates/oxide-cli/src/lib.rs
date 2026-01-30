//! Oxide CLI 库
//!
//! 提供 CLI 的公共 API，包括应用状态、命令系统、渲染和 REPL。

pub mod agent;
pub mod app;
pub mod commands;
pub mod interaction;
pub mod render;
pub mod repl;
pub mod statusbar;

// 重新导出常用类型
pub use agent::RigAgentRunner;
pub use app::{AppState, CliMode, SharedAppState, TokenUsage, create_shared_state};

// 兼容旧接口（已弃用）
#[allow(deprecated)]
pub use agent::{Agent, create_tool_registry};
