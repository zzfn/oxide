# Oxide - AI-Powered Coding Agent

[![Build Status](https://img.shields.io/github/actions/workflow/status/yourusername/oxide/build.yml?branch=main)](https://github.com/yourusername/oxide/actions)
[![Version](https://img.shields.io/crates/v/oxide)](https://crates.io/crates/oxide)
[![License](https://img.shields.io/crates/l/oxide)](https://github.com/yourusername/oxide/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

> 🤖 一个强大的 AI 驱动编程助手，让编程更简单、更高效

## ✨ 特性

https://github.com/ThreeFish-AI/analysis_claude_code
https://github.com/shareAI-lab/Kode-cli
https://github.com/shareAI-lab/learn-claude-code

Claude Code的系统架构和核心机制

- 🧠 **智能对话** - 自然语言交互，理解你的编程需求
- 🛠️ **9+ 核心工具** - 文件操作、代码搜索、Shell 执行等
- 🎯 **技能系统** - 自定义和复用编程技能（内置、全局、本地）
- 🤖 **多 Agent 支持** - Main, Explore, Plan, CodeReviewer, FrontendDeveloper
- 📊 **会话记忆** - 上下文感知的长期对话
- 🔍 **代码库扫描** - 智能解析项目结构
- ⚡ **高性能** - 基于 Rust 构建，快速响应
- 🔌 **可扩展** - 插件化架构，轻松添加新功能

## 🎯 快速开始

### 从源代码编译

```bash
# 克隆仓库
git clone https://github.com/zzfn/oxide.git
cd oxide

# 编译项目
cargo build --release

# 编译后的二进制文件位于 target/release/oxide
```

### 使用 Cargo 安装

```bash
cargo install oxide
```

## 配置

### 环境变量

创建 `.env` 文件并设置以下变量：

```env
# API 认证 Token（必需）
# 支持的环境变量（按优先级排序）:
# 1. OXIDE_AUTH_TOKEN - 推荐使用
# 2. ANTHROPIC_API_KEY - Anthropic API Key
# 3. API_KEY - 通用 API Key
OXIDE_AUTH_TOKEN=sk-your_token_here

# API 基础地址（可选）
# 默认: https://api.anthropic.com
# 支持的环境变量（按优先级排序）:
# 1. OXIDE_BASE_URL - 推荐使用
# 2. API_URL - 通用 API URL
OXIDE_BASE_URL=https://api.anthropic.com

# 模型名称（可选）
# 默认: claude-sonnet-4-20250514
# 不填写则使用服务端默认模型
MODEL_NAME=claude-sonnet-4-20250514

# 最大 Token 数（可选）
# MAX_TOKENS=4096
```

**配置说明：**

- `OXIDE_AUTH_TOKEN`: LLM 提供商的 API 密钥（必需）
- `OXIDE_BASE_URL`: API 端点 URL（可选，默认为 Anthropic）
- `MODEL_NAME`: 使用的模型名称（可选，不填写则使用服务端默认）
- `MAX_TOKENS`: 最大 token 数（可选，默认为 4096）

### 支持的模型

Oxide 支持以下 LLM 提供商：

- **Anthropic** - `claude-sonnet-4-20250514`, `claude-opus-4-20250514`
- **OpenAI** - `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`, `gpt-3.5-turbo`
- **其他 OpenAI 兼容的 API** - 通过设置正确的 `OXIDE_BASE_URL` 使用

**注意：** Provider 判断基于 `OXIDE_BASE_URL` 中是否包含 "anthropic" 字符串。使用非 Anthropic API 时，会使用 OpenAI 兼容接口。

### 配置文件

配置系统支持多层配置，按优先级从低到高：

1. **全局配置** - `~/.oxide/config.toml` 或 `~/.oxide/config.toml`
2. **项目配置** - `.oxide/config.toml`（覆盖全局配置）
3. **项目指令** - `.oxide/CONFIG.md`（系统提示词）
4. **环境变量** - 覆盖所有文件配置（最高优先级）

**全局配置位置：**

```bash
# Linux/macOS
~/.config/oxide/config.toml 或 ~/.oxide/config.toml

# Windows
%APPDATA%\oxide\config.toml
```

**配置示例：**

```toml
# 默认模型配置
[default]
base_url = "https://api.anthropic.com"
model = "claude-sonnet-4-20250514"
max_tokens = 4096
temperature = 0.7

# Agent 特定配置
[agent]
explore.model = "claude-haiku-4-20250514"
plan.model = "claude-sonnet-4-20250514"
code_reviewer.model = "claude-sonnet-4-20250514"

# 主题配置
[theme]
mode = "dark"
# custom_theme = "my-theme.toml"

# 功能开关
[features]
enable_mcp = false
enable_multimodal = false
```

**配置优先级说明：**

- 环境变量 > 项目配置 > 全局配置
- 如果没有配置文件，使用默认值
- 可以使用 `OXIDE_AUTH_TOKEN`、`OXIDE_BASE_URL` 等环境变量覆盖文件配置

## 使用方法

### 启动 CLI

```bash
# 使用 cargo run
cargo run

# 或使用编译后的二进制文件
./target/release/oxide
```

### 斜杠命令

启动后，你可以使用以下斜杠命令：

| 命令           | 说明                   |
| -------------- | ---------------------- | ------ | ---------- | -------- |
| `/help`        | 显示帮助信息           |
| `/clear`       | 清空当前对话           |
| `/config [show | edit                   | reload | validate]` | 管理配置 |
| `/history`     | 显示当前会话的历史消息 |

## 已知问题

- PAOR 工作流未接入主对话：`src/agent/workflow/orchestrator.rs` 仅有占位逻辑，目前只在 `examples/workflow_example.rs` 演示使用。
- Task/TaskOutput 工具未集成到主 Agent：`src/tools/task.rs` 标注同步执行需要完整集成，`src/tools/mod.rs` 也注明暂未集成。
- Agent 类型命名体系不一致：`AgentType` 是实例枚举（Anthropic/OpenAI），`NewAgentType` 才是 Main/Explore/Plan 等类型，CLI 中混用导致“当前 agent 类型”与实例未绑定。
  | `/sessions` | 列出所有保存的会话 |
  | `/load <id>` | 加载指定的会话 |
  | `/delete <id>` | 删除指定会话 |
  | `/agent [list|capabilities]` | 查看 Agent 类型与能力 |
  | `/tasks [list|show <id>|cancel <id>]` | 管理后台任务 |
  | `/skills [list|show <name>]` | 管理和使用技能 |
  | `/<skill-name>` | 执行指定的技能 |
  | `/exit` 或 `/quit` | 退出程序 |

**⚠️ 注意：** 部分功能标记为实验性或未完全实现：

### 对话示例

```
==================================================
Oxide CLI 0.1.0 - Anthropic Agent
==================================================
模型: claude-sonnet-4-20250514
会话: violet-sky-1234
提示: 输入 /help 查看帮助
提示: 输入 /exit 退出

你>[0] 你好！
你好！我是 Oxide 助手，有什么可以帮助你的吗？

你>[1] 帮我查看当前目录的文件结构
[工具] scan_codebase
...
```

## Markdown 渲染

Oxide 支持实时渲染 AI 回复中的 Markdown 格式，提供更好的阅读体验：

### 支持的 Markdown 元素

- **标题** - `# H1`, `## H2`, `### H3` 等（青色显示）
- **粗体** - `**粗体文本**`（白色高亮）
- **斜体** - `*斜体文本*`（黄色显示）
- **行内代码** - `` `代码` ``（绿色显示）
- **代码块** - 三反引号包围（灰色背景）
- **列表** - `- 列表项` 或 `* 列表项`

## 工具调用

Oxide 提供 9 个核心工具供 AI 使用：

1. **read_file** - 读取文件内容
2. **write_file** - 写入文件内容（自动创建不存在的目录）
3. **edit_file** - 使用 unified diff patch 编辑文件（适用于小范围修改）
4. **create_directory** - 创建目录（包括父目录）
5. **delete_file** - 删除文件或目录
6. **grep_search** - 使用正则表达式搜索文件内容
7. **scan_codebase** - 扫描并显示代码库目录结构
8. **shell_execute** - 执行 Shell 命令
9. **glob** - 文件模式匹配

**额外工具（已实现但未完全集成）：**

- `multiedit` - 多文件编辑
- `notebook_edit` - Jupyter Notebook 编辑
- `ask_user_question` - 询问用户问题
- `git_guard` - Git 操作保护
- `commit_linter` - Commit 消息检查
- `task`, `task_output` - 后台任务管理

### 工具使用示例

**使用 edit_file 进行小范围修改：**

```
你> 修改 main.rs 的第 10 行，添加注释
[工具] edit_file
patch: --- a/main.rs
+++ b/main.rs
@@ -8,3 +8,4 @@
     let x = 5;
     let y = 10;
+    // Calculate sum
     let sum = x + y;
```

**使用 grep_search 搜索代码：**

```
你> 搜索所有 .rs 文件中的 "fn main" 函数
[工具] grep_search
query: fn main
root_path: .
找到 5 个匹配项在 3 个文件中
```

## Skill 系统

Skill 系统允许你创建可重用的自定义命令模板，避免重复输入相同的提示词。

### 内置技能

Oxide 提供了一些常用的内置技能：

- `/commit` - 创建符合 Conventional Commits 规范的 git commit
- `/compact` - 压缩当前会话，创建摘要
- `/review` - 审查代码并提供反馈

### 使用技能

```bash
# 列出所有可用技能
/skills list

# 查看技能详情
/skills show commit

# 执行技能（带参数）
/commit -m "feat: add new feature"
```

### 创建自定义技能

你可以创建自己的技能文件，存放在以下位置（按优先级排序）：

1. **本地技能** - `.oxide/skills/` - 项目本地技能（最高优先级）
2. **全局技能** - `~/.oxide/skills/` - 全局技能
3. **内置技能** - 内置在代码中的技能（最低优先级）

技能加载时会按照以上顺序查找，优先使用高优先级的技能。

技能文件格式（Markdown + Front Matter）：

```markdown
---
name: my-skill
description: My custom skill description
args:
  - name: param1
    description: First parameter
    required: true
  - name: param2
    description: Second parameter
    required: false
---

Your skill template goes here.
Use {{param1}} and {{param2}} as placeholders.

The user provided: {{param1}} and {{param2}}
```

### 技能示例

**创建代码审查技能：**

```bash
# 创建 .oxide/skills/code-review.md
cat > .oxide/skills/code-review.md << 'EOF'
---
name: code-review
description: Perform a thorough code review
args:
  - name: file
    description: File path to review
    required: true
---

Please review the code in {{file}} focusing on:
1. Code quality and style
2. Potential bugs
3. Performance issues
4. Security concerns

Provide specific, actionable feedback.
EOF
```

**使用自定义技能：**

```bash
/code-review -file "src/main.rs"
```

## 会话管理

Oxide 自动保存对话历史，支持：

- 自动保存当前会话
- 查看所有历史会话
- 加载之前的会话
- 删除不需要的会话
- 每个会话有唯一的 ID

## 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_config_validation
```

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release
```

### 项目结构

```
oxide/
├── src/
│   ├── main.rs           # 主入口
│   ├── config.rs        # 配置管理
│   ├── context.rs       # 会话上下文管理
│   ├── tools/          # 工具实现
│   │   ├── mod.rs
│   │   ├── edit_file.rs
│   │   ├── grep_search.rs
│   │   └── ...
├── docs/               # 文档
│   └── architecture.md # 架构文档
├── .oxide/             # 会话数据目录
│   └── sessions/       # 保存的会话
└── .env.example        # 配置示例
```

## 文档

- 📖 [架构文档](docs/architecture.md) - 深入了解 Oxide 的架构设计
- 📝 [使用指南](USAGE.md) - 详细的使用说明
- 🎨 [Markdown 渲染](docs/MARKDOWN_RENDERING.md) - Markdown 渲染功能详解

## 待办事项 (TODO)

### CLI 增强

- [ ] **Prompt 样式优化**
  - 添加颜色样式到 prompt（会话 ID、Agent 类型、Token 计数）
  - 使用 `reedline::Style` 实现更丰富的视觉效果
  - 支持自定义颜色主题

- [ ] **Prompt 信息扩展**
  - 显示当前使用的模型名称
  - 显示后台任务数量
  - 显示未读消息或通知数量
  - 显示当前会话的轮次计数

- [ ] **语法高亮**
  - 实现 `Highlighter` trait
  - 为命令（`/commands`）添加高亮
  - 为文件引用（`@files`）添加高亮
  - 为标签（`#tags`）添加高亮

- [ ] **智能提示 (Hinter)**
  - 实现 `Hinter` trait
  - 基于历史记录的输入建议
  - 使用 LRU 算法优先显示常用命令
  - 显示灰色的自动完成建议

- [ ] **输入验证**
  - 实现 `Validator` trait
  - 验证命令语法
  - 验证文件路径是否存在
  - 提供实时错误提示

- [ ] **菜单样式优化**
  - 尝试不同的菜单样式（`ListMenu`、`ColumnarMenu`）
  - 自定义菜单边框和颜色
  - 支持菜单快捷键

- [ ] **多行编辑支持**
  - 支持复杂的多行输入（如代码块）
  - 改进多行编辑的用户体验
  - 添加多行编辑的可视化提示

### 会话管理

- [ ] **会话搜索和过滤**
  - 按日期范围搜索会话
  - 按关键词搜索会话内容
  - 按标签或类型过滤会话

- [ ] **会话导出**
  - 导出为 Markdown
  - 导出为 JSON
  - 导出为 PDF（需要额外依赖）

### 工具系统

- [ ] **新工具**
  - 文件监控（watch）
  - 批量重命名文件
  - Git 集成工具（分支管理、合并等）
  - HTTP 请求工具

- [ ] **工具性能优化**
  - 优化大文件读取性能
  - 添加进度条显示
  - 支持异步工具执行

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

感谢以下开源项目：

- [Rust](https://www.rust-lang.org/)
- [CLAP](https://github.com/clap-rs/clap)
- [Tokio](https://tokio.rs/)
- [Regex](https://github.com/rust-lang/regex)
- [Ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore)

## 📮 联系方式

- 项目主页: [https://github.com/zzfn/oxide](https://github.com/zzfn/oxide)
- 问题反馈: [GitHub Issues](https://github.com/zzfn/oxide/issues)
- 讨论区: [GitHub Discussions](https://github.com/zzfn/oxide/discussions)

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给个 Star！**

Made with ❤️ by [zzfn](https://github.com/zzfn)

</div>
