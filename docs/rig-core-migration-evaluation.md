# 迁移到 rig-core 的评估

## 概述

评估是否将 Oxide 的 LLM 集成从自实现迁移到 `rig-core` 库。

## rig-core 简介

- **仓库**: https://github.com/0xPlaygrounds/rig
- **文档**: https://docs.rig.rs
- **特性**:
  - 支持 20+ LLM 提供商（OpenAI, Anthropic, Cohere 等）
  - 统一的 Agent API
  - 工具调用支持
  - 流式响应
  - 向量存储集成
  - WASM 兼容

## 当前实现 vs rig-core

### 当前实现（oxide-provider）

**优点**:
- ✅ 完全控制，精确定制
- ✅ 轻量级，只有需要的功能
- ✅ 无额外依赖
- ✅ 深入理解 LLM API

**缺点**:
- ❌ 需要自己处理边界情况（如 InputJsonDelta bug）
- ❌ 只支持 Anthropic
- ❌ 需要自己维护和更新
- ❌ 可能有未发现的 bug

**代码量**: ~420 行（anthropic.rs）

### 使用 rig-core

**优点**:
- ✅ 成熟的实现，经过生产验证
- ✅ 支持多个 LLM 提供商
- ✅ 社区维护，bug 修复及时
- ✅ 已处理好流式响应、工具调用等细节
- ✅ 更好的错误处理
- ✅ 可能有更好的性能优化

**缺点**:
- ❌ 增加依赖（rig-core + rig-anthropic）
- ❌ 可能有不需要的功能
- ❌ 需要适配现有的类型系统
- ❌ 学习新的 API

**依赖**:
```toml
rig-core = "0.x"
rig-anthropic = "0.x"
```

## 迁移工作量评估

### 需要修改的文件

1. **oxide-provider/Cargo.toml** - 添加 rig 依赖
2. **oxide-provider/src/anthropic.rs** - 重写为 rig 适配器
3. **oxide-core/src/types.rs** - 可能需要调整类型以匹配 rig
4. **oxide-cli/src/agent.rs** - 调整工具调用逻辑

### 预估工作量

- **研究 rig API**: 2-4 小时
- **类型适配**: 2-3 小时
- **重写 Provider**: 3-4 小时
- **测试和调试**: 2-3 小时
- **总计**: 9-14 小时

## 迁移方案

### 方案 A：完全迁移

直接用 rig-core 替换 oxide-provider。

**步骤**:
1. 添加 rig-core 和 rig-anthropic 依赖
2. 创建 rig 到 oxide 类型的转换层
3. 重写 AnthropicProvider 为 rig 的包装器
4. 更新所有调用点
5. 测试验证

**风险**: 中等，可能需要大量类型转换

### 方案 B：渐进式迁移

保留当前实现，添加 rig 作为可选后端。

**步骤**:
1. 创建 RigProvider 实现 LLMProvider trait
2. 在配置中添加 provider 选择
3. 逐步测试和验证
4. 稳定后移除旧实现

**风险**: 低，可以随时回退

### 方案 C：混合方案

使用 rig-core 的核心功能，但保留自定义的工具调用逻辑。

**步骤**:
1. 只使用 rig 的 HTTP 客户端和基础 API
2. 保留自己的工具调用实现
3. 获得 rig 的稳定性，保持控制权

**风险**: 低，最小改动

## 建议

### 如果你的目标是：

1. **学习和深入理解 LLM API**
   - 👉 保持当前实现
   - 刚才修复的 InputJsonDelta bug 就是很好的学习机会

2. **快速开发功能，专注业务逻辑**
   - 👉 迁移到 rig-core（方案 B）
   - 节省时间，专注于 Oxide 的核心价值

3. **支持多个 LLM 提供商**
   - 👉 迁移到 rig-core（方案 A）
   - 轻松支持 OpenAI, Claude, Gemini 等

4. **保持轻量和控制**
   - 👉 保持当前实现
   - 继续完善和优化

### 我的推荐

**短期（现在）**: 保持当前实现
- 刚修复了 InputJsonDelta bug
- 代码已经很完善
- 继续完成 Phase 2 的其他功能

**中期（Phase 3+）**: 评估迁移
- 当需要支持多个 LLM 时
- 当维护成本变高时
- 当需要更多高级特性时

## 下一步

如果决定迁移，我可以帮你：
1. 研究 rig-core 的工具调用 API
2. 创建类型转换层
3. 实现 RigProvider
4. 编写迁移测试

如果保持当前实现，我可以帮你：
1. 继续完善错误处理
2. 添加更多测试
3. 优化性能
4. 完成 Phase 2 的其他功能

你想怎么做？
