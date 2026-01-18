# CLI 使用指南

## 快捷键

### 终端快捷键

Oxide CLI 运行在终端环境中，支持标准的终端快捷键：

- `Ctrl+C` - 强制终止程序（推荐使用 `/exit` 优雅退出）
- `Ctrl+D` - 发送 EOF（End of File），通常会导致程序退出
- `Ctrl+L` - 清空终端屏幕（不会清除对话历史）
- `Ctrl+U` - 清除当前输入行的内容
- `Ctrl+A` - 移动光标到行首
- `Ctrl+E` - 移动光标到行末

### 斜杠命令

- `/help` - 显示帮助信息，列出所有可用命令
- `/clear` - 清空对话历史，重置会话
- `/exit` - 优雅退出程序

## 使用示例

### 基本对话

```
==================================================
Oxide CLI 0.1.0 - DeepSeek Agent
==================================================
模型: deepseek-chat
提示: 输入 /help 查看帮助
提示: 输入 /exit 退出

你>[0] 你好！
你好！我是 Oxide 助手，有什么可以帮助你的吗？

你>[1] 请用中文回答
好的，我会用中文与你交流。有什么问题需要我帮助解决吗？
```

### 文件操作

```
你>[2] 读取 README.md 文件的内容
[工具] read_file
{"path": "README.md"}

文件内容已读取完成。README.md 包含了项目的介绍和使用说明。

你>[3] 在项目中创建一个新的配置文件
[工具] write_file
{"path": "config.json", "content": "{\"debug\": true}"}

配置文件 config.json 已创建成功，内容为：
{
  "debug": true
}
```

### Shell 命令执行

```
你>[4] 查看当前目录的文件
[工具] shell_execute
{"command": "ls -la"}

当前目录的文件列表：
total 24
drwxr-xr-x  4 user  group   128 Jan 19 10:00 .
drwxr-xr-x 10 user  group   320 Jan 19 10:00 ..
-rw-r--r--  1 user  group    50 Jan 19 10:00 README.md
-rw-r--r--  1 user  group    20 Jan 19 10:00 config.json

你>[5] 统计代码行数
[工具] shell_execute
{"command": "find src -name '*.rs' | xargs wc -l"}

统计结果：
     150 src/main.rs
      50 src/config.rs
       30 src/lib.rs
     230 total
```

### 错误处理

```
你>[6] 读取一个不存在的文件
[工具] read_file
{"path": "nonexistent.txt"}

错误: No such file or directory (os error 2)

你>[7] 执行一个无效的命令
[工具] shell_execute
{"command": "invalidcommand123"}

错误: command not found: invalidcommand123
```

### 使用斜杠命令

```
你>[8] /help

可用命令:
  /help  - 显示此帮助信息
  /clear  - 清空对话历史
  /exit  - 退出程序

你>[9] /clear
对话历史已清空

你>[0] 这是一个新的对话开始
好的，我们可以开始新的对话了！之前的对话历史已被清空。

你>[1] /exit
再见!
```

## 高级技巧

### 多轮对话

Oxide 支持多轮对话，会记住之前的对话上下文：

```
你>[0] 什么是 Rust？
Rust 是一门系统级编程语言，注重安全性、并发性和性能...

你>[1] 它有哪些特点？
基于我们之前的讨论，Rust 的主要特点包括：所有权系统、零成本抽象、模式匹配...
```

### 工具链调用

AI 可以自动调用多个工具来完成复杂任务：

```
你>[0] 帮我检查项目的构建状态
[工具] shell_execute
{"command": "cargo check"}

[工具] shell_execute
{"command": "cargo test"}

项目检查通过，所有测试也正常运行。
```

### 文件批量操作

AI 可以批量处理多个文件：

```
你>[0] 统计项目中所有 .rs 文件的代码行数
[工具] shell_execute
{"command": "find . -name '*.rs' -exec wc -l {} + | tail -1"}

项目中共有 1500 行 Rust 代码。
```

## 故障排除

### 配置问题

**问题：启动时报错 "未找到 DEEPSEEK_API_KEY 环境变量"**

解决方案：
1. 设置环境变量：`export DEEPSEEK_API_KEY=your_key`
2. 或创建 `.env` 文件并添加：`DEEPSEEK_API_KEY=your_key`

**问题：API Key 验证失败**

解决方案：
- 确保 API Key 格式正确（以 `sk-` 开头）
- 检查 API Key 是否有效（登录 DeepSeek 控制台验证）

### 网络问题

**问题：API 请求超时**

解决方案：
- 检查网络连接
- 确认能够访问 https://api.deepseek.com
- 如果使用代理，请配置环境变量

### 终端问题

**问题：颜色显示异常**

解决方案：
- 确保终端支持 ANSI 颜色代码
- 某些终端可能需要启用颜色支持
- Windows 用户可能需要使用 Windows Terminal

## 最佳实践

1. **定期清空对话历史**：如果对话历史过长，使用 `/clear` 重置会话
2. **使用环境变量管理配置**：推荐使用 `.env` 文件存储敏感信息
3. **保护 API Key**：不要将 `.env` 文件提交到版本控制系统
4. **定期更新**：关注项目更新，获取最新功能和修复

## 获取帮助

如果遇到问题，可以通过以下方式获取帮助：

- 查看项目 README.md
- 输入 `/help` 查看内置帮助
- 访问项目 GitHub Issues 页面
- 查阅 DeepSeek API 文档
