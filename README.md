# Oxide CLI

Oxide 是一个基于 Rust 的命令行 AI 助手，支持多种 LLM 提供商，提供智能对话和工具调用功能。

## 功能特性

- 🤖 支持多种 LLM（DeepSeek、OpenAI、Anthropic 等）
- 🛠️ 完整的工具调用系统（文件读写、编辑、搜索、Shell 命令等）
- 💾 会话持久化和管理
- 🎨 彩色输出和格式化显示
- 📝 使用 unified diff patch 进行文件编辑
- 🔍 正则表达式搜索和代码库扫描
- ⚙️ 灵活的配置管理（支持多个 API 端点和模型）
- 💬 多轮对话历史管理

## 安装

### 从源代码编译

```bash
# 克隆仓库
git clone https://github.com/yourusername/oxide.git
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
API_KEY=your_api_key_here
API_URL=https://api.deepseek.com/v1/chat/completions
MODEL_NAME=deepseek-chat
MAX_TOKENS=4096
```

配置说明：
- `API_KEY`: LLM 提供商的 API 密钥（必需）
- `API_URL`: API 端点 URL（可选，默认为 DeepSeek）
- `MODEL_NAME`: 使用的模型名称（可选，默认为 deepseek-chat）
- `MAX_TOKENS`: 最大 token 数（可选，默认为 4096）

### 支持的模型

Oxide 支持以下 LLM 提供商：
- **DeepSeek** - `deepseek-chat`, `deepseek-coder`
- **OpenAI** - `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`, `gpt-3.5-turbo`
- **Anthropic** - `claude-3-5-sonnet`, `claude-4-opus`
- **其他 OpenAI 兼容的 API** - 只需配置正确的 API_URL 和 MODEL_NAME

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

| 命令 | 说明 |
|------|------|
| `/help` 或 `/?` | 显示帮助信息 |
| `/clear` | 清空当前对话 |
| `/config` | 显示当前配置 |
| `/history` | 显示当前会话的历史消息 |
| `/list` | 列出所有保存的会话 |
| `/load <id>` | 加载指定的会话 |
| `/delete` | 删除当前会话 |
| `/exit` 或 `/quit` | 退出程序 |

### 对话示例

```
==================================================
Oxide CLI 0.1.0 - DeepSeek Agent
==================================================
模型: deepseek-chat
会话: violet-sky-1234
提示: 输入 /help 查看帮助
提示: 输入 /exit 退出

你>[0] 你好！
你好！我是 Oxide 助手，有什么可以帮助你的吗？

你>[1] 帮我查看当前目录的文件结构
[工具] scan_codebase
...
```

## 工具调用

Oxide 提供以下工具：

1. **read_file** - 读取文件内容
2. **write_file** - 写入文件内容（自动创建不存在的目录）
3. **edit_file** - 使用 unified diff patch 编辑文件（适用于小范围修改）
4. **create_directory** - 创建目录（包括父目录）
5. **delete_file** - 删除文件或目录
6. **grep_search** - 使用正则表达式搜索文件内容
7. **scan_codebase** - 扫描并显示代码库目录结构
8. **shell_execute** - 执行 Shell 命令

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

## 项目结构

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
│   └── tui/           # TUI 界面
├── .oxide/             # 会话数据目录
│   └── sessions/       # 保存的会话
└── .env.example        # 配置示例
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
