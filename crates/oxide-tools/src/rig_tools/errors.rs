//! rig 工具适配的错误类型定义

use std::fmt;
use thiserror::Error;

/// 文件操作错误
#[derive(Debug, Error)]
pub enum FileError {
    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("路径不是文件: {0}")]
    NotFile(String),

    #[error("未找到要替换的字符串: {0}")]
    StringNotFound(String),

    #[error("找到 {count} 个匹配项，但 replace_all=false。请提供更具体的字符串或设置 replace_all=true")]
    MultipleMatches { count: usize },

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}

/// 搜索操作错误
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("路径不存在: {0}")]
    PathNotFound(String),

    #[error("路径不是目录: {0}")]
    NotDirectory(String),

    #[error("Glob 模式错误: {0}")]
    PatternError(#[from] glob::PatternError),

    #[error("正则表达式错误: {0}")]
    RegexError(String),

    #[error("搜索失败: {0}")]
    SearchFailed(String),
}

/// 命令执行错误
#[derive(Debug, Error)]
pub enum ExecError {
    #[error("命令执行超时 ({0}ms)")]
    Timeout(u64),

    #[error("命令执行失败: {0}")]
    ExecutionFailed(String),

    #[error("任务不存在: {0}")]
    TaskNotFound(String),

    #[error("任务已停止")]
    TaskAlreadyStopped,

    #[error("等待任务完成超时")]
    WaitTimeout,

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}

/// 通用工具错误
#[derive(Debug, Error)]
pub enum RigToolError {
    #[error("参数解析失败: {0}")]
    InvalidArgs(String),

    #[error("执行失败: {0}")]
    ExecutionError(String),

    #[error("文件错误: {0}")]
    FileError(#[from] FileError),

    #[error("搜索错误: {0}")]
    SearchError(#[from] SearchError),

    #[error("执行错误: {0}")]
    ExecError(#[from] ExecError),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// 权限错误
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("权限被拒绝: 工具 '{0}' 被配置禁止执行，操作未执行")]
    ToolDenied(String),

    #[error("用户拒绝: 用户拒绝执行工具 '{0}'，操作未执行。请告知用户操作已取消，不要假装操作成功。")]
    UserRejected(String),

    #[error("权限错误: 工具 '{0}' 需要用户确认，但未配置确认处理器，操作未执行")]
    NoConfirmationHandler(String),
}

/// 包装错误 - 用于 ToolWrapper，可以包含权限错误或内部工具错误
#[derive(Debug)]
pub enum WrappedError<E> {
    /// 权限错误
    Permission(PermissionError),
    /// 内部工具错误
    Inner(E),
}

impl<E: fmt::Display> fmt::Display for WrappedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrappedError::Permission(e) => write!(f, "{}", e),
            WrappedError::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for WrappedError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WrappedError::Permission(e) => Some(e),
            WrappedError::Inner(e) => Some(e),
        }
    }
}

impl<E> From<PermissionError> for WrappedError<E> {
    fn from(e: PermissionError) -> Self {
        WrappedError::Permission(e)
    }
}
