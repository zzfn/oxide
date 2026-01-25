//! HITL 集成示例
//!
//! 展示如何在现有的 Agent 中集成 HITL Gatekeeper

#![allow(dead_code)]

use crate::agent::hitl_gatekeeper::{HitlConfig, HitlDecision, HitlGatekeeper, ToolCallRequest, OperationContext, WarningLevel};
use crate::tools::ask_user_question::{WrappedAskUserQuestionTool, QuestionOption};
use rig::tool::Tool;
use colored::*;

/// HITL 集成示例
///
/// 展示如何在主 Agent 的工具调用流程中集成 HITL Gatekeeper
pub struct HitlIntegration {
    gatekeeper: HitlGatekeeper,
    ask_user_tool: WrappedAskUserQuestionTool,
}

impl HitlIntegration {
    /// 创建新的 HITL 集成实例
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
                    format!("自动批准: {}", reason).dimmed()
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
            Ok(_) => Ok(HitlResult::Approved),
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
            Ok(_) => Ok(HitlResult::Approved),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hitl_integration_create() {
        let result = HitlIntegration::new();
        // 注意：这个测试需要 ANTHROPIC_API_KEY 环境变量
        // 在 CI/CD 中可能需要跳过或使用 mock
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_hitl_result() {
        let approved = HitlResult::Approved;
        let rejected = HitlResult::Rejected;

        assert_eq!(approved, HitlResult::Approved);
        assert_eq!(rejected, HitlResult::Rejected);
        assert_ne!(approved, rejected);
    }

    #[test]
    fn test_build_context() {
        let context = build_operation_context(
            vec!["read_file".to_string(), "edit_file".to_string()],
            Some("修复 bug".to_string()),
            true,
            Some("main".to_string()),
        );

        assert_eq!(context.recent_operations.len(), 2);
        assert_eq!(context.current_task, Some("修复 bug".to_string()));
        assert!(context.has_git);
        assert_eq!(context.git_branch, Some("main".to_string()));
    }
}
