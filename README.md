# Oxide CLI

Oxide 是一个基于 Rust 的命令行 AI 助手，使用 DeepSeek API 提供智能对话和工具调用功能。

## 功能特性

- 🤖 基于 DeepSeek Chat API 的智能对话
- 🛠️ 内置工具调用（文件读写、Shell 命令执行）
- 🎨 彩色输出和格式化显示
- 💬 多轮对话历史管理
- ⚙️ 配置管理（环境变量、.env 文件）

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

设置 `DEEPSEEK_API_KEY` 环境变量：

```bash
export DEEPSEEK_API_KEY=your_api_key_here
```

### .env 文件

在项目根目录创建 `.env` 文件：

```bash
# 复制示例文件
cp .env.example .env

# 编辑 .env 文件，填入你的 API Key
```

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

- `/help` - 显示帮助信息
- `/clear` - 清空对话历史
- `/exit` - 退出程序

### 对话示例

```
==================================================
Oxide CLI 0.1.0 - DeepSeek Agent
==================================================
模型: deepseek-chat
提示: 输入 /help 查看帮助
提示: 输入 /exit 退出

你>[0] 你好！
你好！我是 Oxide 助手，有什么可以帮助你的吗？

你>[1] 帮我查看当前目录
[工具] shell_execute
"ls"
... (命令输出)

你>[2] /help

可用命令:
  /help  - 显示此帮助信息
  /clear  - 清空对话历史
  /exit  - 退出程序

你>[3] /exit
再见!
```

## 工具调用

Oxide 支持以下工具：

1. **read_file** - 读取文件内容
2. **write_file** - 写入文件内容（会自动创建不存在的目录）
3. **shell_execute** - 执行 Shell 命令

AI 助手会根据你的请求自动调用这些工具。

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

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
