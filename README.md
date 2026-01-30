# Oxide 🤖

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Phase 1](https://img.shields.io/badge/Phase%201-✅%20Complete-brightgreen.svg)](docs/phase1-completion.md)

> **Oxide** 是一个基于 Rust 构建的、高性能、极简且强大的 AI 驱动编程助手。

## 🎉 Phase 1 完成！

✅ **LLM 集成层已完成**，现在可以与 Anthropic Claude API 进行完整交互，支持流式响应和自定义端点。

查看 [Phase 1 完成总结](docs/phase1-completion.md) 了解详情。

## 🎯 愿景

Oxide 的目标是提供一个深度集成的本地编程协作环境。不同于笨重的闭源产品，Oxide 专注于：

- **速度**：利用 Rust 的并发能力，实现毫秒级的工具调用和响应。
- **可控**：透明的工具执行，完善的 HITL (Human-In-The-Loop) 机制。
- **扩展性**：模块化设计，轻松接入各种 LLM 和自定义工具。

## 🚀 快速开始

### 1. 克隆并编译

```bash
git clone https://github.com/zzfn/oxide.git
cd oxide
cargo build --workspace
```

### 2. 配置环境

```bash
# 方式 1: 使用 OXIDE_AUTH_TOKEN（推荐）
export OXIDE_AUTH_TOKEN=your_api_key_here

# 方式 2: 使用标准 ANTHROPIC_API_KEY
export ANTHROPIC_API_KEY=your_api_key_here

# 可选：自定义 API 端点
export OXIDE_BASE_URL=https://your-custom-endpoint.com
```

### 3. 测试 API 集成

```bash
cargo run --example test_api --package oxide-provider
```

查看 [快速开始指南](QUICKSTART.md) 了解更多。

## 🏗️ 项目结构

```
oxide/
├── crates/
│   ├── oxide-core/          # ✅ 核心类型和配置
│   ├── oxide-provider/      # ✅ LLM 提供商（Phase 1 完成）
│   ├── oxide-tools/         # 🚧 工具系统（待实现）
│   ├── oxide-agent/         # 🚧 代理系统（待实现）
│   └── oxide-cli/           # ✅ CLI 界面（基础完成）
├── docs/                    # 文档
│   ├── roadmap.md          # 完整路线图
│   └── phase1-completion.md # Phase 1 总结
└── Cargo.toml              # Workspace 配置
```

## 📊 开发进度

| Phase | 功能 | 状态 | 完成度 |
|-------|------|------|--------|
| Phase 0 | 基础设施 | ✅ | 90% |
| Phase 1 | LLM 集成 | ✅ | 100% |
| Phase 2 | 核心工具 | 🚧 | 20% |
| Phase 3 | 高级功能 | ⏳ | 0% |
| Phase 4 | CLI 界面 | ✅ | 85% |
| Phase 5 | Git 集成 | ⏳ | 0% |
| Phase 6 | 扩展功能 | ⏳ | 0% |
| Phase 7 | 优化完善 | ⏳ | 0% |

查看 [完整路线图](docs/roadmap.md) 了解详细计划。

## 💻 使用示例

### 基础对话

```rust
use oxide_core::types::{Message, Role};
use oxide_provider::{AnthropicProvider, LLMProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = AnthropicProvider::new(
        std::env::var("OXIDE_AUTH_TOKEN")?,
        None
    );

    let messages = vec![
        Message::text(Role::User, "Hello, Claude!")
    ];

    let response = provider.complete(&messages).await?;
    println!("{:?}", response);

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
        }
    })
).await?;
```

## 🛠️ 技术栈

- **Language**: [Rust](https://www.rust-lang.org/) 1.70+
- **Runtime**: [Tokio](https://tokio.rs/)
- **HTTP Client**: [Reqwest](https://github.com/seanmonstar/reqwest)
- **CLI**: [Reedline](https://github.com/nushell/reedline)
- **Rendering**: [Termimad](https://github.com/Canop/termimad)

## 📖 文档

- [快速开始](QUICKSTART.md)
- [完整路线图](docs/roadmap.md)
- [Phase 1 完成总结](docs/phase1-completion.md)
- [Provider 使用文档](crates/oxide-provider/README.md)

## 🤝 贡献

欢迎贡献！请遵循以下原则：

1. 保持代码简洁和可读
2. 编写测试覆盖新功能
3. 更新相关文档
4. 遵循 Rust 最佳实践

查看 [路线图](docs/roadmap.md) 了解待完成的任务。

---

<div align="center">

**保持极简，追求极致。**

Made with ❤️ by [zzfn](https://github.com/zzfn)

**当前版本**: 0.1.0 | **最后更新**: 2026-01-30 | **状态**: Phase 1 完成 ✅

</div>
