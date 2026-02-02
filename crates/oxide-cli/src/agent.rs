//! 代理模块 - 基于 rig-core 的工具调用代理
//!
//! 使用 rig Agent 处理工具调用循环，替代自实现的代理循环。

use anyhow::Result;
use indicatif::MultiProgress;
use oxide_core::config::PermissionsConfig;
use oxide_core::types::{ContentBlock, Message, Role};
use oxide_provider::RigAnthropicProvider;
use oxide_tools::{ConfirmationResult, PermissionManager, TaskManager};
use rig::completion::Prompt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::interaction::CliInteractionHandler;
use crate::render::StreamState;

/// 创建 CLI 确认回调
fn create_confirmation_callback(mp: Option<Arc<MultiProgress>>) -> oxide_tools::ConfirmationCallback {
    Arc::new(move |tool_name: String, args: serde_json::Value| {
        let mp = mp.clone();
        Box::pin(async move {
            use dialoguer::{theme::ColorfulTheme, Select};

            let theme = ColorfulTheme::default();

            // 提取工具参数的关键信息
            let detail = extract_tool_detail(&tool_name, &args);
            let prompt = if detail.is_empty() {
                format!("工具 '{}' 需要权限确认", tool_name)
            } else {
                format!("工具 '{}' 需要权限确认\n  {}", tool_name, detail)
            };

            let items = vec![
                "允许本次",
                "始终允许（本次会话）",
                "始终允许（记住选择）",
                "拒绝",
            ];

            let do_select = || {
                match Select::with_theme(&theme)
                    .with_prompt(&prompt)
                    .items(&items)
                    .default(0)
                    .interact()
                {
                    Ok(0) => ConfirmationResult::Allow,
                    Ok(1) => ConfirmationResult::AllowSession,
                    Ok(2) => ConfirmationResult::AllowAlways,
                    Ok(3) => ConfirmationResult::Deny,
                    Ok(_) => ConfirmationResult::Deny,
                    Err(_) => ConfirmationResult::Deny,
                }
            };

            // 如果有 MultiProgress，暂停所有进度条后再显示确认对话框
            if let Some(mp) = mp {
                mp.suspend(do_select)
            } else {
                do_select()
            }
        })
    })
}

/// 从工具参数中提取关键信息用于显示
fn extract_tool_detail(tool_name: &str, args: &serde_json::Value) -> String {
    match tool_name {
        "Bash" => {
            args.get("command")
                .and_then(|v| v.as_str())
                .map(|cmd| format!("命令: {}", truncate_str(cmd, 80)))
                .unwrap_or_default()
        }
        "Edit" | "Write" | "Read" => {
            args.get("file_path")
                .and_then(|v| v.as_str())
                .map(|path| format!("文件: {}", path))
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

/// 截断字符串
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// 创建持久化回调
fn create_persist_callback() -> oxide_tools::PersistCallback {
    Arc::new(|tool_name: String| {
        Box::pin(async move {
            use oxide_core::config::{config_path, Config};

            if config_path().is_ok() {
                if let Ok(mut config) = Config::load() {
                    if !config.permissions.allow.contains(&tool_name) {
                        config.permissions.allow.push(tool_name.clone());
                        let _ = config.save();
                    }
                }
            }
        })
    })
}

/// 创建权限管理器
fn create_permission_manager(config: PermissionsConfig, mp: Option<Arc<MultiProgress>>) -> PermissionManager {
    PermissionManager::new(config)
        .with_confirmation_callback(create_confirmation_callback(mp))
        .with_persist_callback(create_persist_callback())
}

use crate::render::StatusLine;

/// 基于 rig 的代理
pub struct RigAgentRunner {
    /// 工作目录
    working_dir: PathBuf,
    /// 任务管理器（用于后台任务）
    task_manager: TaskManager,
    /// 权限管理器
    permission_manager: PermissionManager,
    /// 权限配置（用于重新创建 permission_manager）
    permissions_config: PermissionsConfig,
    /// 系统提示词
    system_prompt: Option<String>,
    /// MultiProgress 管理器（用于输出）
    mp: Option<Arc<MultiProgress>>,
    /// Statusline（用于工具执行期间隐藏/恢复）
    statusline: Option<StatusLine>,
}

impl RigAgentRunner {
    /// 创建新的代理运行器
    pub fn new(working_dir: PathBuf) -> Self {
        let config = PermissionsConfig::default();
        Self {
            working_dir,
            task_manager: oxide_tools::rig_tools::create_task_manager(),
            permission_manager: create_permission_manager(config.clone(), None),
            permissions_config: config,
            system_prompt: None,
            mp: None,
            statusline: None,
        }
    }

    /// 创建新的代理运行器（带配置）
    pub fn new_with_config(working_dir: PathBuf, config: PermissionsConfig) -> Self {
        Self {
            working_dir,
            task_manager: oxide_tools::rig_tools::create_task_manager(),
            permission_manager: create_permission_manager(config.clone(), None),
            permissions_config: config,
            system_prompt: None,
            mp: None,
            statusline: None,
        }
    }

    /// 设置系统提示词
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }

    /// 设置 MultiProgress 管理器（用于输出）
    pub fn with_multi_progress(mut self, mp: Arc<MultiProgress>) -> Self {
        // 重新创建 permission_manager 以传入 mp
        self.permission_manager = create_permission_manager(self.permissions_config.clone(), Some(mp.clone()));
        self.mp = Some(mp);
        self
    }

    /// 设置任务管理器
    pub fn with_task_manager(mut self, task_manager: TaskManager) -> Self {
        self.task_manager = task_manager;
        self
    }

    /// 设置权限管理器
    pub fn with_permission_manager(mut self, permission_manager: PermissionManager) -> Self {
        self.permission_manager = permission_manager;
        self
    }

    /// 设置 statusline（用于工具执行期间隐藏/恢复）
    pub fn with_statusline(mut self, statusline: StatusLine) -> Self {
        self.statusline = Some(statusline);
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
            .permission_manager(self.permission_manager.clone())
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

        // 创建工具列表
        let mut tools = oxide_tools::rig_tools::OxideToolSetBuilder::new(self.working_dir.clone())
            .task_manager(self.task_manager.clone())
            .permission_manager(self.permission_manager.clone())
            .build_boxed();

        // 添加交互工具
        let ask_tool = oxide_tools::rig_tools::RigAskUserQuestionTool::new();
        ask_tool.set_handler(Arc::new(CliInteractionHandler::new())).await;
        tools.push(Box::new(oxide_tools::rig_tools::ToolWrapper::new(ask_tool)));

        // 创建 Agent
        let agent = provider.create_agent_with_tools(self.system_prompt.as_deref(), tools);

        // 构建提示（包含历史上下文）
        let prompt = if chat_history.is_empty() {
            user_input.to_string()
        } else {
            let history_context = self.format_chat_history(&chat_history);
            format!("{}\n\n用户: {}", history_context, user_input)
        };

        // 获取流式响应
        let mut stream = agent.stream_prompt(&prompt).multi_turn(10).await;

        // 创建流式渲染状态管理器
        let mut state = StreamState::new(self.mp.clone(), self.statusline.clone());
        let mut full_response = String::new();

        // 处理流式响应
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                    state.handle_text(&text.text);
                    full_response.push_str(&text.text);
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning))) => {
                    for r in reasoning.reasoning {
                        state.handle_reasoning(&r);
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ReasoningDelta { reasoning, .. })) => {
                    state.handle_reasoning(&reasoning);
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall(tool_call))) => {
                    let desc = extract_tool_description(&tool_call.function.name, &tool_call.function.arguments);
                    state.start_tool(&tool_call.function.name, desc);
                }
                Ok(MultiTurnStreamItem::StreamUserItem(rig::streaming::StreamedUserContent::ToolResult(_))) => {
                    state.finish_tool();
                }
                Ok(MultiTurnStreamItem::FinalResponse(final_res)) => {
                    full_response = final_res.response().to_string();
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow::anyhow!("流式输出错误: {}", e));
                }
            }
        }

        state.finish();
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

/// 从工具参数中提取关键描述信息
fn extract_tool_description(tool_name: &str, args: &serde_json::Value) -> String {
    match tool_name {
        "Read" => {
            // 提取文件路径
            args.get("file_path")
                .and_then(|v| v.as_str())
                .map(|s| truncate_path(s))
                .unwrap_or_default()
        }
        "Write" => {
            args.get("file_path")
                .and_then(|v| v.as_str())
                .map(|s| truncate_path(s))
                .unwrap_or_default()
        }
        "Edit" => {
            args.get("file_path")
                .and_then(|v| v.as_str())
                .map(|s| truncate_path(s))
                .unwrap_or_default()
        }
        "Glob" => {
            args.get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        "Grep" => {
            args.get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        "Bash" => {
            // 提取命令，截断过长的命令
            args.get("command")
                .and_then(|v| v.as_str())
                .map(|s| truncate_str(s, 60))
                .unwrap_or_default()
        }
        "TaskOutput" | "TaskStop" => {
            args.get("task_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        "AskUserQuestion" => {
            // 提取第一个问题
            args.get("questions")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|q| q.get("question"))
                .and_then(|v| v.as_str())
                .map(|s| truncate_str(s, 40))
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

/// 截断路径，保留文件名和部分目录
fn truncate_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 3 {
        path.to_string()
    } else {
        // 保留最后 3 个部分
        format!(".../{}", parts[parts.len() - 3..].join("/"))
    }
}
