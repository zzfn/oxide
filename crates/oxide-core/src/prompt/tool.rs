//! 工具定义
//!
//! 定义 Agent 可用的工具及其 JSON Schema。

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数 JSON Schema
    pub input_schema: JsonValue,
}

impl ToolDefinition {
    /// 创建新的工具定义
    pub fn new(name: impl Into<String>, description: impl Into<String>, schema: JsonValue) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: schema,
        }
    }

    /// 生成提示词中的工具段落
    pub fn to_prompt_section(&self) -> String {
        format!(
            "## {}\n\n{}\n\n```json\n{}\n```",
            self.name,
            self.description,
            serde_json::to_string_pretty(&self.input_schema).unwrap_or_default()
        )
    }

    /// 转换为 API 格式
    pub fn to_api_format(&self) -> JsonValue {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "input_schema": self.input_schema
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_new() {
        let tool = ToolDefinition::new(
            "TestTool",
            "A test tool for testing",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
        );

        assert_eq!(tool.name, "TestTool");
        assert_eq!(tool.description, "A test tool for testing");
    }

    #[test]
    fn test_to_prompt_section() {
        let tool = ToolDefinition::new(
            "Read",
            "Reads a file from disk",
            serde_json::json!({"type": "object"}),
        );

        let section = tool.to_prompt_section();
        assert!(section.contains("## Read"));
        assert!(section.contains("Reads a file from disk"));
        assert!(section.contains("```json"));
    }

    #[test]
    fn test_to_api_format() {
        let tool = ToolDefinition::new("Test", "Test tool", serde_json::json!({}));
        let api = tool.to_api_format();

        assert_eq!(api["name"], "Test");
        assert_eq!(api["description"], "Test tool");
    }
}
