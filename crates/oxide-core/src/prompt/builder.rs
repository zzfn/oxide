//! 提示词构建器
//!
//! 使用 Builder 模式组装最终的系统提示词。

use std::path::PathBuf;

use super::context::RuntimeContext;
use super::parts::PromptPart;
use super::tool::ToolDefinition;
use crate::config::load_instructions;

/// 提示词构建器
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    /// 系统提示词片段（按顺序拼接）
    system_parts: Vec<PromptPart>,
    /// 工具定义列表
    tools: Vec<ToolDefinition>,
    /// 运行时上下文
    context: Option<RuntimeContext>,
    /// 用户指令（OXIDE.md）
    user_instructions: Option<String>,
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            system_parts: Vec::new(),
            tools: Vec::new(),
            context: None,
            user_instructions: None,
        }
    }

    /// 添加核心系统指令
    pub fn with_core_instructions(mut self) -> Self {
        self.system_parts.push(PromptPart::Core);
        self.system_parts.push(PromptPart::Tone);
        self.system_parts.push(PromptPart::Objectivity);
        self
    }

    /// 添加安全指令
    pub fn with_security(mut self) -> Self {
        self.system_parts.push(PromptPart::Security);
        self
    }

    /// 添加任务管理指令
    pub fn with_task_management(mut self) -> Self {
        self.system_parts.push(PromptPart::TaskManagement);
        self
    }

    /// 添加执行任务指南
    pub fn with_doing_tasks(mut self) -> Self {
        self.system_parts.push(PromptPart::DoingTasks);
        self
    }

    /// 添加工具使用策略
    pub fn with_tool_policy(mut self) -> Self {
        self.system_parts.push(PromptPart::ToolPolicy);
        self
    }

    /// 添加 Git 操作指南
    pub fn with_git_operations(mut self) -> Self {
        self.system_parts.push(PromptPart::GitOperations);
        self
    }

    /// 添加指定的提示词片段
    pub fn with_part(mut self, part: PromptPart) -> Self {
        self.system_parts.push(part);
        self
    }

    /// 添加多个提示词片段
    pub fn with_parts(mut self, parts: Vec<PromptPart>) -> Self {
        self.system_parts.extend(parts);
        self
    }

    /// 添加自定义提示词
    pub fn with_custom(mut self, content: impl Into<String>) -> Self {
        self.system_parts.push(PromptPart::Custom(content.into()));
        self
    }

    /// 添加工具定义
    pub fn with_tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.push(tool);
        self
    }

    /// 批量添加工具
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// 设置运行时上下文
    pub fn with_context(mut self, context: RuntimeContext) -> Self {
        self.context = Some(context);
        self
    }

    /// 加载用户指令（全局 + 项目 OXIDE.md）
    pub fn with_user_instructions(mut self, project_dir: &PathBuf) -> Self {
        if let Ok(instructions) = load_instructions(project_dir) {
            if !instructions.is_empty() {
                self.user_instructions = Some(instructions);
            }
        }
        self
    }

    /// 直接设置用户指令内容
    pub fn with_user_instructions_content(mut self, content: impl Into<String>) -> Self {
        let content = content.into();
        if !content.is_empty() {
            self.user_instructions = Some(content);
        }
        self
    }

    /// 使用默认配置（包含所有标准指令）
    pub fn default_agent() -> Self {
        Self::new()
            .with_core_instructions()
            .with_security()
            .with_task_management()
            .with_doing_tasks()
            .with_tool_policy()
            .with_git_operations()
    }

    /// 使用轻量配置（仅核心指令）
    pub fn lightweight() -> Self {
        Self::new().with_core_instructions().with_security()
    }

    /// 构建最终的系统提示词
    pub fn build(self) -> BuiltPrompt {
        let mut system_prompt = String::new();

        // 1. 拼接系统指令片段
        for part in &self.system_parts {
            let content = part.content();
            if !content.is_empty() {
                system_prompt.push_str(&content);
                system_prompt.push_str("\n\n");
            }
        }

        // 2. 添加工具定义
        if !self.tools.is_empty() {
            system_prompt.push_str("# Tools\n\n");
            for tool in &self.tools {
                system_prompt.push_str(&tool.to_prompt_section());
                system_prompt.push_str("\n\n---\n\n");
            }
        }

        // 3. 注入环境信息
        if let Some(ref context) = self.context {
            system_prompt.push_str(&context.to_env_section());
            system_prompt.push_str("\n\n");
        }

        // 4. 添加用户指令
        if let Some(ref instructions) = self.user_instructions {
            system_prompt.push_str("<system-reminder>\n");
            system_prompt
                .push_str("As you answer the user's questions, you can use the following context:\n");
            system_prompt.push_str("# User Instructions\n\n");
            system_prompt.push_str(instructions);
            system_prompt.push_str("\n</system-reminder>");
        }

        BuiltPrompt {
            system: system_prompt,
            tools: self.tools,
        }
    }
}

/// 构建完成的提示词
#[derive(Debug, Clone)]
pub struct BuiltPrompt {
    /// 系统提示词
    pub system: String,
    /// 工具定义列表
    pub tools: Vec<ToolDefinition>,
}

impl BuiltPrompt {
    /// 获取系统提示词
    pub fn system_prompt(&self) -> &str {
        &self.system
    }

    /// 获取工具定义（API 格式）
    pub fn tools_for_api(&self) -> Vec<serde_json::Value> {
        self.tools.iter().map(|t| t.to_api_format()).collect()
    }

    /// 估算 token 数量（粗略估计）
    ///
    /// 使用简单的字符数估算，每 4 个字符约 1 个 token
    pub fn estimated_tokens(&self) -> usize {
        self.system.len() / 4
    }

    /// 获取系统提示词长度（字符数）
    pub fn system_len(&self) -> usize {
        self.system.len()
    }

    /// 检查是否包含工具
    pub fn has_tools(&self) -> bool {
        !self.tools.is_empty()
    }

    /// 获取工具数量
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = PromptBuilder::new();
        let prompt = builder.build();
        assert!(prompt.system.is_empty() || prompt.system.trim().is_empty());
    }

    #[test]
    fn test_builder_default_agent() {
        let prompt = PromptBuilder::default_agent().build();
        assert!(!prompt.system.is_empty());
        assert!(prompt.system.contains("interactive CLI tool"));
    }

    #[test]
    fn test_builder_lightweight() {
        let prompt = PromptBuilder::lightweight().build();
        assert!(!prompt.system.is_empty());
        // 轻量模式不包含 Git 操作
        assert!(!prompt.system.contains("Committing changes with git"));
    }

    #[test]
    fn test_builder_with_context() {
        let context = RuntimeContext {
            working_dir: PathBuf::from("/test/project"),
            is_git_repo: true,
            platform: "darwin".to_string(),
            os_version: "Darwin 24.0".to_string(),
            today: "2025-01-31".to_string(),
            model_name: "Claude".to_string(),
            model_id: "claude-sonnet-4".to_string(),
        };

        let prompt = PromptBuilder::new()
            .with_core_instructions()
            .with_context(context)
            .build();

        assert!(prompt.system.contains("/test/project"));
        assert!(prompt.system.contains("git repo: Yes"));
    }

    #[test]
    fn test_builder_with_tools() {
        let tool = ToolDefinition::new(
            "TestTool",
            "A test tool",
            serde_json::json!({"type": "object"}),
        );

        let prompt = PromptBuilder::new().with_tool(tool).build();

        assert!(prompt.system.contains("# Tools"));
        assert!(prompt.system.contains("## TestTool"));
        assert_eq!(prompt.tool_count(), 1);
    }

    #[test]
    fn test_builder_with_custom() {
        let prompt = PromptBuilder::new()
            .with_custom("# Custom Instructions\n\nDo something special.")
            .build();

        assert!(prompt.system.contains("Custom Instructions"));
        assert!(prompt.system.contains("Do something special"));
    }

    #[test]
    fn test_builder_with_user_instructions() {
        let prompt = PromptBuilder::new()
            .with_user_instructions_content("Always respond in Chinese.")
            .build();

        assert!(prompt.system.contains("<system-reminder>"));
        assert!(prompt.system.contains("Always respond in Chinese"));
    }

    #[test]
    fn test_built_prompt_methods() {
        let prompt = PromptBuilder::default_agent().build();

        assert!(!prompt.system_prompt().is_empty());
        assert!(prompt.estimated_tokens() > 0);
        assert!(prompt.system_len() > 0);
        assert!(!prompt.has_tools());
        assert_eq!(prompt.tool_count(), 0);
    }

    #[test]
    fn test_tools_for_api() {
        let tool = ToolDefinition::new("Test", "Test tool", serde_json::json!({}));
        let prompt = PromptBuilder::new().with_tool(tool).build();

        let api_tools = prompt.tools_for_api();
        assert_eq!(api_tools.len(), 1);
        assert_eq!(api_tools[0]["name"], "Test");
    }
}
