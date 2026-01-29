//! 统一的错误类型定义

use thiserror::Error;

/// Oxide 统一错误类型
#[derive(Error, Debug)]
pub enum OxideError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("工具执行错误: {0}")]
    ToolExecution(String),

    #[error("Provider 错误: {0}")]
    Provider(String),

    #[error("代理错误: {0}")]
    Agent(String),

    #[error("{0}")]
    Other(String),
}

/// Oxide Result 类型别名
pub type Result<T> = std::result::Result<T, OxideError>;
