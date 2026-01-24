# Change: 添加 @文件上下文引用功能

## Why

用户在对话中需要引用文件内容时，目前必须手动复制粘贴文件内容或等待 AI 调用 read_file 工具。这降低了工作效率，特别是在处理代码审查、文件对比等场景时。通过实现类似 Claude Code 的 `@文件路径` 语法，可以让用户直接在输入中引用文件，系统自动读取并注入文件内容到对话上下文中。

## What Changes

- **动态文件路径补全**：当用户输入 `@` 时，动态扫描当前目录并显示文件列表
- **路径递归补全**：输入 `@src/` 时，显示 src 目录下的文件和子目录
- **自动内容注入**：解析用户输入中的 `@文件路径` 引用，自动读取文件内容并注入到对话上下文
- **可视化反馈**：在发送消息前显示已引用的文件列表，包括文件路径、大小和行数
- **错误处理**：当文件不存在或无法读取时，显示清晰的错误提示
- **多文件引用**：支持在单条消息中引用多个文件（如 `@src/main.rs @config.toml`）

## Impact

- **Affected specs**:
  - `specs/cli-core/spec.md` - 添加新的"文件上下文引用"需求
- **Affected code**:
  - `src/cli/mod.rs:91-98` - 扩展现有的 `build_context_entries()` 函数
  - `src/cli/mod.rs:135-189` - 增强 `OxideCompleter` 以支持文件路径补全
  - `src/cli/command.rs:14-183` - 添加输入解析逻辑，提取 `@` 引用并注入文件内容
  - 新建 `src/cli/file_resolver.rs` - 文件路径解析和内容读取模块

## User Experience Flow

```
用户输入: @s[TAB]
          ↓
自动补全显示: @src/  @setup.rs  @Cargo.toml
          ↓
用户继续输入: @src/m[TAB]
          ↓
自动补全显示: @src/main.rs  @src/cli/mod.rs
          ↓
用户选择: @src/main.rs
          ↓
继续输入: 请帮我重构这个文件
          ↓
系统行为:
  1. 读取 src/main.rs 内容
  2. 显示: 📎 已引用: src/main.rs (1234 bytes, 42 lines)
  3. 将文件内容注入到对话上下文
  4. 发送完整消息给 AI
```

## Non-Goals

- 不实现 `@file`、`@codebase` 等二级菜单（直接使用路径）
- 不支持远程文件引用（URL、HTTP）
- 不支持文件范围引用（如 `@file.rs:10-20`）
- 不支持符号链接跟踪（出于安全考虑）
- 不修改现有的 read_file 工具行为
