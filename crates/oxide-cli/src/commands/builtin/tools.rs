//! Tools 命令 - 列出所有可用工具

use async_trait::async_trait;

use crate::app::SharedAppState;
use crate::commands::{Command, CommandResult};

/// Tools 命令
pub struct ToolsCommand;

#[async_trait]
impl Command for ToolsCommand {
    fn name(&self) -> &str {
        "tools"
    }

    fn description(&self) -> &str {
        "列出所有可用的工具"
    }

    fn usage(&self) -> &str {
        "/tools"
    }

    async fn execute(&self, _args: &str, state: SharedAppState) -> anyhow::Result<CommandResult> {
        let state = state.read().await;

        let Some(ref tool_registry) = state.tool_registry else {
            return Ok(CommandResult::Message("工具注册表未初始化".to_string()));
        };

        let schemas = tool_registry.schemas();

        let mut output = String::from("## 可用工具\n\n");

        for schema in schemas {
            output.push_str(&format!("### {}\n", schema.name));
            output.push_str(&format!("{}\n\n", schema.description));
            output.push_str(&format!("**参数**: \n```json\n{}\n```\n\n",
                serde_json::to_string_pretty(&schema.parameters).unwrap_or_default()));
        }

        Ok(CommandResult::Message(output))
    }
}
