# Oxide Provider

LLM 提供商适配层，基于 [rig-core](https://github.com/0xPlaygrounds/rig) 库，支持 20+ LLM 提供商。

## 功能特性

- ✅ 基于 rig-core 的成熟实现
- ✅ 支持 Anthropic Claude API
- ✅ 支持自定义 API 端点
- ✅ 流式响应支持
- ✅ 工具调用支持（通过 rig Tool trait）
- ✅ 可扩展支持更多 LLM 提供商

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
export OXIDE_MODEL=claude-sonnet-4-20250514
```

默认值：`claude-sonnet-4-20250514`

## 使用示例

### 基础用法

```rust
use oxide_core::types::{Message, Role};
use oxide_provider::{RigAnthropicProvider, LLMProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 Provider
    let provider = RigAnthropicProvider::new(
        "your_api_key".to_string(),
        Some("claude-sonnet-4-20250514".to_string())
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

### 自定义端点

```rust
let provider = RigAnthropicProvider::with_base_url(
    api_key,
    "https://custom-api.com".to_string(),
    None
);
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

### 使用 rig-core 工具

```rust
use oxide_provider::{Tool, ToolDefinition, ToolSet};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct MyToolArgs {
    input: String,
}

#[derive(Serialize, Deserialize)]
struct MyTool;

impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    type Error = anyhow::Error;
    type Args = MyToolArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "my_tool".to_string(),
            description: "My custom tool".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                },
                "required": ["input"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(format!("Processed: {}", args.input))
    }
}
```

## 运行测试示例

```bash
# 设置环境变量
export OXIDE_AUTH_TOKEN=your_api_key
export OXIDE_BASE_URL=https://api.anthropic.com  # 可选

# 运行测试
cargo run --example test_api --package oxide-provider
```

## 依赖项

- `rig-core` - LLM 框架
- `tokio` - 异步运行时
- `serde` / `serde_json` - 序列化
- `anyhow` - 错误处理

## 开发状态

✅ **已完成** - 基于 rig-core 的 LLM 集成

- [x] RigAnthropicProvider 实现
- [x] 自定义端点支持
- [x] 流式响应支持
- [x] rig Tool trait 导出
- [ ] 多 LLM 提供商支持（OpenAI, Gemini 等）
