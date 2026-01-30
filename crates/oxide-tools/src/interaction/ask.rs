//! AskUserQuestion 工具 - 询问用户问题

use crate::registry::{Tool, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 问题选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    /// 选项标签
    pub label: String,
    /// 选项描述
    pub description: Option<String>,
    /// 是否推荐
    #[serde(default)]
    pub recommended: bool,
}

/// 问题类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionType {
    /// 单选
    Single,
    /// 多选
    Multiple,
}

/// AskUserQuestion 工具参数
#[derive(Debug, Deserialize)]
pub struct AskUserQuestionArgs {
    /// 问题文本
    pub question: String,
    /// 问题类型（单选/多选）
    #[serde(rename = "type")]
    pub question_type: QuestionType,
    /// 选项列表
    pub options: Vec<QuestionOption>,
    /// 是否允许自定义输入
    #[serde(default)]
    pub allow_custom: bool,
}

/// AskUserQuestion 工具输出
#[derive(Debug, Serialize)]
pub struct AskUserQuestionOutput {
    /// 用户选择的答案
    pub answers: Vec<String>,
    /// 是否为自定义输入
    pub is_custom: bool,
}

/// 用户交互处理器 trait
#[async_trait]
pub trait InteractionHandler: Send + Sync {
    /// 询问用户问题
    async fn ask_question(&self, args: &AskUserQuestionArgs) -> anyhow::Result<AskUserQuestionOutput>;
}

/// AskUserQuestion 工具
pub struct AskUserQuestionTool {
    handler: Arc<Mutex<Option<Arc<dyn InteractionHandler>>>>,
}

impl AskUserQuestionTool {
    /// 创建新的工具实例
    pub fn new() -> Self {
        Self {
            handler: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置交互处理器
    pub async fn set_handler(&self, handler: Arc<dyn InteractionHandler>) {
        let mut h = self.handler.lock().await;
        *h = Some(handler);
    }
}

impl Default for AskUserQuestionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AskUserQuestionTool {
    fn name(&self) -> &str {
        "AskUserQuestion"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: "询问用户问题，支持单选、多选和自定义输入".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "要询问的问题"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["single", "multiple"],
                        "description": "问题类型：single（单选）或 multiple（多选）"
                    },
                    "options": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "label": {
                                    "type": "string",
                                    "description": "选项标签"
                                },
                                "description": {
                                    "type": "string",
                                    "description": "选项描述（可选）"
                                },
                                "recommended": {
                                    "type": "boolean",
                                    "description": "是否为推荐选项"
                                }
                            },
                            "required": ["label"]
                        },
                        "description": "选项列表"
                    },
                    "allow_custom": {
                        "type": "boolean",
                        "description": "是否允许用户输入自定义答案"
                    }
                },
                "required": ["question", "type", "options"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value) -> anyhow::Result<ToolResult> {
        let args: AskUserQuestionArgs = serde_json::from_value(input)?;

        let handler = self.handler.lock().await;
        let Some(ref h) = *handler else {
            return Ok(ToolResult::error("交互处理器未设置"));
        };

        let output = h.ask_question(&args).await?;
        let json_output = serde_json::to_string_pretty(&output)?;
        Ok(ToolResult::success(json_output))
    }
}
