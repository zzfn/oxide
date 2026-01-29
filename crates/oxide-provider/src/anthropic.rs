//! Anthropic API 客户端（占位）

use async_trait::async_trait;
use oxide_core::types::{ContentBlock, Message};

use crate::traits::LLMProvider;

/// Anthropic Provider
pub struct AnthropicProvider {
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn complete(&self, _messages: &[Message]) -> anyhow::Result<Message> {
        // TODO: 实现 Anthropic API 调用
        todo!("实现 Anthropic API 调用")
    }

    async fn complete_stream(
        &self,
        _messages: &[Message],
        _callback: Box<dyn Fn(ContentBlock) + Send>,
    ) -> anyhow::Result<Message> {
        // TODO: 实现流式 API 调用
        todo!("实现流式 API 调用")
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}
