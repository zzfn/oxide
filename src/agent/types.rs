//! Agent 类型定义
//!
//! 定义了不同类型的 Agent 及其能力、工具权限和系统提示词。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent 类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// 主对话 Agent
    Main,

    /// 代码库探索 Agent（只读）
    Explore,

    /// 架构规划 Agent
    Plan,

    /// 代码审查 Agent（只读）
    CodeReviewer,

    /// 前端开发 Agent
    FrontendDeveloper,

    /// 通用 Agent
    General,
}

impl AgentType {
    /// 获取 Agent 的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::Main => "Main",
            AgentType::Explore => "Explore",
            AgentType::Plan => "Plan",
            AgentType::CodeReviewer => "Code Reviewer",
            AgentType::FrontendDeveloper => "Frontend Developer",
            AgentType::General => "General",
        }
    }

    /// 获取 Agent 的描述
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            AgentType::Main => "主对话 Agent，具有所有工具访问权限",
            AgentType::Explore => "代码库探索 Agent，用于快速分析代码结构和搜索文件",
            AgentType::Plan => "架构规划 Agent，用于设计实现方案和规划任务",
            AgentType::CodeReviewer => "代码审查 Agent，用于检查代码质量和安全性",
            AgentType::FrontendDeveloper => "前端开发 Agent，专注于 UI/UX 实现",
            AgentType::General => "通用 Agent，用于一般性任务",
        }
    }

    /// 解析字符串为 Agent 类型
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "main" => Some(AgentType::Main),
            "explore" | "explorer" => Some(AgentType::Explore),
            "plan" | "planner" => Some(AgentType::Plan),
            "code_reviewer" | "code-reviewer" | "reviewer" => Some(AgentType::CodeReviewer),
            "frontend_developer" | "frontend-developer" | "frontend" => Some(AgentType::FrontendDeveloper),
            "general" => Some(AgentType::General),
            _ => None,
        }
    }
}

/// Agent 能力描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Agent 类型
    pub agent_type: AgentType,

    /// 能力名称
    pub name: String,

    /// 能力描述
    pub description: String,

    /// 可用的工具列表
    pub tools: Vec<String>,

    /// 系统提示词
    pub system_prompt: String,

    /// 是否只读（只能访问只读工具）
    pub read_only: bool,
}

impl AgentCapability {
    /// 创建新的 Agent 能力
    #[allow(dead_code)]
    pub fn new(
        agent_type: AgentType,
        name: String,
        description: String,
        tools: Vec<String>,
        system_prompt: String,
        read_only: bool,
    ) -> Self {
        Self {
            agent_type,
            name,
            description,
            tools,
            system_prompt,
            read_only,
        }
    }

    /// 获取 Main Agent 的能力定义
    pub fn main_capability() -> Self {
        Self {
            agent_type: AgentType::Main,
            name: "Main Agent".to_string(),
            description: "主对话 Agent，具有所有工具访问权限".to_string(),
            tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "edit_file".to_string(),
                "delete_file".to_string(),
                "shell_execute".to_string(),
                "scan_codebase".to_string(),
                "create_directory".to_string(),
                "grep_search".to_string(),
            ],
            system_prompt: r#"
Your name is Oxide. You are a helpful AI code assistant with comprehensive file system and command execution access.
You can read, write, edit (with patches), and delete files, execute bash commands, scan codebase structures, search text in the codebase and create directories.
Use the edit_file tool for making small, targeted changes to existing files - it's more efficient than rewriting entire files.
Please provide clear and concise responses and be careful when modifying files or executing commands.
"#.trim().to_string(),
            read_only: false,
        }
    }

    /// 获取 Explore Agent 的能力定义
    pub fn explore_capability() -> Self {
        Self {
            agent_type: AgentType::Explore,
            name: "Explore Agent".to_string(),
            description: "代码库探索 Agent，用于快速分析代码结构和搜索文件".to_string(),
            tools: vec![
                "read_file".to_string(),
                "grep_search".to_string(),
                "scan_codebase".to_string(),
            ],
            system_prompt: r#"
You are an Explore Agent specialized in codebase exploration and analysis.
Your capabilities are limited to read-only operations: reading files, searching text, and scanning the codebase structure.
When exploring a codebase:
1. Start by getting an overview of the project structure
2. Identify key files and directories
3. Search for relevant code patterns
4. Provide concise summaries of your findings
Use Glob for file pattern matching and Grep for content searching.
"#.trim().to_string(),
            read_only: true,
        }
    }

    /// 获取 Plan Agent 的能力定义
    pub fn plan_capability() -> Self {
        Self {
            agent_type: AgentType::Plan,
            name: "Plan Agent".to_string(),
            description: "架构规划 Agent，用于设计实现方案和规划任务".to_string(),
            tools: vec![
                "read_file".to_string(),
                "grep_search".to_string(),
                "scan_codebase".to_string(),
                "todo_write".to_string(),
            ],
            system_prompt: r#"
You are a Plan Agent specialized in software architecture and implementation planning.
Your role is to:
1. Analyze requirements and explore the codebase
2. Design implementation strategies
3. Break down complex tasks into manageable steps
4. Identify potential issues and trade-offs
5. Create clear, actionable plans
When planning, be thorough but focus on practical, implementable solutions.
"#.trim().to_string(),
            read_only: false,
        }
    }

    /// 获取 Code Reviewer Agent 的能力定义
    pub fn code_reviewer_capability() -> Self {
        Self {
            agent_type: AgentType::CodeReviewer,
            name: "Code Reviewer Agent".to_string(),
            description: "代码审查 Agent，用于检查代码质量和安全性".to_string(),
            tools: vec![
                "read_file".to_string(),
                "grep_search".to_string(),
                "scan_codebase".to_string(),
            ],
            system_prompt: r#"
You are a Code Reviewer Agent specialized in code quality analysis and security review.
Your responsibilities include:
1. Reviewing code for bugs and logic errors
2. Identifying security vulnerabilities (OWASP Top 10, injection attacks, etc.)
3. Checking for code quality issues and maintainability problems
4. Verifying adherence to project conventions
5. Suggesting improvements and best practices
Focus on high-priority issues that truly matter. Be constructive and specific in your feedback.
"#.trim().to_string(),
            read_only: true,
        }
    }

    /// 获取 Frontend Developer Agent 的能力定义
    pub fn frontend_developer_capability() -> Self {
        Self {
            agent_type: AgentType::FrontendDeveloper,
            name: "Frontend Developer Agent".to_string(),
            description: "前端开发 Agent，专注于 UI/UX 实现".to_string(),
            tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "edit_file".to_string(),
                "shell_execute".to_string(),
            ],
            system_prompt: r#"
You are a Frontend Developer Agent specialized in building modern, production-grade user interfaces.
Your expertise includes:
- React, Next.js, Vue, Svelte, and other modern frameworks
- Tailwind CSS, shadcn/ui, and component libraries
- Responsive design and accessibility
- Performance optimization
- Creating polished, maintainable code that avoids generic AI aesthetics
When building UI components, prioritize user experience, maintainability, and web standards compliance.
"#.trim().to_string(),
            read_only: false,
        }
    }

    /// 获取所有 Agent 的能力定义
    #[allow(dead_code)]
    pub fn all_capabilities() -> HashMap<AgentType, AgentCapability> {
        let mut capabilities = HashMap::new();

        capabilities.insert(AgentType::Main, Self::main_capability());
        capabilities.insert(AgentType::Explore, Self::explore_capability());
        capabilities.insert(AgentType::Plan, Self::plan_capability());
        capabilities.insert(AgentType::CodeReviewer, Self::code_reviewer_capability());
        capabilities.insert(AgentType::FrontendDeveloper, Self::frontend_developer_capability());

        capabilities
    }

    /// 获取指定 Agent 类型的能力定义
    #[allow(dead_code)]
    pub fn for_agent_type(agent_type: AgentType) -> Self {
        match agent_type {
            AgentType::Main => Self::main_capability(),
            AgentType::Explore => Self::explore_capability(),
            AgentType::Plan => Self::plan_capability(),
            AgentType::CodeReviewer => Self::code_reviewer_capability(),
            AgentType::FrontendDeveloper => Self::frontend_developer_capability(),
            AgentType::General => Self::main_capability(), // General 使用 Main 的能力
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display_name() {
        assert_eq!(AgentType::Main.display_name(), "Main");
        assert_eq!(AgentType::Explore.display_name(), "Explore");
        assert_eq!(AgentType::Plan.display_name(), "Plan");
    }

    #[test]
    fn test_agent_type_from_str() {
        assert_eq!(AgentType::from_str("main"), Some(AgentType::Main));
        assert_eq!(AgentType::from_str("explore"), Some(AgentType::Explore));
        assert_eq!(AgentType::from_str("EXPLORER"), Some(AgentType::Explore));
        assert_eq!(AgentType::from_str("invalid"), None);
    }

    #[test]
    fn test_main_capability() {
        let capability = AgentCapability::main_capability();
        assert_eq!(capability.agent_type, AgentType::Main);
        assert!(!capability.read_only);
        assert!(capability.tools.contains(&"read_file".to_string()));
        assert!(capability.tools.contains(&"write_file".to_string()));
    }

    #[test]
    fn test_explore_capability_read_only() {
        let capability = AgentCapability::explore_capability();
        assert_eq!(capability.agent_type, AgentType::Explore);
        assert!(capability.read_only);
        assert!(!capability.tools.contains(&"write_file".to_string()));
        assert!(!capability.tools.contains(&"edit_file".to_string()));
    }

    #[test]
    fn test_all_capabilities() {
        let capabilities = AgentCapability::all_capabilities();
        assert!(capabilities.contains_key(&AgentType::Main));
        assert!(capabilities.contains_key(&AgentType::Explore));
        assert!(capabilities.contains_key(&AgentType::Plan));
        assert!(capabilities.contains_key(&AgentType::CodeReviewer));
        assert!(capabilities.contains_key(&AgentType::FrontendDeveloper));
    }
}
