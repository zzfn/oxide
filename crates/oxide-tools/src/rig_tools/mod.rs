//! rig Tool trait 适配层
//!
//! 本模块将 Oxide 工具适配为 rig-core 的 Tool trait，
//! 以便与 rig Agent 集成使用。

pub mod errors;
pub mod exec;
pub mod file;
pub mod search;

pub use errors::*;
pub use exec::*;
pub use file::*;
pub use search::*;

use rig::tool::{ToolDyn, ToolSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::exec::BackgroundTask;

/// 后台任务管理器类型
pub type TaskManager = Arc<RwLock<HashMap<String, BackgroundTask>>>;

/// 创建新的任务管理器
pub fn create_task_manager() -> TaskManager {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Oxide 工具集构建器
///
/// 用于创建包含所有 Oxide 工具的 rig ToolSet
pub struct OxideToolSetBuilder {
    working_dir: PathBuf,
    task_manager: Option<TaskManager>,
    include_search: bool,
    include_file: bool,
    include_exec: bool,
}

impl OxideToolSetBuilder {
    /// 创建新的构建器
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            task_manager: None,
            include_search: true,
            include_file: true,
            include_exec: true,
        }
    }

    /// 设置任务管理器（用于后台任务）
    pub fn task_manager(mut self, manager: TaskManager) -> Self {
        self.task_manager = Some(manager);
        self
    }

    /// 是否包含搜索工具 (Glob, Grep)
    pub fn search_tools(mut self, include: bool) -> Self {
        self.include_search = include;
        self
    }

    /// 是否包含文件工具 (Read, Write, Edit)
    pub fn file_tools(mut self, include: bool) -> Self {
        self.include_file = include;
        self
    }

    /// 是否包含执行工具 (Bash, TaskOutput, TaskStop)
    pub fn exec_tools(mut self, include: bool) -> Self {
        self.include_exec = include;
        self
    }

    /// 构建 ToolSet
    pub fn build(self) -> ToolSet {
        let mut toolset = ToolSet::default();
        let task_manager = self.task_manager.unwrap_or_else(create_task_manager);

        // 添加搜索工具
        if self.include_search {
            toolset.add_tool(RigGlobTool::new(self.working_dir.clone()));
            toolset.add_tool(RigGrepTool::new(self.working_dir.clone()));
        }

        // 添加文件工具
        if self.include_file {
            toolset.add_tool(RigReadTool::new(self.working_dir.clone()));
            toolset.add_tool(RigWriteTool::new(self.working_dir.clone()));
            toolset.add_tool(RigEditTool::new(self.working_dir.clone()));
        }

        // 添加执行工具
        if self.include_exec {
            toolset.add_tool(RigBashTool::new(self.working_dir.clone(), task_manager.clone()));
            toolset.add_tool(RigTaskOutputTool::new(task_manager.clone()));
            toolset.add_tool(RigTaskStopTool::new(task_manager));
        }

        toolset
    }

    /// 构建为 boxed 工具列表（用于动态场景）
    pub fn build_boxed(self) -> Vec<Box<dyn ToolDyn>> {
        let mut tools: Vec<Box<dyn ToolDyn>> = Vec::new();
        let task_manager = self.task_manager.unwrap_or_else(create_task_manager);

        if self.include_search {
            tools.push(Box::new(RigGlobTool::new(self.working_dir.clone())));
            tools.push(Box::new(RigGrepTool::new(self.working_dir.clone())));
        }

        if self.include_file {
            tools.push(Box::new(RigReadTool::new(self.working_dir.clone())));
            tools.push(Box::new(RigWriteTool::new(self.working_dir.clone())));
            tools.push(Box::new(RigEditTool::new(self.working_dir.clone())));
        }

        if self.include_exec {
            tools.push(Box::new(RigBashTool::new(self.working_dir.clone(), task_manager.clone())));
            tools.push(Box::new(RigTaskOutputTool::new(task_manager.clone())));
            tools.push(Box::new(RigTaskStopTool::new(task_manager)));
        }

        tools
    }
}

/// 快速创建包含所有工具的 ToolSet
pub fn create_oxide_toolset(working_dir: PathBuf) -> ToolSet {
    OxideToolSetBuilder::new(working_dir).build()
}

/// 快速创建包含所有工具的 ToolSet（带任务管理器）
pub fn create_oxide_toolset_with_task_manager(
    working_dir: PathBuf,
    task_manager: TaskManager,
) -> ToolSet {
    OxideToolSetBuilder::new(working_dir)
        .task_manager(task_manager)
        .build()
}
