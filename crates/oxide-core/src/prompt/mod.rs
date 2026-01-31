//! 提示词管理模块
//!
//! 负责加载、组织和构建系统提示词。

mod builder;
mod context;
mod parts;
mod tool;

pub use builder::{BuiltPrompt, PromptBuilder};
pub use context::RuntimeContext;
pub use parts::{PromptPart, Prompts};
pub use tool::ToolDefinition;
