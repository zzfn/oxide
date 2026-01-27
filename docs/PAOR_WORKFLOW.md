# PAOR 结构化思考模式

## 🎯 功能概述

Oxide 现已集成 **PAOR（Plan-Act-Observe-Reflect）结构化思考框架**，能够根据任务复杂度**自动选择**最合适的处理模式：

- **简单任务** → 直接对话模式（快速响应）
- **复杂任务** → PAOR 结构化思考模式（系统化分析）

> **注意**：当前实现为**简化版本**，通过在提示词中嵌入 PAOR 框架来引导 AI 进行结构化思考。完整的工作流引擎（真正的循环迭代、状态管理）正在开发中。

---

## 🤖 自动触发机制

### 复杂度评估标准

系统会根据以下因素评估任务复杂度：

1. **任务长度**
   - 短任务（< 50字）→ 简单模式
   - 中等任务（50-100字）→ 中等复杂度
   - 长任务（≥ 100字）→ 高复杂度

2. **关键词检测**
   - **复杂关键词**（+0.8分）：设计、实现、重构、架构、优化、研究、探索、规划、方案等
   - **多步骤关键词**（+0.5分）：分析、步骤、阶段、流程等
   - **简单关键词**（-0.3分）：什么是、怎么、查看、显示等

3. **评分规则**
   - 总分 ≥ 1.5 → **Complex**（强制使用工作流）
   - 总分 ≥ 0.5 → **Medium**（建议使用工作流）
   - 总分 < 0.5 → **Simple**（简单对话）

---

## 💡 使用示例

### 自动触发示例

#### 简单任务（直接对话）
```bash
oxide> 什么是闭包？
# 使用简单对话模式，快速回答
```

#### 复杂任务（自动启用工作流）
```bash
oxide> 设计并实现一个完整的用户认证系统
# 检测到"设计"和"实现"关键词，自动启用 PAOR 工作流

oxide> 分析整个代码库并重构所有错误处理
# 检测到"分析"、"代码库"、"重构"，自动启用工作流

oxide> 探索项目结构，找出所有 TODO 项，然后创建任务列表
# 检测到多个关键词，自动启用工作流
```

### 手动控制示例

#### 强制启用工作流
```bash
oxide> #workflow 帮我优化这段代码的性能
oxide> 使用工作流重构这个模块
```

#### 强制使用简单对话
```bash
oxide> #simple 列出当前目录的文件
oxide> 简单模式：解释什么是异步编程
```

---

## 🎛️ 命令支持

### `/workflow` 命令

```bash
# 查看工作流状态
oxide> /workflow

# 启用自动模式（默认）
oxide> /workflow on

# 禁用自动模式
oxide> /workflow off
```

---

## 🔄 PAOR 结构化思考阶段

当系统检测到复杂任务时，会在提示词中引导 AI 遵循 PAOR 框架：

1. **Planning（规划）** - 分析任务，制定执行计划
   - 识别子任务和依赖关系
   - 确定需要的工具和资源
   - 预估潜在问题

2. **Acting（执行）** - 按计划执行操作
   - 使用工具（read_file, write_file, edit_file 等）
   - 记录执行过程

3. **Observing（观察）** - 检查执行结果
   - 验证步骤是否成功
   - 收集反馈和数据

4. **Reflecting（反思）** - 总结和改进
   - 评估完成度
   - 总结经验教训

AI 会在响应中展示这个结构化的思考过程。

---

## 📊 配置选项

### OrchestratorConfig

```rust
pub struct OrchestratorConfig {
    /// 最大迭代次数
    pub max_iterations: u32,  // 默认: 15

    /// 是否启用详细日志
    pub verbose: bool,  // 默认: false

    /// 是否自动重试失败的任务
    pub auto_retry: bool,  // 默认: true

    /// 最大重试次数
    pub max_retries: u32,  // 默认: 3
}
```

---

## 🎯 最佳实践

1. **自然表达** - 用自然的中文描述任务，系统会自动判断复杂度
2. **明确目标** - 复杂任务应明确说明期望的结果
3. **使用标记** - 特殊情况下使用 `#workflow` 或 `#simple` 手动控制
4. **提供上下文** - 对于复杂任务，可以先用文件引用（@）提供代码上下文

---

## 🔧 技术实现

### 核心组件

- **ComplexityEvaluator** - 任务复杂度评估器
- **WorkflowExecutor** - 工作流执行器（封装 WorkflowOrchestrator）
- **OxideCli** - CLI 集成层（智能路由）

### 文件结构

```
src/agent/workflow/
├── complexity.rs      # 复杂度评估器
├── executor.rs        # 工作流执行器
├── orchestrator.rs    # PAOR 编排器
├── state.rs          # 工作流状态管理
└── types.rs          # 类型定义

src/cli/
├── command.rs        # 命令处理（包含工作流路由）
└── mod.rs           # CLI 主模块
```

---

## 📈 性能考虑

- **简单任务**：直接对话，无额外开销
- **复杂任务**：PAOR 工作流可能需要多次迭代，但能提供更系统化的解决方案
- **智能选择**：系统自动判断，避免不必要的复杂度

---

## 🚀 未来改进

### Phase 2: 完整工作流引擎（规划中）

- [ ] 真正的迭代循环：Plan → Act → Observe → Reflect → Plan
- [ ] 状态持久化和恢复
- [ ] 任务分解和并行执行
- [ ] 中途人工干预和引导
- [ ] 进度可视化和调试

### 当前增强计划

- [ ] 支持用户自定义复杂度阈值
- [ ] 添加更多 PAOR 模板（代码审查、架构设计等）
- [ ] 支持中途切换模式
- [ ] 添加工作流记忆功能

---

## 📝 相关文档

- [工作流引擎规范](../openspec/changes/add-autonomous-workflow-engine/specs/workflow-engine/spec.md)
- [架构设计文档](../openspec/changes/add-autonomous-workflow-engine/design.md)
- [实现总结](../openspec/changes/add-autonomous-workflow-engine/IMPLEMENTATION_SUMMARY.md)
