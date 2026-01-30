# API 兼容性问题

## 问题描述

使用自定义 API 端点 `https://open.bigmodel.cn/api/anthropic` 时，AI 返回的工具调用参数为空对象 `{}`。

## 调试日志

```
[DEBUG] 工具调用:
  名称: Glob
  输入: {}
  错误: missing field `pattern`
```

## 可能的原因

1. **第三方 API 兼容性问题** - 智谱 AI 的 Anthropic 兼容接口可能不完全支持工具调用
2. **工具参数未正确传递** - API 返回了 ToolUse 块，但 input 字段为空
3. **模型限制** - 使用的模型可能不支持工具调用

## 解决方案

### 方案 1：使用官方 Anthropic API

```bash
export OXIDE_AUTH_TOKEN=your_anthropic_api_key
unset OXIDE_BASE_URL  # 使用默认的 Anthropic API
./target/debug/oxide
```

### 方案 2：测试不同的端点

```bash
# 测试另一个端点
export OXIDE_BASE_URL=https://yunyi.skem.cn/claude
export ANTHROPIC_AUTH_TOKEN=your_token
./target/debug/oxide
```

### 方案 3：添加 API 兼容性检测

在代码中添加检测，如果工具调用参数为空，给出友好提示。

## 测试对比

### 官方 Anthropic API（正常）
```json
{
  "name": "Read",
  "input": {
    "file_path": "/tmp/oxide_test.txt"
  }
}
```

### 智谱 AI 端点（异常）
```json
{
  "name": "Glob",
  "input": {}  ← 空对象
}
```

## 建议

1. 如果可能，使用官方 Anthropic API 进行测试
2. 或者联系智谱 AI 确认其 Anthropic 兼容接口是否完整支持工具调用
3. 考虑添加降级策略：如果检测到工具参数为空，提示用户切换 API 端点
