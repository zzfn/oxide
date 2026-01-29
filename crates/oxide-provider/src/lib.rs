//! LLM 提供商适配
//!
//! 本模块提供与各种 LLM 服务的集成。

pub mod anthropic;
pub mod traits;

pub use anthropic::AnthropicProvider;
pub use traits::LLMProvider;
