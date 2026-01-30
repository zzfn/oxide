# Oxide Provider

LLM 提供商适配层，支持与 Anthropic Claude API 集成。

## 功能特性

- ✅ 完整的 Anthropic API 支持
- ✅ 流式响应（Server-Sent Events）
- ✅ 多模态内容（文本、图片）
- ✅ 工具调用协议（Tool Use）
- ✅ 自定义 API 端点
- ✅ 灵活的配置选项

## 环境变量配置

### API 认证

Provider 支持两种方式配置 API Key（按优先级排序）：

1. **OXIDE_AUTH_TOKEN** - Oxide 自定义 Token（推荐）
2. **ANTHROPIC_API_KEY** - 标准 Anthropic API Key

```bash
# 方式 1: 使用 Oxide Token
export OXIDE_AUTH_TOKEN=your_api_key

# 方式 2: 使用标准 Anthropic Key
export ANTHROPIC_API_KEY=your_api_key
```

### 自定义 API 端点

如果需要使用自定义 API 端点（如代理或本地服务）：

```bash
export OXIDE_BASE_URL=https://your-custom-endpoint.com
```

默认值：`https://api.anthropic.com`

### 模型选择

```bash
export OXIDE_MODEL=claude-sonnet-4-5-20250929
```

默认值：`claude-sonnet-4-5-20250929`

## 使用示例

### 基础用法

```rust
use oxide_core::types::{Message, Role};
use oxide_provider::{AnthropicProvider, LLMProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 Provider
    let provider = AnthropicProvider::new(
        "your_api_key".to_string(),
        Some("claude-sonnet-4-5-20250929".to_string())
    );

    // 发送消息
    let messages = vec![
        Message::text(Role::User, "Hello!")
    ];

    let response = provider.complete(&messages).await?;
    println!("{:?}", response);

    Ok(())
}
```

### 自定义配置

```rust
let provider = AnthropicProvider::new(api_key, None)
    .with_base_url("https://custom-api.com".to_string())
    .with_max_tokens(4096)
    .with_temperature(0.7);
```

### 流式响应

```rust
use oxide_core::types::ContentBlock;

provider.complete_stream(
    &messages,
    Box::new(|block| {
        if let ContentBlock::Text { text } = block {
            print!("{}", text);
        }
    })
).await?;
```

## 运行测试示例

```bash
# 设置环境变量
export OXIDE_AUTH_TOKEN=your_api_key
export OXIDE_BASE_URL=https://api.anthropic.com  # 可选

# 运行测试
cargo run --example test_api --package oxide-provider
```

## API 兼容性

本实现遵循 Anthropic Messages API 规范：

- API Version: `2023-06-01`
- 支持的内容类型：
  - `text` - 文本内容
  - `image` - 图片（Base64 或 URL）
  - `tool_use` - 工具调用
  - `tool_result` - 工具结果

## 错误处理

Provider 使用 `anyhow::Result` 进行错误处理，所有 API 错误都会被转换为可读的错误消息。

```rust
match provider.complete(&messages).await {
    Ok(response) => {
        // 处理响应
    }
    Err(e) => {
        eprintln!("API 错误: {}", e);
    }
}
```

## 依赖项

- `reqwest` - HTTP 客户端
- `tokio` - 异步运行时
- `serde` / `serde_json` - 序列化
- `futures` - 流式处理

## 开发状态

✅ **Phase 1 完成** - LLM 集成已完成

- [x] Provider trait 定义
- [x] Anthropic API 客户端
- [x] 流式响应支持
- [x] 消息格式转换
- [x] 工具调用协议
- [x] 多模态内容支持
