//! HITL 集成示例
//!
//! 展示如何在现有的 Agent 中集成 HITL Gatekeeper

#![allow(dead_code)]

use crate::agent::hitl_gatekeeper::{HitlConfig, HitlDecision, HitlGatekeeper, ToolCallRequest, OperationContext, WarningLevel};
use crate::tools::ask_user_question::{WrappedAskUserQuestionTool, QuestionOption};
use rig::tool::Tool;
use colored::*;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// HITL 集成示例
///
/// 展示如何在主 Agent 的工具调用流程中集成 HITL Gatekeeper
pub struct HitlIntegration {
    pub gatekeeper: HitlGatekeeper,
    pub ask_user_tool: WrappedAskUserQuestionTool,
}

impl HitlIntegration {
    /// 创建新的 HITL 集成实例
    pub fn new() -> Result<Self> {
        let config = HitlConfig {
            trust: crate::agent::hitl_gatekeeper::TrustConfig::default(),
        };
        let gatekeeper = HitlGatekeeper::new(config)?;
        let ask_user_tool = WrappedAskUserQuestionTool::new();

        Ok(Self {
            gatekeeper,
            ask_user_tool,
        })
    }

    /// 在工具调用前进行 HITL 检查
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // 在主 Agent 的 tool 调用前
    /// let hitl = HitlIntegration::new()?;
    ///
    /// let request = ToolCallRequest {
    ///     tool_name: "delete_file".to_string(),
    ///     args: json!({ "file_path": "/tmp/file.txt" }),
    ///     context: build_context(),
    /// };
    ///
    /// match hitl.evaluate_and_confirm(request).await? {
    ///     HitlResult::Approved => {
    ///         // 用户批准，执行工具
    ///         let result = tool.call(args).await?;
    ///         hitl.record_success(tool_name).await;
    ///     }
    ///     HitlResult::Rejected => {
    ///         // 用户拒绝
    ///         println!("操作已取消");
    ///     }
    /// }
    /// ```
    pub async fn evaluate_and_confirm(
        &self,
        request: ToolCallRequest,
    ) -> Result<HitlResult, HitlIntegrationError> {
        // 1. 使用 Gatekeeper 评估
        let decision = self.gatekeeper
            .evaluate_tool_call(request.clone())
            .await
            .map_err(|e| HitlIntegrationError::GatekeeperError(e.to_string()))?;

        // 2. 根据决策处理
        match decision {
            HitlDecision::ExecuteDirectly { reason } => {
                println!(
                    "{} {}",
                    "✓".green(),
                    format!("自动批准({}): {}", request.tool_name, reason).dimmed()
                );
                Ok(HitlResult::Approved)
            }

            HitlDecision::RequireConfirmation { reason, warning_level } => {
                self.request_confirmation(&reason, &warning_level).await
            }

            HitlDecision::RequireChoice { question, options, default } => {
                self.request_choice(&question, &options, &default).await
            }

            HitlDecision::Reject { reason, suggestion } => {
                self.handle_rejection(&reason, suggestion.as_deref()).await
            }
        }
    }

    /// 请求用户确认
    async fn request_confirmation(
        &self,
        reason: &str,
        warning_level: &WarningLevel,
    ) -> Result<HitlResult, HitlIntegrationError> {
        let (icon, _color) = match warning_level {
            WarningLevel::Info => ("ℹ️", "bright_blue"),
            WarningLevel::Low => ("⚠️", "bright_yellow"),
            WarningLevel::Medium => ("⚠️", "yellow"),
            WarningLevel::High => ("🚨", "red"),
            WarningLevel::Critical => ("🔴", "bright_red"),
        };

        println!();
        println!("{} {}", icon, reason.bright_white());

        // 使用 AskUserQuestion 工具
        let args = crate::tools::ask_user_question::AskUserQuestionArgs {
            questions: vec![crate::tools::ask_user_question::Question {
                question: format!("确认执行此操作？"),
                header: "确认".to_string(),
                options: vec![
                    QuestionOption {
                        label: "确认".to_string(),
                        description: "继续执行操作".to_string(),
                    },
                    QuestionOption {
                        label: "取消".to_string(),
                        description: "取消此操作".to_string(),
                    },
                ],
                multi_select: false,
            }],
        };

        match self.ask_user_tool.call(args).await {
            Ok(output) => {
                if let Some(answer) = output.answers.get("确认") {
                    if answer.as_str() == Some("确认") || answer.as_str() == Some("是") {
                        return Ok(HitlResult::Approved);
                    }
                }
                Ok(HitlResult::Rejected)
            }
            Err(_) => Ok(HitlResult::Rejected),
        }
    }

    /// 请求用户选择
    async fn request_choice(
        &self,
        question: &str,
        options: &[crate::agent::hitl_gatekeeper::UserChoice],
        _default: &str,
    ) -> Result<HitlResult, HitlIntegrationError> {
        println!();
        println!("{}", question.bright_white());
        println!();

        // 将选项转换为 AskUserQuestion 格式
        let ask_options = options.iter().map(|opt| {
            QuestionOption {
                label: opt.label.clone(),
                description: opt.description.clone(),
            }
        }).collect();

        let args = crate::tools::ask_user_question::AskUserQuestionArgs {
            questions: vec![crate::tools::ask_user_question::Question {
                question: "请选择:".to_string(),
                header: "选择".to_string(),
                options: ask_options,
                multi_select: false,
            }],
        };

        match self.ask_user_tool.call(args).await {
            Ok(output) => {
                if let Some(answer) = output.answers.get("选择") {
                    if !answer.is_null() {
                        return Ok(HitlResult::Approved);
                    }
                }
                Ok(HitlResult::Rejected)
            }
            Err(_) => Ok(HitlResult::Rejected),
        }
    }

    /// 处理拒绝
    async fn handle_rejection(
        &self,
        reason: &str,
        suggestion: Option<&str>,
    ) -> Result<HitlResult, HitlIntegrationError> {
        println!();
        println!("{}", "❌ 操作被拒绝".bright_red());
        println!("{}", reason.bright_white());

        if let Some(suggestion) = suggestion {
            println!();
            println!("{}", "💡 建议:".bright_cyan());
            println!("  {}", suggestion);
        }

        println!();
        Ok(HitlResult::Rejected)
    }

    /// 记录操作成功
    pub async fn record_success(&self, operation: String) {
        self.gatekeeper.record_success(operation).await;
    }

    /// 记录用户拒绝
    pub async fn record_rejection(&self) {
        self.gatekeeper.record_rejection().await;
    }

    /// 获取当前信任分数
    pub async fn trust_score(&self) -> f32 {
        self.gatekeeper.trust_score().await
    }
}

/// HITL 结果
#[derive(Debug, Clone, PartialEq)]
pub enum HitlResult {
    /// 用户批准，继续执行
    Approved,

    /// 用户拒绝，取消操作
    Rejected,
}

/// HITL 集成错误
#[derive(Debug, thiserror::Error)]
pub enum HitlIntegrationError {
    #[error("Gatekeeper 错误: {0}")]
    GatekeeperError(String),

    #[error("用户交互错误: {0}")]
    UserInteractionError(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 构建操作上下文
pub fn build_operation_context(
    recent_operations: Vec<String>,
    current_task: Option<String>,
    has_git: bool,
    git_branch: Option<String>,
) -> OperationContext {
    OperationContext {
        recent_operations,
        current_task,
        has_git,
        git_branch,
    }
}

/// 可见性更高的 HITL 包装工具
/// 
/// 包装任何 rig::Tool，在执行前进行 HITL 评估和确认。
/// 如果 hitl 为 None，则直接执行。
pub struct MaybeHitlTool<T: Tool> {
    pub inner: T,
    pub hitl: Option<Arc<HitlIntegration>>,
}

impl<T: Tool> MaybeHitlTool<T> {
    pub fn new(inner: T, hitl: Option<Arc<HitlIntegration>>) -> Self {
        Self { inner, hitl }
    }
}

impl<T: Tool + Send + Sync> Tool for MaybeHitlTool<T> 
where 
    T::Args: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    T::Output: Serialize + Send + Sync,
    T::Error: From<crate::tools::FileToolError> + Send + Sync,
{
    const NAME: &'static str = T::NAME;

    type Error = T::Error;
    type Args = T::Args;
    type Output = T::Output;

    async fn definition(&self, prompt: String) -> rig::completion::ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let hitl = match &self.hitl {
            Some(h) => h,
            None => return self.inner.call(args).await,
        };

        // 1. 构建工具调用请求
        let tool_name = T::NAME.to_string();
        let args_json = serde_json::to_value(&args).unwrap_or(serde_json::Value::Null);

        // 获取当前任务上下文 (暂时使用默认值，后续可以从全局状态获取)
        let context = OperationContext {
            recent_operations: Vec::new(),
            current_task: None,
            has_git: std::path::Path::new(".git").exists(),
            git_branch: None,
        };

        let request = ToolCallRequest {
            tool_name: tool_name.clone(),
            args: args_json,
            context,
        };

        // 2. HITL 评估
        match hitl.evaluate_and_confirm(request).await {
            Ok(HitlResult::Approved) => {
                let result = self.inner.call(args).await;
                if result.is_ok() {
                    hitl.record_success(tool_name).await;
                }
                result
            }
            Ok(HitlResult::Rejected) => {
                println!("{} {} 操作已被用户取消", "🚫".red(), T::NAME);
                // 使用内部方法创建取消错误。如果工具支持，则返回具体的取消错误。
                Err(self.create_cancellation_error())
            }
            Err(e) => {
                println!("{} HITL 系统错误: {}", "❌".red(), e);
                self.inner.call(args).await
            }
        }
    }
}

impl<T: Tool> MaybeHitlTool<T> 
where
    T::Error: From<crate::tools::FileToolError> + Send + Sync,
{
    fn create_cancellation_error(&self) -> T::Error {
        crate::tools::FileToolError::Cancelled.into()
    }
}
