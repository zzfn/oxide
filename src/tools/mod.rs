use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Path is not a file: {0}")]
    NotAFile(String),
    #[error("Input is invalid: {0}")]
    InvalidInput(String),
    #[error("Operation cancelled by user")]
    #[allow(dead_code)]
    Cancelled,
}

pub mod ask_user_question;
pub mod commit_linter;
pub mod create_directory;
pub mod delete_file;
pub mod edit_file;
pub mod git_guard;
pub mod glob;
pub mod grep_search;
pub mod multiedit;
pub mod notebook_edit;
pub mod read_file;
pub mod scan_codebase;
pub mod write_file;
pub mod search_replace;
pub mod shell_execute;
pub mod task;
pub mod task_output;

pub use create_directory::WrappedCreateDirectoryTool;
pub use delete_file::WrappedDeleteFileTool;
pub use edit_file::WrappedEditFileTool;
pub use glob::WrappedGlobTool;
pub use grep_search::WrappedGrepSearchTool;
pub use read_file::WrappedReadFileTool;
pub use scan_codebase::WrappedScanCodebaseTool;
pub use write_file::WrappedWriteFileTool;
pub use shell_execute::WrappedShellExecuteTool;
pub use search_replace::WrappedSearchReplaceTool;

// task 和 task_output 模块暂未集成到主 Agent
// 这些工具将在未来版本中使用
