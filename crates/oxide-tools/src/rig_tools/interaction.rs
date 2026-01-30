//! AskUserQuestion 工具的 rig 适配

use crate::interaction::ask::{AskUserQuestionArgs, AskUserQuestionOutput, InteractionHandler};
use crate::rig_tools::errors::RigToolError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// rig AskUserQuestion 工具
#[derive(Clone)]
pub struct RigAskUserQuestionTool {
    handler: Arc<Mutex<Option<Arc<dyn InteractionHandler>>>>,
}

impl RigAskUserQuestionTool {
    pub fn new() -> Self {
        Self {
            handler: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_handler(&self, handler: Arc<dyn InteractionHandler>) {
        let mut h = self.handler.lock().await;
        *h = Some(handler);
    }
}

impl Default for RigAskUserQuestionTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for RigAskUserQuestionTool {
    const NAME: &'static str = "AskUserQuestion";

    type Error = RigToolError;
    type Args = AskUserQuestionArgs;
    type Output = AskUserQuestionOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
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

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let handler = self.handler.lock().await;
        let Some(ref h) = *handler else {
            return Err(RigToolError::ExecutionError("交互处理器未设置".to_string()));
        };

        h.ask_question(&args).await.map_err(|e| RigToolError::ExecutionError(e.to_string()))
    }
}
