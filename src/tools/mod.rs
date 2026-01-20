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
}

pub mod create_directory;
pub mod delete_file;
pub mod edit_file;
pub mod glob;
pub mod grep_search;
pub mod read_file;
pub mod scan_codebase;
pub mod write_file;
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
pub use task::WrappedTaskTool;
pub use task_output::WrappedTaskOutputTool;
