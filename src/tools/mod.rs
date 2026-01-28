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
pub mod plan_mode;
pub mod read_file;
pub mod scan_codebase;
pub mod write_file;
pub mod search_replace;
pub mod shell_execute;
pub mod task;
pub mod task_output;
pub mod task_create;
pub mod task_update;
pub mod task_list;
pub mod task_get;

pub use ask_user_question::WrappedAskUserQuestionTool;
pub use create_directory::WrappedCreateDirectoryTool;
pub use delete_file::WrappedDeleteFileTool;
pub use edit_file::WrappedEditFileTool;
pub use glob::WrappedGlobTool;
pub use grep_search::WrappedGrepSearchTool;
pub use plan_mode::{WrappedEnterPlanModeTool, WrappedExitPlanModeTool};
pub use plan_mode::{AllowedPrompt, PlanModeState, is_in_plan_mode, is_plan_approved, is_operation_allowed, set_plan_content, get_plan_state};
pub use read_file::WrappedReadFileTool;
pub use scan_codebase::WrappedScanCodebaseTool;
pub use write_file::WrappedWriteFileTool;
pub use shell_execute::WrappedShellExecuteTool;
pub use search_replace::WrappedSearchReplaceTool;

// 任务管理工具
pub use task_create::WrappedTaskCreateTool;
pub use task_update::WrappedTaskUpdateTool;
pub use task_list::WrappedTaskListTool;
pub use task_get::WrappedTaskGetTool;
