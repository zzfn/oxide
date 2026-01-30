//! 任务系统错误类型

use thiserror::Error;

/// 任务系统错误
#[derive(Debug, Error)]
pub enum TaskError {
    /// 任务不存在
    #[error("任务不存在: {0}")]
    TaskNotFound(String),

    /// 无效的状态转换
    #[error("无效的状态转换: {0}")]
    InvalidStatusTransition(String),

    /// 循环依赖
    #[error("检测到循环依赖: {0}")]
    CircularDependency(String),

    /// 任务已被阻塞
    #[error("任务被阻塞，无法执行: {0}")]
    TaskBlocked(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// 其他错误
    #[error("任务系统错误: {0}")]
    Other(String),
}

impl TaskError {
    /// 创建任务不存在错误
    pub fn not_found(task_id: impl Into<String>) -> Self {
        Self::TaskNotFound(task_id.into())
    }

    /// 创建循环依赖错误
    pub fn circular_dependency(msg: impl Into<String>) -> Self {
        Self::CircularDependency(msg.into())
    }

    /// 创建任务被阻塞错误
    pub fn blocked(task_id: impl Into<String>) -> Self {
        Self::TaskBlocked(task_id.into())
    }
}
