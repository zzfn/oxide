//! LLM 提供商适配
//!
//! 本模块提供与各种 LLM 服务的集成，基于 rig-core 库。

pub mod rig_provider;
pub mod traits;

pub use rig_provider::RigAnthropicProvider;
pub use traits::LLMProvider;

// Re-export rig types
pub use rig_provider::{Prompt, RigAgent, Tool, ToolDefinition, ToolDyn};
// ToolSet 通过 rig_provider::ToolSet 访问
