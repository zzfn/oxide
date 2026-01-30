//! Oxide CLI 库
//!
//! 提供 CLI 的公共 API，包括应用状态、命令系统、渲染和 REPL。

pub mod app;
pub mod commands;
pub mod render;
pub mod repl;
pub mod statusbar;

// 重新导出常用类型
pub use app::{AppState, CliMode, SharedAppState, TokenUsage, create_shared_state};
