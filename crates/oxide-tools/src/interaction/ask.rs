//! AskUserQuestion 工具的数据结构和 trait 定义

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 问题选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: Option<String>,
    #[serde(default)]
    pub recommended: bool,
}

/// 问题类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuestionType {
    Single,
    Multiple,
}

/// AskUserQuestion 工具的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserQuestionArgs {
    pub question_type: QuestionType,
    pub question: String,
    pub options: Vec<QuestionOption>,
    #[serde(default)]
    pub allow_custom: bool,
}

/// AskUserQuestion 工具的输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserQuestionOutput {
    pub answers: Vec<String>,
    #[serde(default)]
    pub is_custom: bool,
}

/// 交互处理器 trait
#[async_trait]
pub trait InteractionHandler: Send + Sync {
    async fn ask_question(&self, args: &AskUserQuestionArgs) -> Result<AskUserQuestionOutput>;
}
