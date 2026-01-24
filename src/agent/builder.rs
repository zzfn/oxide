//! Agent 构建器
//!
//! 根据不同的 Agent 类型创建具有相应工具权限的 Agent 实例。

use crate::agent::types::AgentType;
use crate::tools::{
    WrappedCreateDirectoryTool, WrappedDeleteFileTool, WrappedEditFileTool,
    WrappedGlobTool, WrappedGrepSearchTool, WrappedReadFileTool,
    WrappedScanCodebaseTool, WrappedWriteFileTool, WrappedShellExecuteTool,
};
use anyhow::Result;
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::providers::{anthropic, openai};

/// Agent 构建器
///
/// 根据指定的 Agent 类型创建相应的 Agent 实例,配置对应的系统提示词和工具权限。
pub struct AgentBuilder {
    /// API 基础 URL
    base_url: String,

    /// API 认证令牌
    auth_token: String,

    /// 模型名称(可选)
    model: Option<String>,
}

impl AgentBuilder {
    /// 创建新的 Agent 构建器
    pub fn new(base_url: String, auth_token: String, model: Option<String>) -> Self {
        Self {
            base_url,
            auth_token,
            model,
        }
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
                .api_key(&self.auth_token)
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("Your name is Oxide. You are a helpful AI code assistant with comprehensive file system and command execution access. You can read, write, edit (with patches), and delete files, execute bash commands, scan codebase structures, search text in the codebase and create directories. Use the edit_file tool for making small, targeted changes to existing files - it's more efficient than rewriting entire files. Please provide clear and concise responses and be careful when modifying files or executing commands.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.write_file)
                .tool(tools.edit_file)
                .tool(tools.delete_file)
                .tool(tools.shell_execute)
                .tool(tools.scan_codebase)
                .tool(tools.make_dir)
                .tool(tools.grep_find)
                .tool(tools.glob)
                .build();

            Ok(AgentEnum::Anthropic(agent))
        } else {
            let client = openai::Client::builder()
                .api_key(&self.auth_token)
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("Your name is Oxide. You are a helpful AI code assistant with comprehensive file system and command execution access. You can read, write, edit (with patches), and delete files, execute bash commands, scan codebase structures, search text in the codebase and create directories. Use the edit_file tool for making small, targeted changes to existing files - it's more efficient than rewriting entire files. Please provide clear and concise responses and be careful when modifying files or executing commands.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.write_file)
                .tool(tools.edit_file)
                .tool(tools.delete_file)
                .tool(tools.shell_execute)
                .tool(tools.scan_codebase)
                .tool(tools.make_dir)
                .tool(tools.grep_find)
                .tool(tools.glob)
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
                .api_key(&self.auth_token)
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
                .api_key(&self.auth_token)
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
                .api_key(&self.auth_token)
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
                .api_key(&self.auth_token)
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
                .api_key(&self.auth_token)
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
                .api_key(&self.auth_token)
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
                .api_key(&self.auth_token)
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Frontend Developer Agent specialized in building modern, production-grade user interfaces. Your expertise includes: - React, Next.js, Vue, Svelte, and other modern frameworks - Tailwind CSS, shadcn/ui, and component libraries - Responsive design and accessibility - Performance optimization - Creating polished, maintainable code that avoids generic AI aesthetics. When building UI components, prioritize user experience, maintainability, and web standards compliance.")
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
                .api_key(&self.auth_token)
                .base_url(&self.base_url)
                .build()?;

            let agent = client
                .agent(&model_name)
                .preamble("You are a Frontend Developer Agent specialized in building modern, production-grade user interfaces. Your expertise includes: - React, Next.js, Vue, Svelte, and other modern frameworks - Tailwind CSS, shadcn/ui, and component libraries - Responsive design and accessibility - Performance optimization - Creating polished, maintainable code that avoids generic AI aesthetics. When building UI components, prioritize user experience, maintainability, and web standards compliance.")
                .max_tokens(4096)
                .tool(tools.read_file)
                .tool(tools.write_file)
                .tool(tools.edit_file)
                .tool(tools.shell_execute)
                .tool(tools.grep_find)
                .tool(tools.glob)
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
        AllTools {
            read_file: WrappedReadFileTool::new(),
            write_file: WrappedWriteFileTool::new(),
            edit_file: WrappedEditFileTool::new(),
            delete_file: WrappedDeleteFileTool::new(),
            shell_execute: WrappedShellExecuteTool::new(),
            scan_codebase: WrappedScanCodebaseTool::new(),
            make_dir: WrappedCreateDirectoryTool::new(),
            grep_find: WrappedGrepSearchTool::new(),
            glob: WrappedGlobTool::new(),
        }
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

impl AgentEnum {
    /// 获取 Agent 类型的名称
    pub fn type_name(&self) -> &'static str {
        match self {
            AgentEnum::Anthropic(_) => "claude",
            AgentEnum::OpenAI(_) => "openai",
        }
    }
}

/// 便捷函数:创建指定类型的 Agent
#[allow(dead_code)]
pub fn create_agent_of_type(
    agent_type: AgentType,
    base_url: String,
    auth_token: String,
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
            "test-key".to_string(),
            None,
        );

        assert_eq!(builder.base_url, "https://api.anthropic.com");
        assert_eq!(builder.auth_token, "test-key");
        assert!(builder.model.is_none());
    }

    #[test]
    fn test_agent_builder_with_custom_model() {
        let builder = AgentBuilder::new(
            "https://api.anthropic.com".to_string(),
            "test-key".to_string(),
            Some("claude-opus-4-20250514".to_string()),
        );

        assert_eq!(
            builder.model,
            Some("claude-opus-4-20250514".to_string())
        );
    }

    // 注意: 实际的 build 测试需要有效的 API 凭据,这里我们只测试结构
}
