//! rig 工具适配的错误类型定义

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
