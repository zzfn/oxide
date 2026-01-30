//! 工具系统实现
//!
//! 本模块提供 Oxide 的工具框架和内置工具。
//!
//! ## 模块结构
//!
//! - `registry` - 工具注册表和基础 trait
//! - `exec` - 命令执行工具 (Bash, TaskOutput, TaskStop)
//! - `file` - 文件操作工具 (Read, Write, Edit)
//! - `search` - 搜索工具 (Glob, Grep)
//! - `rig_tools` - rig Tool trait 适配层

pub mod exec;
pub mod file;
pub mod interaction;
pub mod permission;
pub mod plan;
pub mod registry;
pub mod rig_tools;
pub mod search;
pub mod task;
pub mod web;

// 重新导出常用类型
pub use exec::{BashTool, TaskOutputTool, TaskStopTool};
pub use file::{EditTool, ReadTool, WriteTool};
pub use interaction::{AskUserQuestionTool, QuestionOption, QuestionType};
pub use permission::{ConfirmationCallback, PermissionManager};
pub use plan::{PlanManager, RigEnterPlanModeTool, RigExitPlanModeTool};
pub use registry::{Tool, ToolRegistry, ToolResult, ToolSchema};
pub use search::{GlobTool, GrepTool};
pub use task::{Task, TaskError, TaskManager, TaskStatus};

// 创建任务管理器的工厂函数
pub fn create_task_manager() -> TaskManager {
    TaskManager::new()
}

// 重新导出 rig 适配工具
pub use rig_tools::{
    create_oxide_toolset, create_oxide_toolset_with_task_manager, OxideToolSetBuilder,
    // 搜索工具
    RigGlobTool, RigGrepTool,
    // 文件工具
    RigEditTool, RigReadTool, RigWriteTool,
    // 执行工具
    RigBashTool, RigTaskOutputTool, RigTaskStopTool,
};
