# Oxide 🤖

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Phase 1](https://img.shields.io/badge/Phase%201-✅%20Complete-brightgreen.svg)](docs/archive/phase1-completion.md)

> **Oxide** 是一个基于 Rust 构建的、高性能、极简且强大的 AI 驱动编程助手。

## 🎉 最新进展

✅ **Phase 1 (LLM 集成)** - 已完成，支持流式响应和自定义端点
✅ **Phase 2 (核心工具)** - 95% 完成！实现了完整的工具系统和代理循环
✅ **代理主循环** - 刚刚完成！AI 现在可以自主调用工具完成任务
🆕 **rig-core 集成** - 新增！支持 20+ LLM 提供商，保留自实现作为备选

查看完成总结：

- [Phase 1 完成总结](docs/archive/phase1-completion.md)
- [Phase 2.2 完成总结](docs/archive/phase2.2-completion.md) - 文件操作工具
- [Phase 2.3 完成总结](docs/archive/phase2.3-completion.md) - 搜索工具
- [Phase 2.4 完成总结](docs/archive/phase2.4-completion.md) - 代理主循环
- [rig-core 迁移完成](docs/archive/rig-core-migration-complete.md) - rig-core 集成

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

查看 [快速开始指南](docs/QUICKSTART.md) 了解更多。

## 🏗️ 项目结构

```
oxide/
├── crates/
│   ├── oxide-core/          # ✅ 核心类型和配置
│   ├── oxide-provider/      # ✅ LLM 提供商（支持工具调用）
│   ├── oxide-tools/         # ✅ 工具系统（Read, Write, Edit, Glob, Grep, Bash）
│   ├── oxide-agent/         # 🚧 代理系统（基础功能完成）
│   └── oxide-cli/           # ✅ CLI 界面（完整的代理循环）
├── docs/                    # 文档
│   ├── roadmap.md          # 完整路线图
│   ├── QUICKSTART.md       # 快速开始指南
│   ├── CLI_INTEGRATION.md  # CLI 集成文档
│   ├── task-system.md      # 任务系统文档
│   └── archive/            # 历史完成总结
└── Cargo.toml              # Workspace 配置
```

## 📊 开发进度

| Phase   | 功能     | 状态 | 完成度 |
| ------- | -------- | ---- | ------ |
| Phase 0 | 基础设施 | ✅   | 100%   |
| Phase 1 | LLM 集成 | ✅   | 100%   |
| Phase 2 | 核心工具 | 🚧   | 95%    |
| Phase 3 | 高级功能 | ⏳   | 0%     |
| Phase 4 | CLI 界面 | ✅   | 100%   |
| Phase 5 | Git 集成 | ⏳   | 0%     |
| Phase 6 | 扩展功能 | ⏳   | 0%     |
| Phase 7 | 优化完善 | ⏳   | 0%     |

查看 [完整路线图](docs/roadmap.md) 了解详细计划。

## 💻 使用示例

### 启动 CLI

```bash
# 设置 API Key
export OXIDE_AUTH_TOKEN=your_api_key_here

# 启动 Oxide CLI
cargo run --bin oxide

# 或者编译后运行
cargo build --release
./target/release/oxide
```

### 与 AI 对话

```
[N] > 帮我读取 src/main.rs 文件

Assistant ⚙ 执行工具: Read
  ✓ 工具 Read 执行成功

这是 main.rs 的内容...
```

### 基础对话（编程使用）

```rust
use oxide_core::types::{Message, Role};
use oxide_provider::{RigAnthropicProvider, LLMProvider};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = RigAnthropicProvider::new(
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
- **LLM Framework**: [rig-core](https://github.com/0xPlaygrounds/rig) - 支持 20+ LLM 提供商
- **CLI**: [Reedline](https://github.com/nushell/reedline)
- **Rendering**: [Termimad](https://github.com/Canop/termimad)

## 📖 文档

- [快速开始](docs/QUICKSTART.md)
- [完整路线图](docs/roadmap.md)
- [Phase 1 完成总结](docs/archive/phase1-completion.md)
- [Phase 2.2 完成总结](docs/archive/phase2.2-completion.md) - 文件操作工具
- [Phase 2.3 完成总结](docs/archive/phase2.3-completion.md) - 搜索工具
- [Phase 2.4 完成总结](docs/archive/phase2.4-completion.md) - 代理主循环
- [CLI 集成文档](docs/CLI_INTEGRATION.md)
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

**当前版本**: 0.1.0 | **最后更新**: 2026-01-30 | **状态**: Phase 2 (95%) 🚀

</div>
