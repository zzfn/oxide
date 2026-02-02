# Oxide - 快速开始

## 🎉 Phase 1 已完成！

Oxide 的 LLM 集成层已经完成，现在可以与 Anthropic Claude API 进行完整交互。

## 📦 安装依赖

```bash
cd /Users/c.chen/dev/oxide
cargo build --workspace
```

## 🔑 配置 API Key

### 方式 1: 使用 OXIDE_AUTH_TOKEN（推荐）

```bash
export OXIDE_AUTH_TOKEN=your_api_key_here
```

### 方式 2: 使用标准 ANTHROPIC_API_KEY

```bash
export ANTHROPIC_API_KEY=your_api_key_here
```

### 可选：自定义 API 端点

```bash
export OXIDE_BASE_URL=https://your-custom-endpoint.com
```

## 🧪 测试 API 集成

运行测试示例验证配置：

```bash
# 确保已设置 API Key
export OXIDE_AUTH_TOKEN=your_api_key

# 运行测试
cargo run --example test_api --package oxide-provider
```

预期输出：
```
🚀 测试 Anthropic API 集成

📝 测试 1: 简单对话
✅ 响应成功:
   我是 Claude，一个由 Anthropic 开发的 AI 助手。

---

📝 测试 2: 流式响应
✅ 流式输出: 安全、高效、可靠
✅ 流式响应完成

🎉 所有测试通过！
```

## 📚 使用示例

### 基础对话

```rust
use oxide_core::types::{Message, Role};
use oxide_provider::{AnthropicProvider, LLMProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 Provider
    let provider = AnthropicProvider::new(
        std::env::var("OXIDE_AUTH_TOKEN")?,
        None // 使用默认模型
    );

    // 发送消息
    let messages = vec![
        Message::text(Role::User, "Hello, Claude!")
    ];

    let response = provider.complete(&messages).await?;

    // 打印响应
    for block in response.content {
        if let oxide_core::types::ContentBlock::Text { text } = block {
            println!("{}", text);
        }
    }

    Ok(())
}
```

### 流式响应

```rust
use oxide_core::types::ContentBlock;

provider.complete_stream(
    &messages,
    Box::new(|block| {
        if let ContentBlock::Text { text } = block {
            print!("{}", text);
            std::io::stdout().flush().unwrap();
        }
    })
).await?;
```

### 自定义配置

```rust
let provider = AnthropicProvider::new(api_key, Some("claude-opus-4-5".to_string()))
    .with_base_url("https://custom-api.com".to_string())
    .with_max_tokens(4096)
    .with_temperature(0.7);
```

## 🏗️ 项目结构

```
oxide/
├── crates/
│   ├── oxide-core/          # ✅ 核心类型和配置
│   ├── oxide-provider/      # ✅ LLM 提供商（Phase 1 完成）
│   ├── oxide-tools/         # ✅ 工具系统（已完成）
│   └── oxide-cli/           # ✅ CLI 界面（基础完成）
├── docs/
│   ├── roadmap.md           # 项目路线图
│   └── phase1-completion.md # Phase 1 完成总结
└── Cargo.toml               # Workspace 配置
```

## ✅ 已完成功能

### Phase 0: 基础设施 (90%)
- ✅ Workspace 结构
- ✅ 配置系统
- ✅ 错误处理
- ✅ 会话管理

### Phase 1: LLM 集成 (100%)
- ✅ Provider trait
- ✅ Anthropic API 客户端
- ✅ 流式响应
- ✅ 消息类型
- ✅ 工具调用协议
- ✅ 多模态内容

### Phase 4: CLI 界面 (85%)
- ✅ Reedline 编辑器
- ✅ 命令系统
- ✅ Markdown 渲染
- ✅ 状态栏

## 🎯 下一步计划

### Phase 2: 核心工具系统
实现以下工具：
- [ ] Read - 文件读取
- [ ] Write - 文件写入
- [ ] Edit - 文件编辑
- [ ] Glob - 文件搜索
- [ ] Grep - 内容搜索
- [ ] Bash - 命令执行
- [ ] WebFetch - 网页获取

### Phase 3: 高级功能
- [x] 代理主循环
- [x] 工具调用循环
- [ ] 子代理系统（待实现）
- [x] 任务管理

## 📖 文档

- [完整路线图](./docs/roadmap.md)
- [Phase 1 完成总结](./docs/phase1-completion.md)
- [Provider 使用文档](./crates/oxide-provider/README.md)

## 🐛 故障排除

### API Key 未设置

```
Error: 未设置 API Key 环境变量
```

**解决方案**: 设置 `OXIDE_AUTH_TOKEN` 或 `ANTHROPIC_API_KEY`

### API 请求失败

```
Error: API 请求失败 (401): Unauthorized
```

**解决方案**: 检查 API Key 是否正确

### 自定义端点连接失败

```
Error: API 请求失败 (Connection refused)
```

**解决方案**: 检查 `OXIDE_BASE_URL` 是否正确，确保端点可访问

## 💡 提示

1. **API Key 安全**: 永远不要在代码中硬编码 API Key
2. **环境变量**: 使用 `.env` 文件或 shell 配置管理环境变量
3. **自定义端点**: 适用于代理、本地测试或企业部署
4. **流式响应**: 提供更好的用户体验，实时显示生成内容

## 🤝 贡献

欢迎贡献！请查看 [roadmap.md](./docs/roadmap.md) 了解待完成的任务。

---

**更新时间**: 2026-01-30
**当前版本**: 0.1.0
**状态**: Phase 1 完成 ✅
