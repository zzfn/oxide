# CLI 使用指南

## 启动与退出

- 运行 `oxide` 进入 CLI 会话
- 使用 `/exit` 或 `/quit` 优雅退出
- 也可使用 `Ctrl+C` 强制终止

## 斜杠命令

- `/help` - 查看帮助
- `/clear` - 清空对话历史，重置会话
- `/history` - 查看对话历史
- `/sessions` - 列出已保存的会话
- `/load <session_id>` - 加载会话
- `/delete <session_id>` - 删除会话
- `/config show|edit|reload|validate` - 配置管理
- `/agent list|switch <type>|capabilities` - Agent 管理
- `/tasks list|show <id>|cancel <id>` - 任务管理
- `/skills list|show <name>` - 技能管理

## 使用示例

### 基本对话

1. 运行 `oxide` 进入 CLI
2. 输入问题并回车发送
3. 查看模型回复

### 文件操作

```
读取 README.md 文件的内容
在项目中创建一个新的配置文件
```

### Shell 命令执行

```
查看当前目录的文件
统计代码行数
```

### 使用斜杠命令

```
/clear
/exit
```

## 多轮对话

Oxide 支持多轮对话，会记住之前的上下文：

```
你>[0] 什么是 Rust？
Rust 是一门系统级编程语言，注重安全性、并发性和性能...

你>[1] 它有哪些特点？
基于我们之前的讨论，Rust 的主要特点包括：所有权系统、零成本抽象、模式匹配...
```

## Markdown 渲染

Oxide CLI 支持 AI 回复的实时 Markdown 渲染，包括：

- **标题** - `# 标题` 显示为青色
- **粗体** - `**粗体**` 显示为白色高亮
- **斜体** - `*斜体*` 显示为黄色
- **行内代码** - `` `代码` `` 显示为绿色
- **代码块** - 三反引号包围，带灰色背景
- **列表** - 支持 `-` 和 `*` 开头的列表

注意事项：

- 表格 Markdown 语法会以原始文本显示（不渲染成表格）
- 渲染是逐行进行的，保持流式输出的响应速度
- 代码块需要终端支持背景色

## 故障排除

### 配置问题

**问题：启动时报错 "未找到 OXIDE_AUTH_TOKEN 环境变量"**

解决方案：
1. 设置环境变量：`export OXIDE_AUTH_TOKEN=your_key`
2. 或创建 `.env` 文件并添加：`OXIDE_AUTH_TOKEN=your_key`

**问题：打字机效果过慢或过快**

解决方案：
- 设置 `STREAM_CHARS_PER_TICK` 调整流式速度（例如 `export STREAM_CHARS_PER_TICK=12`）

### 网络问题

**问题：API 请求超时**

解决方案：
- 检查网络连接
- 确认能够访问配置的 `base_url`
- 如果使用代理，请配置环境变量

### 终端问题

**问题：颜色显示异常**

解决方案：
- 确保终端支持 ANSI 颜色代码
- 某些终端可能需要启用颜色支持

**问题：Markdown 格式显示异常**

解决方案：
- Oxide 使用 `termimad` 库渲染 Markdown
- 确保终端宽度足够（建议 80 列以上）
- 如果表格格式错位，这是正常现象（表格以原始文本显示）

## 获取帮助

- 查看项目 README.md
- 输入 `/help` 查看内置帮助
