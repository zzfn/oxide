//! rig Tool trait 适配层
//!
//! 本模块将 Oxide 工具适配为 rig-core 的 Tool trait，
//! 以便与 rig Agent 集成使用。

pub mod errors;
pub mod exec;
pub mod file;
pub mod interaction;
pub mod plan;
pub mod search;
pub mod wrapper;

pub use errors::*;
pub use exec::*;
pub use file::*;
pub use interaction::*;
pub use plan::*;
pub use search::*;
pub use wrapper::ToolWrapper;

use rig::tool::{ToolDyn, ToolSet};
use std::path::PathBuf;

use crate::task::TaskManager;

// 重新导出 TaskManager
pub use crate::task::TaskManager as TaskManagerType;

/// 创建新的任务管理器
pub fn create_task_manager() -> TaskManager {
    TaskManager::new()
}

/// Oxide 工具集构建器
///
/// 用于创建包含所有 Oxide 工具的 rig ToolSet
pub struct OxideToolSetBuilder {
    working_dir: PathBuf,
    task_manager: Option<TaskManager>,
    plan_manager: Option<PlanManager>,
    include_search: bool,
    include_file: bool,
    include_exec: bool,
    include_task: bool,
    include_interaction: bool,
    include_plan: bool,
}

impl OxideToolSetBuilder {
    /// 创建新的构建器
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            task_manager: None,
            plan_manager: None,
            include_search: true,
            include_file: true,
            include_exec: true,
            include_task: true,
            include_interaction: true,
            include_plan: true,
        }
    }

    /// 设置任务管理器（用于后台任务）
    pub fn task_manager(mut self, manager: TaskManager) -> Self {
        self.task_manager = Some(manager);
        self
    }

    /// 设置计划管理器（用于计划模式）
    pub fn plan_manager(mut self, manager: PlanManager) -> Self {
        self.plan_manager = Some(manager);
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

    /// 是否包含任务管理工具 (TaskCreate, TaskList, TaskGet, TaskUpdate)
    pub fn task_tools(mut self, include: bool) -> Self {
        self.include_task = include;
        self
    }

    /// 是否包含交互工具 (AskUserQuestion)
    pub fn interaction_tools(mut self, include: bool) -> Self {
        self.include_interaction = include;
        self
    }

    /// 是否包含计划模式工具 (EnterPlanMode, ExitPlanMode)
    pub fn plan_tools(mut self, include: bool) -> Self {
        self.include_plan = include;
        self
    }

    /// 构建 ToolSet
    pub fn build(self) -> ToolSet {
        let mut toolset = ToolSet::default();
        let task_manager = self.task_manager.unwrap_or_else(create_task_manager);
        let plan_manager = self.plan_manager.unwrap_or_else(PlanManager::new);

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
            toolset.add_tool(RigTaskStopTool::new(task_manager.clone()));
        }

        // 添加任务管理工具
        if self.include_task {
            use crate::task::tools::*;
            toolset.add_tool(RigTaskCreateTool::new(task_manager.clone()));
            toolset.add_tool(RigTaskListTool::new(task_manager.clone()));
            toolset.add_tool(RigTaskGetTool::new(task_manager.clone()));
            toolset.add_tool(RigTaskUpdateTool::new(task_manager.clone()));
        }

        // 添加交互工具
        if self.include_interaction {
            toolset.add_tool(RigAskUserQuestionTool::new());
        }

        // 添加计划模式工具
        if self.include_plan {
            toolset.add_tool(RigEnterPlanModeTool::new(plan_manager.clone()));
            toolset.add_tool(RigExitPlanModeTool::new(plan_manager.clone()));
        }

        toolset
    }

    /// 构建为 boxed 工具列表（用于动态场景）
    pub fn build_boxed(self) -> Vec<Box<dyn ToolDyn>> {
        let mut tools: Vec<Box<dyn ToolDyn>> = Vec::new();
        let task_manager = self.task_manager.unwrap_or_else(create_task_manager);
        let plan_manager = self.plan_manager.unwrap_or_else(PlanManager::new);

        if self.include_search {
            tools.push(Box::new(ToolWrapper::new(RigGlobTool::new(self.working_dir.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigGrepTool::new(self.working_dir.clone()))));
        }

        if self.include_file {
            tools.push(Box::new(ToolWrapper::new(RigReadTool::new(self.working_dir.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigWriteTool::new(self.working_dir.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigEditTool::new(self.working_dir.clone()))));
        }

        if self.include_exec {
            tools.push(Box::new(ToolWrapper::new(RigBashTool::new(self.working_dir.clone(), task_manager.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigTaskOutputTool::new(task_manager.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigTaskStopTool::new(task_manager.clone()))));
        }

        if self.include_task {
            use crate::task::tools::*;
            tools.push(Box::new(ToolWrapper::new(RigTaskCreateTool::new(task_manager.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigTaskListTool::new(task_manager.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigTaskGetTool::new(task_manager.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigTaskUpdateTool::new(task_manager.clone()))));
        }

        if self.include_interaction {
            tools.push(Box::new(ToolWrapper::new(RigAskUserQuestionTool::new())));
        }

        if self.include_plan {
            tools.push(Box::new(ToolWrapper::new(RigEnterPlanModeTool::new(plan_manager.clone()))));
            tools.push(Box::new(ToolWrapper::new(RigExitPlanModeTool::new(plan_manager.clone()))));
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
