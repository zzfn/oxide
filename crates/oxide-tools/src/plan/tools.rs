//! 计划模式工具

use oxide_core::error::OxideError;
use oxide_core::session::{AllowedPrompt, Plan};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::manager::PlanManager;

/// EnterPlanMode 工具参数
#[derive(Debug, Deserialize)]
pub struct EnterPlanModeArgs {}

/// EnterPlanMode 工具输出
#[derive(Debug, Serialize)]
pub struct EnterPlanModeOutput {
    pub plan_id: String,
    pub message: String,
}

/// EnterPlanMode 工具
#[derive(Clone)]
pub struct RigEnterPlanModeTool {
    plan_manager: PlanManager,
}

impl RigEnterPlanModeTool {
    pub fn new(plan_manager: PlanManager) -> Self {
        Self { plan_manager }
    }
}

impl Tool for RigEnterPlanModeTool {
    const NAME: &'static str = "EnterPlanMode";

    type Error = OxideError;
    type Args = EnterPlanModeArgs;
    type Output = EnterPlanModeOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "进入计划模式。在此模式下，代理将探索代码库并设计实现方案，而不是直接执行代码修改。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let plan_id = self.plan_manager.enter_plan_mode(None).await;

        Ok(EnterPlanModeOutput {
            plan_id: plan_id.to_string(),
            message: format!(
                "已进入计划模式 (Plan ID: {})。现在可以探索代码库并设计实现方案。完成后使用 ExitPlanMode 工具保存计划。",
                plan_id
            ),
        })
    }
}

/// 权限提示参数
#[derive(Debug, Deserialize)]
pub struct AllowedPromptArg {
    pub tool: String,
    pub prompt: String,
}

/// ExitPlanMode 工具参数
#[derive(Debug, Deserialize)]
pub struct ExitPlanModeArgs {
    /// 计划内容（Markdown 格式）
    pub plan_content: String,
    /// 计划标题（可选）
    pub plan_title: Option<String>,
    /// 实现计划所需的权限
    #[serde(default, rename = "allowedPrompts")]
    pub allowed_prompts: Option<Vec<AllowedPromptArg>>,
}

/// ExitPlanMode 工具输出
#[derive(Debug, Serialize)]
pub struct ExitPlanModeOutput {
    pub plan_id: String,
    pub plan_path: String,
    pub message: String,
}

/// ExitPlanMode 工具
#[derive(Clone)]
pub struct RigExitPlanModeTool {
    plan_manager: PlanManager,
}

impl RigExitPlanModeTool {
    pub fn new(plan_manager: PlanManager) -> Self {
        Self { plan_manager }
    }
}

impl Tool for RigExitPlanModeTool {
    const NAME: &'static str = "ExitPlanMode";

    type Error = OxideError;
    type Args = ExitPlanModeArgs;
    type Output = ExitPlanModeOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "退出计划模式并保存计划。计划将保存到 ~/.oxide/plans/ 目录。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "plan_content": {
                        "type": "string",
                        "description": "计划内容（Markdown 格式），包括实现步骤、文件修改清单、架构决策等"
                    },
                    "plan_title": {
                        "type": "string",
                        "description": "计划标题（可选）"
                    },
                    "allowedPrompts": {
                        "type": "array",
                        "description": "实现计划所需的权限列表（可选）",
                        "items": {
                            "type": "object",
                            "properties": {
                                "tool": {
                                    "type": "string",
                                    "description": "工具名称（如 Bash）",
                                    "enum": ["Bash"]
                                },
                                "prompt": {
                                    "type": "string",
                                    "description": "权限描述（如 'run tests', 'install dependencies'）"
                                }
                            },
                            "required": ["tool", "prompt"]
                        }
                    }
                },
                "required": ["plan_content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let plan_id = self
            .plan_manager
            .current_plan_id()
            .await
            .unwrap_or_else(Uuid::new_v4);

        // 创建计划对象
        let title = args
            .plan_title
            .unwrap_or_else(|| format!("Plan {}", plan_id));

        let allowed_prompts = args
            .allowed_prompts
            .map(|prompts| {
                prompts
                    .into_iter()
                    .map(|p| AllowedPrompt {
                        tool: p.tool,
                        prompt: p.prompt,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let plan = Plan::new(title.clone(), args.plan_content, plan_id)
            .with_allowed_prompts(allowed_prompts);

        // 保存计划
        plan.save().map_err(|e| {
            OxideError::ToolExecution(format!("Failed to save plan: {}", e))
        })?;

        // 退出计划模式
        self.plan_manager.exit_plan_mode().await;

        let plan_path = format!("~/.oxide/plans/{}.json", plan_id);

        let mut message = format!(
            "计划已保存: {}\n\n计划标题: {}\n计划 ID: {}",
            plan_path, title, plan_id
        );

        if !plan.allowed_prompts.is_empty() {
            message.push_str("\n\n请求的权限:");
            for prompt in &plan.allowed_prompts {
                message.push_str(&format!("\n  - {} ({})", prompt.prompt, prompt.tool));
            }
        }

        Ok(ExitPlanModeOutput {
            plan_id: plan_id.to_string(),
            plan_path: plan_path.clone(),
            message,
        })
    }
}
