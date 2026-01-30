//! 基于 rig-core 的 LLM Provider
//!
//! 使用 rig-core 库提供 LLM 集成，支持多种提供商。

use anyhow::Result;
use async_trait::async_trait;
use oxide_core::types::{ContentBlock, Message, Role};
use rig::client::CompletionClient;
use rig::providers::anthropic;
use rig::providers::anthropic::completion::CompletionModel;
pub use rig::tool::ToolSet;

use crate::LLMProvider;

/// 基于 rig-core 的 Anthropic Provider
pub struct RigAnthropicProvider {
    client: anthropic::Client,
    model: String,
}

impl RigAnthropicProvider {
    /// 创建新的 Provider
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = anthropic::Client::builder()
            .api_key(api_key)
            .build()
            .expect("Failed to create Anthropic client");

        Self {
            client,
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        }
    }

    /// 设置自定义 Base URL
    pub fn with_base_url(api_key: String, base_url: String, model: Option<String>) -> Self {
        let client = anthropic::Client::builder()
            .api_key(api_key)
            .base_url(&base_url)
            .build()
            .expect("Failed to create Anthropic client");

        Self {
            client,
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        }
    }

    /// 获取 rig client 引用（用于创建 Agent）
    pub fn client(&self) -> &anthropic::Client {
        &self.client
    }

    /// 获取模型名称
    pub fn model(&self) -> &str {
        &self.model
    }

    /// 创建 AgentBuilder（用于自定义配置）
    pub fn agent_builder(&self) -> rig::agent::AgentBuilder<CompletionModel> {
        self.client.agent(&self.model)
    }

    /// 创建带工具的 Agent
    pub fn create_agent_with_tools(
        &self,
        preamble: Option<&str>,
        tools: Vec<Box<dyn ToolDyn>>,
    ) -> rig::agent::Agent<CompletionModel> {
        let mut builder = self.client.agent(&self.model);
        if let Some(sys) = preamble {
            builder = builder.preamble(sys);
        }
        builder.tools(tools).build()
    }

    /// 创建不带工具的 Agent
    pub fn create_agent(&self, preamble: Option<&str>) -> rig::agent::Agent<CompletionModel> {
        let mut builder = self.client.agent(&self.model);
        if let Some(sys) = preamble {
            builder = builder.preamble(sys);
        }
        builder.build()
    }
}

#[async_trait]
impl LLMProvider for RigAnthropicProvider {
    async fn complete(&self, messages: &[Message]) -> Result<Message> {
        use rig::client::CompletionClient;
        use rig::completion::Prompt;

        // 转换消息格式
        let prompt = messages
            .iter()
            .filter(|m| m.role == Role::User)
            .filter_map(|m| {
                m.content.iter().find_map(|block| {
                    if let ContentBlock::Text { text } = block {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
            })
            .last()
            .unwrap_or_default();

        // 获取 system prompt
        let system = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .filter_map(|m| {
                m.content.iter().find_map(|block| {
                    if let ContentBlock::Text { text } = block {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
            })
            .last();

        // 创建 agent
        let mut agent_builder = self.client.agent(&self.model);
        if let Some(sys) = system {
            agent_builder = agent_builder.preamble(&sys);
        }
        let agent = agent_builder.build();

        // 执行 prompt
        let response = agent.prompt(&prompt).await?;

        Ok(Message {
            id: uuid::Uuid::new_v4(),
            role: Role::Assistant,
            content: vec![ContentBlock::Text { text: response }],
            created_at: chrono::Utc::now(),
        })
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        callback: Box<dyn Fn(ContentBlock) + Send>,
    ) -> Result<Message> {
        // 暂时使用非流式实现
        // TODO: 实现真正的流式响应
        let response = self.complete(messages).await?;

        // 模拟流式输出
        for block in &response.content {
            callback(block.clone());
        }

        Ok(response)
    }

    async fn complete_with_tools(
        &self,
        _messages: &[Message],
        _tools: Option<Vec<serde_json::Value>>,
    ) -> Result<Message> {
        // 工具调用通过 RigAgent 实现，这里不直接支持
        anyhow::bail!("请使用 RigAgent 进行工具调用")
    }

    async fn complete_stream_with_tools(
        &self,
        _messages: &[Message],
        _tools: Option<Vec<serde_json::Value>>,
        _callback: Box<dyn Fn(ContentBlock) + Send>,
    ) -> Result<Message> {
        // 工具调用通过 RigAgent 实现，这里不直接支持
        anyhow::bail!("请使用 RigAgent 进行工具调用")
    }

    fn name(&self) -> &str {
        "rig-anthropic"
    }
}

// Re-export rig types for convenience
pub use rig::agent::{Agent as RigAgent, AgentBuilder};
pub use rig::completion::{Prompt, ToolDefinition};
pub use rig::tool::{Tool, ToolDyn};
