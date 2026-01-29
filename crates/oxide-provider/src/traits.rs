//! LLM Provider trait 定义

use async_trait::async_trait;
use oxide_core::types::{ContentBlock, Message};

/// LLM Provider trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// 发送消息并获取响应
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<Message>;

    /// 流式发送消息
    async fn complete_stream(
        &self,
        messages: &[Message],
        callback: Box<dyn Fn(ContentBlock) + Send>,
    ) -> anyhow::Result<Message>;

    /// 获取 provider 名称
    fn name(&self) -> &str;
}
