//! 代理模块 - 基于 rig-core 的工具调用代理
//!
//! 使用 rig Agent 处理工具调用循环，替代自实现的代理循环。

use anyhow::Result;
use oxide_core::types::{ContentBlock, Message, Role};
use oxide_provider::RigAnthropicProvider;
use oxide_tools::TaskManager;
use rig::completion::Prompt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::interaction::CliInteractionHandler;
use crate::render::Renderer;

/// 基于 rig 的代理
pub struct RigAgentRunner {
    /// 工作目录
    working_dir: PathBuf,
    /// 任务管理器（用于后台任务）
    task_manager: TaskManager,
    /// 渲染器
    renderer: Renderer,
    /// 系统提示词
    system_prompt: Option<String>,
}

impl RigAgentRunner {
    /// 创建新的代理运行器
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            task_manager: oxide_tools::rig_tools::create_task_manager(),
            renderer: Renderer::new(),
            system_prompt: None,
        }
    }

    /// 设置系统提示词
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }

    /// 设置任务管理器
    pub fn with_task_manager(mut self, task_manager: TaskManager) -> Self {
        self.task_manager = task_manager;
        self
    }

    /// 获取任务管理器
    pub fn task_manager(&self) -> TaskManager {
        self.task_manager.clone()
    }

    /// 执行代理
    ///
    /// 使用 rig Agent 处理用户输入，自动处理工具调用循环
    pub async fn run(
        &self,
        provider: &RigAnthropicProvider,
        user_input: &str,
        chat_history: Vec<Message>,
    ) -> Result<String> {
        // 创建工具列表（boxed）
        let mut tools = oxide_tools::rig_tools::OxideToolSetBuilder::new(self.working_dir.clone())
            .task_manager(self.task_manager.clone())
            .build_boxed();

        // 添加交互工具并设置处理器
        let ask_tool = oxide_tools::rig_tools::RigAskUserQuestionTool::new();
        ask_tool.set_handler(Arc::new(CliInteractionHandler::new())).await;
        tools.push(Box::new(oxide_tools::rig_tools::ToolWrapper::new(ask_tool)));

        // 创建 rig Agent
        let agent = provider.create_agent_with_tools(
            self.system_prompt.as_deref(),
            tools,
        );

        // 构建完整的提示（包含历史上下文）
        let prompt = if chat_history.is_empty() {
            user_input.to_string()
        } else {
            // 将历史消息转换为上下文字符串
            let history_context = self.format_chat_history(&chat_history);
            format!("{}\n\n用户: {}", history_context, user_input)
        };

        // 调用 Agent（非流式）
        let response = agent.prompt(&prompt).await?;

        Ok(response)
    }

    /// 执行代理（流式输出）
    ///
    /// 使用 rig Agent 处理用户输入，支持流式输出
    pub async fn run_stream(
        &self,
        provider: &RigAnthropicProvider,
        user_input: &str,
        chat_history: Vec<Message>,
    ) -> Result<String> {
        use rig::streaming::StreamingPrompt;
        use rig::agent::MultiTurnStreamItem;
        use rig::streaming::StreamedAssistantContent;
        use futures::StreamExt;

        // 创建工具列表（boxed）
        let mut tools = oxide_tools::rig_tools::OxideToolSetBuilder::new(self.working_dir.clone())
            .task_manager(self.task_manager.clone())
            .build_boxed();

        // 添加交互工具并设置处理器
        let ask_tool = oxide_tools::rig_tools::RigAskUserQuestionTool::new();
        ask_tool.set_handler(Arc::new(CliInteractionHandler::new())).await;
        tools.push(Box::new(oxide_tools::rig_tools::ToolWrapper::new(ask_tool)));

        // 创建 rig Agent
        let agent = provider.create_agent_with_tools(
            self.system_prompt.as_deref(),
            tools,
        );

        // 构建完整的提示（包含历史上下文）
        let prompt = if chat_history.is_empty() {
            user_input.to_string()
        } else {
            // 将历史消息转换为上下文字符串
            let history_context = self.format_chat_history(&chat_history);
            format!("{}\n\n用户: {}", history_context, user_input)
        };

        // 获取流式响应
        let mut stream = agent.stream_prompt(&prompt).multi_turn(10).await;

        // 收集完整响应
        let mut full_response = String::new();

        // 逐块处理流式输出
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                    // 打印文本到终端
                    print!("{}", text.text);
                    use std::io::Write;
                    let _ = std::io::stdout().flush();

                    // 收集完整响应
                    full_response.push_str(&text.text);
                }
                Ok(MultiTurnStreamItem::FinalResponse(final_res)) => {
                    // 最终响应
                    full_response = final_res.response().to_string();
                }
                Ok(_) => {
                    // 忽略其他类型（工具调用等）
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("流式输出错误: {}", e));
                }
            }
        }

        println!(); // 换行

        Ok(full_response)
    }

    /// 格式化聊天历史为上下文字符串
    fn format_chat_history(&self, messages: &[Message]) -> String {
        let mut context = String::new();

        for msg in messages {
            let role_str = match msg.role {
                Role::User => "用户",
                Role::Assistant => "助手",
                Role::System => continue, // 跳过系统消息
            };

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        context.push_str(&format!("{}: {}\n\n", role_str, text));
                    }
                    ContentBlock::ToolUse { name, .. } => {
                        context.push_str(&format!("{}: [调用工具: {}]\n\n", role_str, name));
                    }
                    ContentBlock::ToolResult { content, is_error, .. } => {
                        let status = if *is_error { "错误" } else { "结果" };
                        // 截断过长的工具结果
                        let truncated = if content.len() > 500 {
                            format!("{}... (已截断)", &content[..500])
                        } else {
                            content.clone()
                        };
                        context.push_str(&format!("[工具{}]: {}\n\n", status, truncated));
                    }
                    ContentBlock::Image { .. } => {
                        // 跳过图片内容
                        context.push_str(&format!("{}: [图片]\n\n", role_str));
                    }
                }
            }
        }

        context
    }
}

/// 创建工具集的任务管理器
pub fn create_task_manager() -> TaskManager {
    oxide_tools::rig_tools::create_task_manager()
}

// ============================================================================
// 兼容层：保留旧的 Agent 接口以便逐步迁移
// ============================================================================

use oxide_tools::ToolRegistry;

/// 旧版代理（兼容层）
///
/// 注意：此实现已弃用，请使用 RigAgentRunner
#[deprecated(note = "请使用 RigAgentRunner")]
pub struct Agent {
    tool_registry: Arc<ToolRegistry>,
    renderer: Renderer,
}

#[allow(deprecated)]
impl Agent {
    /// 创建新的代理
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            tool_registry,
            renderer: Renderer::new(),
        }
    }

    /// 执行代理循环（兼容旧接口）
    pub async fn run(
        &mut self,
        _provider: Arc<dyn oxide_provider::LLMProvider>,
        _messages: &mut Vec<Message>,
        _stream_callback: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    ) -> Result<Message> {
        // 旧接口不再支持，返回错误提示使用新接口
        anyhow::bail!("旧版 Agent 已弃用，请使用 RigAgentRunner")
    }
}

/// 创建工具注册表并注册所有工具（兼容旧接口）
#[deprecated(note = "请使用 oxide_tools::create_oxide_toolset")]
pub fn create_tool_registry(working_dir: PathBuf) -> Arc<ToolRegistry> {
    let mut registry = ToolRegistry::new();

    // 创建共享的任务管理器
    let task_manager = oxide_tools::create_task_manager();

    // 注册文件操作工具
    registry.register(Arc::new(oxide_tools::ReadTool::new(working_dir.clone())));
    registry.register(Arc::new(oxide_tools::WriteTool::new(working_dir.clone())));
    registry.register(Arc::new(oxide_tools::EditTool::new(working_dir.clone())));

    // 注册搜索工具
    registry.register(Arc::new(oxide_tools::GlobTool::new(working_dir.clone())));
    registry.register(Arc::new(oxide_tools::GrepTool::new(working_dir.clone())));

    // 注册执行工具（共享任务管理器）
    registry.register(Arc::new(oxide_tools::BashTool::with_task_manager(
        working_dir.clone(),
        task_manager.clone(),
    )));
    registry.register(Arc::new(oxide_tools::TaskOutputTool::new(task_manager.clone())));
    registry.register(Arc::new(oxide_tools::TaskStopTool::new(task_manager)));

    Arc::new(registry)
}
