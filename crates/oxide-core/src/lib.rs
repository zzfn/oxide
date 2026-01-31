//! Oxide 核心类型和 trait
//!
//! 本模块定义了 Oxide 项目的基础类型、错误处理和配置系统。

pub mod config;
pub mod env;
pub mod error;
pub mod prompt;
pub mod session;
pub mod types;

pub use config::Config;
pub use env::Env;
pub use error::{OxideError, Result};
pub use prompt::{BuiltPrompt, PromptBuilder, PromptPart, RuntimeContext, ToolDefinition};
pub use session::{History, SessionState};
