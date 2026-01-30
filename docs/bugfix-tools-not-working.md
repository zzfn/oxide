# Bug 修复：工具调用未生效

**发现时间**: 2026-01-30
**严重程度**: 🔴 Critical

## 问题描述

实现了完整的代理主循环和工具注册系统后，发现 AI 不会自动调用工具。用户输入"请读取文件"等指令时，AI 只会回复"我无法访问文件系统"，而不是调用 Read 工具。

## 根本原因

在 `crates/oxide-provider/src/anthropic.rs` 的 `complete_stream_with_tools` 方法中，第 333 行：

```rust
let request = AnthropicRequest {
    model: self.model.clone(),
    messages: api_messages,
    max_tokens: self.max_tokens,
    temperature: self.temperature,
    system,
    tools: None,  // ❌ 硬编码为 None，忽略了传入的 tools 参数
    stream: true,
};
```

虽然方法接收了 `tools: Option<Vec<serde_json::Value>>` 参数，但在构建请求时使用了 `tools: None`，导致工具定义从未发送给 Anthropic API。

## 修复方案

将硬编码的 `None` 改为使用传入的参数：

```rust
let request = AnthropicRequest {
    model: self.model.clone(),
    messages: api_messages,
    max_tokens: self.max_tokens,
    temperature: self.temperature,
    system,
    tools,  // ✅ 使用传入的 tools 参数
    stream: true,
};
```

## 影响范围

- **受影响的功能**: 所有工具调用（Read, Write, Edit, Glob, Grep, Bash, TaskOutput, TaskStop）
- **受影响的方法**: `complete_stream_with_tools`
- **非流式方法**: `complete_with_tools` 已正确实现，未受影响

## 测试验证

### 修复前
```
用户: 请读取 /tmp/oxide_test.txt 文件的内容
AI: 我无法直接访问或读取您的本地文件系统...
```

### 修复后（预期）
```
用户: 请读取 /tmp/oxide_test.txt 文件的内容
Assistant ⚙ 执行工具: Read
  ✓ 工具 Read 执行成功
AI: 文件内容如下：
这是一个测试文件。
用于测试 Oxide CLI 的文件读取功能。
...
```

## 经验教训

1. **参数未使用警告很重要** - 编译器警告 `unused variable: tools` 是一个明确的信号
2. **端到端测试必不可少** - 单元测试通过不代表集成正常工作
3. **复制粘贴代码要小心** - 可能从非工具版本复制了代码，忘记更新

## 相关文件

- `crates/oxide-provider/src/anthropic.rs:333` - 修复位置
- `crates/oxide-cli/src/agent.rs` - 工具调用循环（正常）
- `crates/oxide-tools/src/` - 工具实现（正常）

## 后续行动

- [x] 修复 `complete_stream_with_tools` 中的 bug
- [ ] 进行端到端测试验证修复
- [ ] 添加集成测试防止回归
- [ ] 检查是否有其他类似的硬编码问题
