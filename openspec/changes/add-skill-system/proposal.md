# Proposal: Skill System

## Summary
实现类似 Claude Code 的 Skill 功能，允许用户创建可重用的 prompt 模板（Skills），通过斜杠命令快速调用。Skills 支持参数传递、工具调用，并可扩展自定义功能。

## Motivation
当前 Oxide 的斜杠命令都是硬编码在 CLI 中的固定命令。用户无法自定义常用的工作流程或 prompt 模板。通过引入 Skill 系统，用户可以：

1. **复用常用 prompt**：将常用的提示词保存为 skill，避免重复输入
2. **参数化执行**：支持传递参数，使 skill 更加灵活
3. **工具集成**：skill 可以调用其他工具或子 Agent
4. **团队共享**：通过版本控制共享项目特定的 skill

## Goals
- 支持从 `~/.oxide/skills/` 和 `.oxide/skills/` 加载 skill 文件（本地优先）
- 使用 Markdown 格式定义 skill（Front matter + Prompt 内容）
- 支持参数传递（如 `/commit -m "message"`）
- 集成到现有的斜杠命令系统
- 提供基础示例 skills（/commit、/compact）

## Non-Goals
- 完整实现 Claude Code 的所有内置 skills（只提供示例）
- Skill marketplace 或在线分享功能
- 动态 skill 热重载（需要重启 CLI）
- Skill 之间的相互调用

## Proposed Solution

### 1. Skill 文件格式
使用 Markdown + Front matter：

```markdown
---
name: commit
description: Create a git commit with conventional commit format
args:
  - name: message
    description: Commit message
    required: true
---

Create a git commit following these guidelines:
- Use conventional commits format (feat:, fix:, docs:, etc.)
- Keep descriptions concise and focused on the "why"
- Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>

User provided message: {{message}}
```

### 2. 存储位置
- **全局**: `~/.oxide/skills/*.md`
- **本地**: `.oxide/skills/*.md`（优先级更高）
- **内置**: 示例技能可随二进制分发

### 3. 执行流程
1. 用户输入 `/commit -m "Fix bug"`
2. CLI 解析 skill 名称和参数
3. 从文件系统加载 skill 模板
4. 替换模板变量（`{{message}}` → "Fix bug"）
5. 将渲染后的内容发送给 AI
6. AI 执行相应操作

### 4. CLI 集成
扩展现有的斜杠命令系统：
- `/skills` 或 `/skills list` - 列出所有可用 skills
- `/skills show <name>` - 显示 skill 详细信息
- `/skills create <name>` - 交互式创建新 skill
- `/<skill-name> [args]` - 执行 skill

## Alternatives Considered

### A. 纯文本格式 (.txt)
**优点**: 简单
**缺点**: 无法添加元数据（描述、参数定义）

### B. YAML/JSON 格式
**优点**: 结构化，易于解析
**缺点**: 不够直观，编辑体验差

### C. 硬编码所有 skills
**优点**: 性能好，无需文件 I/O
**缺点**: 无法扩展，用户无法自定义

## Impact

### Breaking Changes
无。这是新功能，不影响现有功能。

### Dependencies
- 使用现有的 `serde`、`toml` 解析 Front matter
- 可能需要 `handlebars` 或 `tera` 模板引擎
- `dialoguer` 用于交互式创建

### Performance
- 启动时扫描 skill 目录（可缓存）
- 每次执行时读取和渲染模板（开销很小）

## Implementation Tasks
详见 `tasks.md`

## Open Questions
1. 是否需要支持 skill 版本管理？
2. skill 执行失败时如何处理？
3. 是否需要 skill 验证（语法检查）？
