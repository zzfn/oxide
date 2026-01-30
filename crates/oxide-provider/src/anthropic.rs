//! Anthropic API 客户端

use async_trait::async_trait;
use futures::StreamExt;
use oxide_core::types::{ContentBlock, Message, Role};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::traits::LLMProvider;

const DEFAULT_API_BASE: &str = "https://api.anthropic.com";
const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";
const API_VERSION: &str = "2023-06-01";

/// Anthropic API 请求
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<ApiMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    stream: bool,
}

/// API 消息格式
#[derive(Debug, Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: Vec<ApiContentBlock>,
}

/// API 内容块
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiContentBlock {
    Text { text: String },
    Image { source: ApiImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, #[serde(skip_serializing_if = "Option::is_none")] is_error: Option<bool> },
}

/// API 图片来源
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ApiImageSource {
    Base64 { media_type: String, data: String },
    Url { url: String },
}

/// API 响应
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ApiContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: Usage,
}

/// Token 使用统计
#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// 流式响应事件
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StreamEvent {
    MessageStart { message: MessageStart },
    ContentBlockStart { index: usize, content_block: ApiContentBlock },
    ContentBlockDelta { index: usize, delta: Delta },
    ContentBlockStop { index: usize },
    MessageDelta { delta: MessageDelta, usage: Usage },
    MessageStop,
    Ping,
    Error { error: ApiError },
}

#[derive(Debug, Deserialize)]
struct MessageStart {
    id: String,
    role: String,
    model: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Delta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
struct MessageDelta {
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Anthropic Provider
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
    temperature: Option<f32>,
}

impl AnthropicProvider {
    /// 创建新的 Provider
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: DEFAULT_API_BASE.to_string(),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            max_tokens: 8192,
            temperature: None,
        }
    }

    /// 设置自定义 Base URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// 设置 max_tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// 设置 temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// 转换内部消息格式到 API 格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<ApiMessage> {
        messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| ApiMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "system".to_string(),
                },
                content: m.content.iter().map(|c| self.convert_content_block(c)).collect(),
            })
            .collect()
    }

    /// 转换内容块
    fn convert_content_block(&self, block: &ContentBlock) -> ApiContentBlock {
        match block {
            ContentBlock::Text { text } => ApiContentBlock::Text { text: text.clone() },
            ContentBlock::Image { source } => ApiContentBlock::Image {
                source: match source {
                    oxide_core::types::ImageSource::Base64 { media_type, data } => {
                        ApiImageSource::Base64 {
                            media_type: media_type.clone(),
                            data: data.clone(),
                        }
                    }
                    oxide_core::types::ImageSource::Url { url } => {
                        ApiImageSource::Url { url: url.clone() }
                    }
                },
            },
            ContentBlock::ToolUse { id, name, input } => ApiContentBlock::ToolUse {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
            },
            ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                ApiContentBlock::ToolResult {
                    tool_use_id: tool_use_id.clone(),
                    content: content.clone(),
                    is_error: if *is_error { Some(true) } else { None },
                }
            }
        }
    }

    /// 转换 API 响应到内部格式
    fn convert_response(&self, response: AnthropicResponse) -> Message {
        Message {
            id: uuid::Uuid::new_v4(),
            role: Role::Assistant,
            content: response
                .content
                .into_iter()
                .map(|c| self.convert_api_content_block(c))
                .collect(),
            created_at: chrono::Utc::now(),
        }
    }

    /// 转换 API 内容块到内部格式
    fn convert_api_content_block(&self, block: ApiContentBlock) -> ContentBlock {
        match block {
            ApiContentBlock::Text { text } => ContentBlock::Text { text },
            ApiContentBlock::Image { source } => ContentBlock::Image {
                source: match source {
                    ApiImageSource::Base64 { media_type, data } => {
                        oxide_core::types::ImageSource::Base64 { media_type, data }
                    }
                    ApiImageSource::Url { url } => oxide_core::types::ImageSource::Url { url },
                },
            },
            ApiContentBlock::ToolUse { id, name, input } => {
                ContentBlock::ToolUse { id, name, input }
            }
            ApiContentBlock::ToolResult { tool_use_id, content, is_error } => {
                ContentBlock::ToolResult {
                    tool_use_id,
                    content,
                    is_error: is_error.unwrap_or(false),
                }
            }
        }
    }

    /// 提取 system 消息
    fn extract_system(&self, messages: &[Message]) -> Option<String> {
        messages
            .iter()
            .find(|m| m.role == Role::System)
            .and_then(|m| {
                m.content.iter().find_map(|c| {
                    if let ContentBlock::Text { text } = c {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
            })
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<Message> {
        let api_messages = self.convert_messages(messages);
        let system = self.extract_system(messages);

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: api_messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            system,
            tools: None,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("API 请求失败 ({}): {}", status, error_text);
        }

        let api_response: AnthropicResponse = response.json().await?;
        Ok(self.convert_response(api_response))
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        callback: Box<dyn Fn(ContentBlock) + Send>,
    ) -> anyhow::Result<Message> {
        let api_messages = self.convert_messages(messages);
        let system = self.extract_system(messages);

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: api_messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            system,
            tools: None,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("API 请求失败 ({}): {}", status, error_text);
        }

        let mut stream = response.bytes_stream();
        let mut accumulated_content: Vec<ContentBlock> = Vec::new();
        let mut current_text = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }

                    if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                        match event {
                            StreamEvent::ContentBlockStart { content_block, .. } => {
                                if !current_text.is_empty() {
                                    accumulated_content.push(ContentBlock::Text {
                                        text: current_text.clone(),
                                    });
                                    current_text.clear();
                                }
                                match content_block {
                                    ApiContentBlock::ToolUse { id, name, input } => {
                                        accumulated_content.push(ContentBlock::ToolUse {
                                            id,
                                            name,
                                            input,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            StreamEvent::ContentBlockDelta { delta, .. } => {
                                if let Delta::TextDelta { text } = delta {
                                    current_text.push_str(&text);
                                    callback(ContentBlock::Text { text });
                                }
                            }
                            StreamEvent::Error { error } => {
                                anyhow::bail!("流式响应错误: {}", error.message);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if !current_text.is_empty() {
            accumulated_content.push(ContentBlock::Text { text: current_text });
        }

        Ok(Message {
            id: uuid::Uuid::new_v4(),
            role: Role::Assistant,
            content: accumulated_content,
            created_at: chrono::Utc::now(),
        })
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}
