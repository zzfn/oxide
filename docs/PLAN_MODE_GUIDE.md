# Plan 模式使用指南

> 最后更新: 2026-01-28
> 状态: ✅ 已实现

## 📋 功能概述

Plan 模式是 Oxide 的一个特殊工作模式，通过 Tab 键切换激活。在此模式下，所有用户输入都会自动使用 PAOR (Plan-Act-Observe-Reflect) 工作流引擎处理，并将生成的计划保存到文件系统。

## 🎯 核心特性

### 1. 强制工作流模式
- 在 Plan 模式下，**所有任务**都会使用 PAOR 工作流处理
- 无需手动添加 `#workflow` 标记
- 自动进行多步骤规划和执行

### 2. 计划文件自动保存
- 计划保存路径: `.oxide/plans/plan_<timestamp>.md`
- 包含完整的执行统计、工作流摘要和最终响应
- Markdown 格式，便于查看和分享

### 3. 可视化模式切换
- 按 Tab 键在三种模式间循环切换：
  - 🦀 **Oxide 模式** (默认) - 标准对话模式
  - ⚡ **Fast 模式** - 快速响应模式
  - 📋 **Plan 模式** - 强制工作流模式
- 切换时显示清晰的提示信息

## 🚀 使用方法

### 启动 Plan 模式

1. 启动 Oxide CLI
2. 按 **Tab** 键两次，切换到 Plan 模式
3. 左侧提示符显示为 `plan>`
4. 看到提示信息：
   ```
   📋 已切换到 Plan 模式
      所有任务将使用 PAOR 工作流处理
      Planning → Acting → Observing → Reflecting
   ```

### 执行任务

在 Plan 模式下输入任何任务，例如：

```
plan> 帮我重构 src/main.rs 文件，提取重复代码
```

系统会自动：
1. 显示 "📋 Plan 模式 - 使用 PAOR 工作流引擎"
2. 执行 PAOR 循环（Planning → Acting → Observing → Reflecting）
3. 显示执行统计和结果
4. 保存计划到 `.oxide/plans/plan_YYYYMMDD_HHMMSS.md`

### 查看保存的计划

```bash
# 列出所有计划
ls -lh .oxide/plans/

# 查看最新计划
cat .oxide/plans/plan_*.md | tail -100

# 使用编辑器打开
code .oxide/plans/plan_20260128_143022.md
```

## 📄 计划文件格式

生成的计划文件包含以下部分：

```markdown
# 工作流计划

> 生成时间: 2026-01-28 14:30:22
> 会话 ID: abc123def456
> 状态: ✅ 成功

---

## 📊 执行统计

- **迭代次数**: 3
- **最终阶段**: Complete

---

## 📋 工作流摘要

[详细的执行步骤和分析]

---

## 📝 最终响应

[AI 的最终回复内容]
```

## 🔄 模式切换流程

```
Oxide (默认)  →  Fast  →  Plan  →  Oxide (循环)
  🦀              ⚡       📋        🦀
```

- **Oxide 模式**: 标准对话，自动检测复杂任务
- **Fast 模式**: 快速响应（预留功能）
- **Plan 模式**: 强制使用工作流，保存计划文件

## 💡 使用场景

### 适合 Plan 模式的任务

1. **复杂重构任务**
   ```
   plan> 重构整个认证系统，使用 JWT 替换 Session
   ```

2. **多文件修改**
   ```
   plan> 添加日志系统到所有 API 端点
   ```

3. **架构设计**
   ```
   plan> 设计一个可扩展的插件系统
   ```

4. **需要审查的变更**
   ```
   plan> 优化数据库查询性能
   ```

### 不适合 Plan 模式的任务

- 简单问答（"什么是 Rust?"）
- 单行代码修改
- 快速查询（"显示当前配置"）

对于这些任务，使用 **Oxide 模式**或 **Fast 模式**更合适。

## 🎨 UI 示例

### 切换到 Plan 模式
```
oxide> [按 Tab 两次]

📋 已切换到 Plan 模式
   所有任务将使用 PAOR 工作流处理
   Planning → Acting → Observing → Reflecting

plan>
```

### 执行任务
```
plan> 重构 src/main.rs

📋 Plan 模式 - 使用 PAOR 工作流引擎

📋 PAOR 工作流阶段:
  1. Planning  - 分析任务，制定执行计划
  2. Acting    - 执行计划中的任务
  3. Observing - 收集和分析执行结果
  4. Reflecting - 评估进展，决定下一步

🚀 启动 PAOR 工作流...

🔄 迭代 1/15 | 阶段: Planning
[...]

✅ 工作流执行成功

📊 执行统计:
  迭代次数: 3
  最终阶段: Complete

💾 计划已保存到: .oxide/plans/plan_20260128_143022.md

📝 工作流摘要:
[详细内容]
```

## 🔧 技术实现

### 关键组件

1. **PromptLabel 枚举** (`src/cli/mod.rs:493-517`)
   - 定义三种模式：Oxide, Fast, Plan
   - 支持循环切换

2. **is_plan_mode() 方法** (`src/cli/command.rs`)
   - 检查当前是否处于 Plan 模式
   - 用于决定是否强制使用工作流

3. **save_plan_to_file() 方法** (`src/cli/command.rs`)
   - 保存工作流结果到 Markdown 文件
   - 自动创建 `.oxide/plans/` 目录

4. **show_mode_switch_hint() 方法** (`src/cli/mod.rs`)
   - 显示模式切换提示信息
   - 提供清晰的视觉反馈

### 工作流程

```
用户按 Tab → 切换 PromptLabel → 显示提示
                                    ↓
用户输入任务 → is_plan_mode() 检查 → 强制使用工作流
                                    ↓
执行 PAOR 循环 → 生成 WorkflowResult → save_plan_to_file()
                                    ↓
                            保存到 .oxide/plans/
```

## 📚 相关文档

- [PAOR 工作流详解](./PAOR_WORKFLOW.md)
- [架构文档](./architecture.md)
- [功能对比](./CLAUDE_CODE_COMPARISON.md)

## 🐛 已知限制

1. **Fast 模式功能未实现**
   - 目前 Fast 模式与 Oxide 模式行为相同
   - 未来可以实现使用更快的模型（如 Haiku）

2. **计划审批流程未实现**
   - 当前计划生成后直接执行
   - 未来可以添加用户确认步骤

3. **计划文件管理**
   - 没有自动清理旧计划文件
   - 需要手动管理 `.oxide/plans/` 目录

## 🚀 未来改进

- [ ] 实现 Fast 模式（使用 Haiku 模型）
- [ ] 添加计划审批流程（生成后等待用户确认）
- [ ] 计划文件版本控制和比较
- [ ] 计划模板系统
- [ ] 计划执行历史查询
- [ ] 导出计划为 PDF/HTML

---

**提示**: 如有问题或建议，请在 GitHub Issues 中反馈。
