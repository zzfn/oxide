//! Agent 构建器
//!
//! 根据不同的 Agent 类型创建具有相应工具权限的 Agent 实例。

use crate::agent::{HitlIntegration, MaybeHitlTool};
use crate::agent::types::AgentType;
use crate::config::secret::Secret;
use crate::tools::{
    WrappedAskUserQuestionTool, WrappedCreateDirectoryTool, WrappedDeleteFileTool,
    WrappedEditFileTool, WrappedGlobTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedWriteFileTool, WrappedShellExecuteTool,
    WrappedSearchReplaceTool, WrappedEnterPlanModeTool, WrappedExitPlanModeTool,
};
use anyhow::Result;
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::providers::{anthropic, openai};
use std::sync::Arc;

use crate::agent::workflow::observation::ObservationCollector;

/// Agent 构建器
///
/// 根据指定的 Agent 类型创建相应的 Agent 实例,配置对应的系统提示词和工具权限。
pub struct AgentBuilder {
    /// API 基础 URL
    base_url: String,

    /// API 认证令牌 (使用 Secret 保护)
    auth_token: Secret<String>,

    /// 模型名称(可选)
    model: Option<String>,

    /// HITL 集成 (可选)
    hitl: Option<Arc<HitlIntegration>>,

    /// 观察数据收集器 (可选)
    observation_collector: Option<ObservationCollector>,
}

impl AgentBuilder {
    /// 创建新的 Agent 构建器
    pub fn new(base_url: String, auth_token: Secret<String>, model: Option<String>) -> Self {
        Self {
            base_url,
            auth_token,
            model,
            hitl: None,
            observation_collector: None,
        }
    }

    /// 设置 HITL 集成
    pub fn with_hitl(mut self, hitl: Arc<HitlIntegration>) -> Self {
        self.hitl = Some(hitl);
        self
    }

    /// 设置观察收集器
    pub fn with_observations(mut self, collector: ObservationCollector) -> Self {
        self.observation_collector = Some(collector);
        self
    }

    /// 构建 Main Agent(拥有所有工具)
    pub fn build_main(&self) -> Result<AgentEnum> {
        let tools = self.create_tools();
        let model_name = self
            .model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        if self.base_url.contains("/anthropic") || self.base_url.contains("anthropic.com") {
            let client = anthropic::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble(r#"Your name is Oxide. You are a helpful AI code assistant with comprehensive file system and command execution access. You can read, write, edit (with patches or search/replace), and delete files, execute bash commands, scan codebase structures, search text in the codebase and create directories. Use edit_file for precise small changes with diffs. Use search_replace for block replacements where you match content rather than lines (robust to line number shifts). search_replace is preferred for modifying functions or blocks of code. Please provide clear and concise responses and be careful when modifying files or executing commands.

【Tool Usage Strategy】
- ✅ WHEN to use tools: When users explicitly request file operations, code search, command execution, or system interactions
- ❌ WHEN NOT to use tools: For general conversation, capability questions, or questions that can be answered directly from your knowledge
- 🤖 Answer directly: Questions about your capabilities, features, technical concepts, or general programming questions should be answered directly without calling tools
- 📋 Read first: ALWAYS read files before attempting to edit them to ensure you have the current content

【Plan Mode】
Use enter_plan_mode proactively when you're about to start a non-trivial implementation task:
- New feature implementation requiring architectural decisions
- Multiple valid approaches exist for the task
- Code modifications affecting existing behavior
- Multi-file changes (more than 2-3 files)
- Unclear requirements needing exploration

In plan mode:
1. Explore the codebase using read, grep, glob tools
2. Design your implementation approach
3. Use exit_plan_mode to present your plan and request user approval
4. Only proceed with implementation after user approves

Skip plan mode for simple tasks like typo fixes, single-line changes, or tasks with very specific instructions.

【User Interaction】
Use ask_user_question when you need to:
- Gather user preferences or requirements during execution
- Clarify ambiguous instructions
- Get decisions on implementation choices
- Offer choices about what direction to take
Users can always select "Other" to provide custom input. Use multiSelect: true to allow multiple answers."#)
                .max_tokens(4096)
                .tool(MaybeHitlTool::new(tools.read_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.write_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.edit_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.delete_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.shell_execute, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.scan_codebase, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.make_dir, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.grep_find, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.glob, self.hitl.clone()))
                .tool(tools.enter_plan_mode)
                .tool(tools.exit_plan_mode)
                .tool(tools.ask_user_question)
                .build();

            Ok(AgentEnum::Anthropic(agent))
        } else {
            let client = openai::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble(r#"Your name is Oxide. You are a helpful AI code assistant with comprehensive file system and command execution access. You can read, write, edit (with patches or search/replace), and delete files, execute bash commands, scan codebase structures, search text in the codebase and create directories. Use edit_file for precise small changes with diffs. Use search_replace for block replacements where you match content rather than lines (robust to line number shifts). search_replace is preferred for modifying functions or blocks of code. Please provide clear and concise responses and be careful when modifying files or executing commands.

【Tool Usage Strategy】
- ✅ WHEN to use tools: When users explicitly request file operations, code search, command execution, or system interactions
- ❌ WHEN NOT to use tools: For general conversation, capability questions, or questions that can be answered directly from your knowledge
- 🤖 Answer directly: Questions about your capabilities, features, technical concepts, or general programming questions should be answered directly without calling tools
- 📋 Read first: ALWAYS read files before attempting to edit them to ensure you have the current content

【Plan Mode】
Use enter_plan_mode proactively when you're about to start a non-trivial implementation task:
- New feature implementation requiring architectural decisions
- Multiple valid approaches exist for the task
- Code modifications affecting existing behavior
- Multi-file changes (more than 2-3 files)
- Unclear requirements needing exploration

In plan mode:
1. Explore the codebase using read, grep, glob tools
2. Design your implementation approach
3. Use exit_plan_mode to present your plan and request user approval
4. Only proceed with implementation after user approves

Skip plan mode for simple tasks like typo fixes, single-line changes, or tasks with very specific instructions.

【User Interaction】
Use ask_user_question when you need to:
- Gather user preferences or requirements during execution
- Clarify ambiguous instructions
- Get decisions on implementation choices
- Offer choices about what direction to take
Users can always select "Other" to provide custom input. Use multiSelect: true to allow multiple answers."#)
                .max_tokens(4096)
                .tool(MaybeHitlTool::new(tools.read_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.write_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.edit_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.delete_file, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.shell_execute, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.scan_codebase, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.make_dir, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.grep_find, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.glob, self.hitl.clone()))
                .tool(MaybeHitlTool::new(tools.search_replace, self.hitl.clone()))
                .tool(tools.enter_plan_mode)
                .tool(tools.exit_plan_mode)
                .tool(tools.ask_user_question)
                .build();

            Ok(AgentEnum::OpenAI(agent))
        }
    }

    /// 构建 Explore Agent(只读工具)
    #[allow(dead_code)]
    pub fn build_explore(&self) -> Result<AgentEnum> {
        let tools = self.create_tools();
        let model_name = self
            .model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        if self.base_url.contains("/anthropic") || self.base_url.contains("anthropic.com") {
            let client = anthropic::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are an Explore Agent specialized in codebase exploration and analysis. Your capabilities are limited to read-only operations: reading files, searching text, and scanning the codebase structure. When exploring a codebase: 1. Start by getting an overview of the project structure 2. Identify key files and directories 3. Search for relevant code patterns 4. Provide concise summaries of your findings. Use Glob for file pattern matching and Grep for content searching.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.grep_find)
                .tool(tools.scan_codebase)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::Anthropic(agent))
        } else {
            let client = openai::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are an Explore Agent specialized in codebase exploration and analysis. Your capabilities are limited to read-only operations: reading files, searching text, and scanning the codebase structure. When exploring a codebase: 1. Start by getting an overview of the project structure 2. Identify key files and directories 3. Search for relevant code patterns 4. Provide concise summaries of your findings. Use Glob for file pattern matching and Grep for content searching.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.grep_find)
                .tool(tools.scan_codebase)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::OpenAI(agent))
        }
    }

    /// 构建 Plan Agent
    #[allow(dead_code)]
    pub fn build_plan(&self) -> Result<AgentEnum> {
        let tools = self.create_tools();
        let model_name = self
            .model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        if self.base_url.contains("/anthropic") || self.base_url.contains("anthropic.com") {
            let client = anthropic::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Plan Agent specialized in software architecture and implementation planning. Your role is to: 1. Analyze requirements and explore the codebase 2. Design implementation strategies 3. Break down complex tasks into manageable steps 4. Identify potential issues and trade-offs 5. Create clear, actionable plans. When planning, be thorough but focus on practical, implementable solutions.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.grep_find)
                .tool(tools.scan_codebase)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::Anthropic(agent))
        } else {
            let client = openai::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Plan Agent specialized in software architecture and implementation planning. Your role is to: 1. Analyze requirements and explore the codebase 2. Design implementation strategies 3. Break down complex tasks into manageable steps 4. Identify potential issues and trade-offs 5. Create clear, actionable plans. When planning, be thorough but focus on practical, implementable solutions.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.grep_find)
                .tool(tools.scan_codebase)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::OpenAI(agent))
        }
    }

    /// 构建 Code Reviewer Agent(只读工具)
    #[allow(dead_code)]
    pub fn build_code_reviewer(&self) -> Result<AgentEnum> {
        let tools = self.create_tools();
        let model_name = self
            .model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        if self.base_url.contains("/anthropic") || self.base_url.contains("anthropic.com") {
            let client = anthropic::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Code Reviewer Agent specialized in code quality analysis and security review. Your responsibilities include: 1. Reviewing code for bugs and logic errors 2. Identifying security vulnerabilities (OWASP Top 10, injection attacks, etc.) 3. Checking for code quality issues and maintainability problems 4. Verifying adherence to project conventions 5. Suggesting improvements and best practices. Focus on high-priority issues that truly matter. Be constructive and specific in your feedback.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.grep_find)
                .tool(tools.scan_codebase)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::Anthropic(agent))
        } else {
            let client = openai::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Code Reviewer Agent specialized in code quality analysis and security review. Your responsibilities include: 1. Reviewing code for bugs and logic errors 2. Identifying security vulnerabilities (OWASP Top 10, injection attacks, etc.) 3. Checking for code quality issues and maintainability problems 4. Verifying adherence to project conventions 5. Suggesting improvements and best practices. Focus on high-priority issues that truly matter. Be constructive and specific in your feedback.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.grep_find)
                .tool(tools.scan_codebase)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::OpenAI(agent))
        }
    }

    /// 构建 Frontend Developer Agent
    #[allow(dead_code)]
    pub fn build_frontend_developer(&self) -> Result<AgentEnum> {
        let tools = self.create_tools();
        let model_name = self
            .model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

        if self.base_url.contains("/anthropic") || self.base_url.contains("anthropic.com") {
            let client = anthropic::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Frontend Developer Agent specialized in building modern, production-grade user interfaces. Your expertise includes: - React, Next.js, Vue, Svelte, and other modern frameworks - Tailwind CSS, shadcn/ui, and component libraries - Responsive design and accessibility - Performance optimization - Creating polished, maintainable code that avoids generic AI aesthetics. When building UI components, prioritize user experience, maintainability, and web standards compliance. Use search_replace for safe block replacements when strict line numbers are unknown.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.write_file)
                .tool(tools.edit_file)
                .tool(tools.shell_execute)
                .tool(tools.grep_find)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::Anthropic(agent))
        } else {
            let client = openai::Client::builder()
                .api_key(self.auth_token.expose_secret())
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Frontend Developer Agent specialized in building modern, production-grade user interfaces. Your expertise includes: - React, Next.js, Vue, Svelte, and other modern frameworks - Tailwind CSS, shadcn/ui, and component libraries - Responsive design and accessibility - Performance optimization - Creating polished, maintainable code that avoids generic AI aesthetics. When building UI components, prioritize user experience, maintainability, and web standards compliance. Use search_replace for safe block replacements when strict line numbers are unknown.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.write_file)
                .tool(tools.edit_file)
                .tool(tools.shell_execute)
                .tool(tools.grep_find)
                .tool(tools.glob)
                .tool(tools.search_replace)
                .build();

            Ok(AgentEnum::OpenAI(agent))
        }
    }

    /// 根据指定的 Agent 类型构建对应的 Agent
    #[allow(dead_code)]
    pub fn build_with_type(&self, agent_type: AgentType) -> Result<AgentEnum> {
        match agent_type {
            AgentType::Main => self.build_main(),
            AgentType::Explore => self.build_explore(),
            AgentType::Plan => self.build_plan(),
            AgentType::CodeReviewer => self.build_code_reviewer(),
            AgentType::FrontendDeveloper => self.build_frontend_developer(),
            AgentType::General => self.build_main(), // General 使用 Main 的配置
        }
    }

    /// 创建所有可用的工具
    fn create_tools(&self) -> AllTools {
        let tools = AllTools {
            read_file: WrappedReadFileTool::new(),
            write_file: WrappedWriteFileTool::new(),
            edit_file: WrappedEditFileTool::new(),
            delete_file: WrappedDeleteFileTool::new(),
            shell_execute: WrappedShellExecuteTool::new(),
            scan_codebase: WrappedScanCodebaseTool::new(),
            make_dir: WrappedCreateDirectoryTool::new(),
            grep_find: WrappedGrepSearchTool::new(),
            glob: WrappedGlobTool::new(),
            search_replace: WrappedSearchReplaceTool::new(),
            enter_plan_mode: WrappedEnterPlanModeTool::new(),
            exit_plan_mode: WrappedExitPlanModeTool::new(),
            ask_user_question: WrappedAskUserQuestionTool::new(),
        };

        // 如果启用了 HITL，则包装工具
        if let Some(_hitl) = &self.hitl {
            // 注意：这里由于类型不同，不能简单的重新赋值，除非 AllTools 也支持泛型包装器。
            // 为了简化实现，我们暂且认为只有 Main Agent 环境会启用 HITL，
            // 且我们只包装那些高风险工具。
            // 在实际代码中，这可能需要更复杂的泛型处理或 trait object。
        }

        tools
    }
}

/// 所有可用的工具
struct AllTools {
    read_file: WrappedReadFileTool,
    write_file: WrappedWriteFileTool,
    edit_file: WrappedEditFileTool,
    delete_file: WrappedDeleteFileTool,
    shell_execute: WrappedShellExecuteTool,
    scan_codebase: WrappedScanCodebaseTool,
    make_dir: WrappedCreateDirectoryTool,
    grep_find: WrappedGrepSearchTool,
    glob: WrappedGlobTool,
    search_replace: WrappedSearchReplaceTool,
    enter_plan_mode: WrappedEnterPlanModeTool,
    exit_plan_mode: WrappedExitPlanModeTool,
    ask_user_question: WrappedAskUserQuestionTool,
}

/// Agent 枚举 - 支持多种客户端
///
/// 这个枚举包装了不同类型的 Agent 实例，允许统一处理来自不同 LLM 提供商的 Agent。
pub enum AgentEnum {
    /// Anthropic Claude Agent
    Anthropic(Agent<anthropic::completion::CompletionModel>),

    /// OpenAI 兼容 Agent
    OpenAI(Agent<openai::responses_api::ResponsesCompletionModel>),
}

// 手动实现 Debug，避免暴露内部 Agent 细节
impl std::fmt::Debug for AgentEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentEnum::Anthropic(_) => f.debug_tuple("AgentEnum::Anthropic").field(&"...").finish(),
            AgentEnum::OpenAI(_) => f.debug_tuple("AgentEnum::OpenAI").field(&"...").finish(),
        }
    }
}

/// 便捷函数:创建指定类型的 Agent
#[allow(dead_code)]
pub fn create_agent_of_type(
    agent_type: AgentType,
    base_url: String,
    auth_token: Secret<String>,
    model: Option<String>,
) -> Result<AgentEnum> {
    let builder = AgentBuilder::new(base_url, auth_token, model);
    builder.build_with_type(agent_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder_creation() {
        let builder = AgentBuilder::new(
            "https://api.anthropic.com".to_string(),
            Secret::new("test-key".to_string()),
            None,
        );

        assert_eq!(builder.base_url, "https://api.anthropic.com");
        assert_eq!(builder.auth_token.expose_secret(), "test-key");
        assert!(builder.model.is_none());
    }

    #[test]
    fn test_agent_builder_with_custom_model() {
        let builder = AgentBuilder::new(
            "https://api.anthropic.com".to_string(),
            Secret::new("test-key".to_string()),
            Some("claude-opus-4-20250514".to_string()),
        );

        assert_eq!(
            builder.model,
            Some("claude-opus-4-20250514".to_string())
        );
    }

    // 注意: 实际的 build 测试需要有效的 API 凭据,这里我们只测试结构
}
