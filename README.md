# Oxide 🤖

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **Oxide** 是一个基于 Rust 构建的、高性能、极简且强大的 AI 驱动编程助手。

## 🎯 愿景

Oxide 的目标是提供一个深度集成的本地编程协作环境。不同于笨重的闭源产品，Oxide 专注于：

- **速度**：利用 Rust 的并发能力，实现毫秒级的工具调用和响应。
- **可控**：透明的工具执行，完善的 HITL (Human-In-The-Loop) 机制。
- **扩展性**：基于 `rig-core` 框架，轻松接入各种 LLM 和自定义工具。

## 🚀 重新启航

目前项目正在从零开始重构 (Rebuilding from scratch)，以实现更优雅的架构和更强的功能。

### 快速开始

1. **克隆并编译**

   ```bash
   git clone https://github.com/zzfn/oxide.git
   cd oxide
   cargo build
   ```

2. **配置环境**
   创建 `.env` 文件：

   ```env
   ANTHROPIC_API_KEY=your_key_here
   ```

3. **运行**
   ```bash
   cargo run
   ```

## 🏗️ 架构规划

- [ ] **Core**: 强健的配置、依赖注入与错误处理。
- [ ] **Provider**: 适配多种主流 LLM 提供商。
- [ ] **Memory**: 支持向量数据库与持久化会话。
- [ ] **Tools**: 针对文件系统、终端和代码搜索优化的工具集。
- [ ] **UI/CLI**: 基于 Reedline 的沉浸式交互体验。

## 🛠️ 技术栈

- **Language**: [Rust](https://www.rust-lang.org/)
- **AI Framework**: [rig-core](https://github.com/0xPlayground/rig)
- **Runtime**: [Tokio](https://tokio.rs/)
- **CLI**: [Reedline](https://github.com/nushell/reedline)

---

<div align="center">

**保持极简，追求极致。**

Made with ❤️ by [zzfn](https://github.com/zzfn)

</div>
