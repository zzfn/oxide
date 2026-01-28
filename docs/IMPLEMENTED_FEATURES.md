# Oxide 已实现功能清单

> 最后更新: 2026-01-28
> 状态: 持续更新

本文档记录 Oxide 项目中已完整实现的功能特性。

---

## 📊 已实现功能总览

| 功能模块 | 实现时间 | 相关文件 | 测试覆盖 |
|---------|---------|---------|---------|
| **计划模式系统** | 2026-01-28 | src/tools/plan_mode.rs | ✅ 完整 |

---

## 🎯 核心功能

### 1. 计划模式系统 (Plan Mode)

**实现时间**: 2026-01-28 (commit: 523b0e0)
**对标**: Claude Code EnterPlanMode/ExitPlanMode

#### 功能描述

计划模式系统允许 Agent 在执行复杂任务前，先进入计划模式设计实现方案，并请求用户批准后再执行。这确保了 Agent 的行为符合用户预期，避免浪费时间和资源。

#### 核心组件

1. **EnterPlanModeTool** (src/tools/plan_mode.rs:231-333)
   - 进入计划模式
   - 创建计划 ID: `plan_YYYYMMDD_HHMMSS`
   - 初始化计划目录: `.oxide/plans/`
   - 显示友好的终端 UI

2. **ExitPlanModeTool** (src/tools/plan_mode.rs:374-615)
   - 退出计划模式并请求用户批准
   - 显示计划内容和权限列表
   - 用户交互式审批（批准/修改/取消）
   - 保存计划到 Markdown 文件

3. **PlanModeManager** (src/tools/plan_mode.rs:135-199)
   - 全局状态管理器（单例模式）
   - 管理计划模式状态（active, approved, plan_content）
   - 权限验证机制

4. **AllowedPrompt** (src/tools/plan_mode.rs:21-42)
   - 权限描述结构（tool + prompt）
   - 权限匹配和验证
   - 支持语义化权限描述

#### 实现特性

##### 1. 计划文件管理
- 自动生成计划 ID: `plan_YYYYMMDD_HHMMSS`
- 保存路径: `.oxide/plans/<plan_id>.md`
- Markdown 格式，包含：
  - 元数据（生成时间、状态）
  - 权限列表
  - 计划内容

##### 2. 用户审批流程
- 显示计划内容（超过 2000 字符自动截断）
- 显示需要的权限列表
- 三选一交互：
  - ✅ 批准并执行
  - ✏️ 修改计划（支持输入修改意见）
  - ❌ 取消
- 批准后保留权限状态供后续验证

##### 3. 权限管理
- `AllowedPrompt` 结构：
  ```rust
  pub struct AllowedPrompt {
      pub tool: String,    // 工具名称（如 "Bash", "Write"）
      pub prompt: String,  // 权限描述（如 "run tests", "install dependencies"）
  }
  ```
- 权限验证机制：
  - 检查工具名称匹配
  - 语义化匹配操作描述
  - 批准后才允许执行
- 批准后保留权限状态

##### 4. 友好的终端 UI
- 彩色输出（使用 `colored` crate）
- 表格边框和分隔线
- 清晰的状态提示
- 进度反馈

#### 使用示例

```rust
// Agent 自动调用 EnterPlanMode
// 用户看到：
╔══════════════════════════════════════════════════════════════╗
║                    📋 进入计划模式                            ║
╚══════════════════════════════════════════════════════════════╝

计划 ID: plan_20260128_143022
计划文件: .oxide/plans/plan_20260128_143022.md

在计划模式下，你可以：
  • 探索代码库，了解现有架构
  • 设计实现方案
  • 使用 exit_plan_mode 提交计划并请求用户批准

// Agent 设计完方案后调用 ExitPlanMode
// 用户看到：
╔══════════════════════════════════════════════════════════════╗
║                    📋 计划审批请求                            ║
╚══════════════════════════════════════════════════════════════╝

📝 计划内容:
────────────────────────────────────────────────────────────
# 实现任务管理系统

## 目标
完善任务管理系统，添加依赖关系和元数据支持。

## 实施步骤
1. 扩展 Task 数据结构
2. 实现依赖关系管理
3. 添加单元测试
────────────────────────────────────────────────────────────

🔐 需要的权限:
  1. Bash - run tests
  2. Write - create new files
  3. Edit - modify existing files

请选择操作:
  ❯ 批准并执行 - Approve and execute the plan
    修改计划 - Request modifications to the plan
    取消 - Cancel and discard the plan
```

#### 集成状态

- ✅ 已集成到主 Agent (src/agent/builder.rs:385-386)
- ✅ 工具定义完整
- ✅ 单元测试覆盖（8 个测试用例）

#### 测试覆盖

```rust
// src/tools/plan_mode.rs:727-788
#[cfg(test)]
mod tests {
    #[test] fn test_allowed_prompt_creation()
    #[test] fn test_allowed_prompt_matches()
    #[test] fn test_plan_mode_state_default()
    #[test] fn test_plan_mode_state_enter()
    #[test] fn test_plan_mode_state_exit()
    #[test] fn test_plan_mode_state_is_allowed()
}
```

#### 相关文件

- `src/tools/plan_mode.rs` - 完整实现（789 行）
- `src/agent/builder.rs` - 工具集成
- `.oxide/plans/` - 计划文件存储目录

#### 技术亮点

1. **全局状态管理**: 使用 `Lazy<PlanModeManager>` 实现线程安全的全局单例
2. **类型安全**: 完整的类型定义和错误处理
3. **用户体验**: 友好的终端 UI 和交互流程
4. **可测试性**: 完整的单元测试覆盖
5. **可扩展性**: 支持远程会话推送（预留接口）

#### 与 Claude Code 的对比

| 特性 | Claude Code | Oxide | 说明 |
|-----|-------------|-------|------|
| EnterPlanMode | ✅ | ✅ | 完全实现 |
| ExitPlanMode | ✅ | ✅ | 完全实现 |
| 计划文件保存 | ✅ | ✅ | Markdown 格式 |
| 权限管理 | ✅ | ✅ | AllowedPrompt 系统 |
| 用户审批流程 | ✅ | ✅ | 三选一交互 |
| 远程会话推送 | ✅ | ⚠️ | 接口预留，未实现 |

#### 未来改进方向

- [ ] 支持计划版本控制
- [ ] 支持计划模板
- [ ] 支持远程会话推送
- [ ] 支持计划历史查看
- [ ] 支持计划导出和分享

---

## 📝 更新日志

### 2026-01-28
- ✅ 实现计划模式系统（EnterPlanMode/ExitPlanMode）
- ✅ 实现权限管理系统（AllowedPrompt）
- ✅ 集成到主 Agent
- ✅ 添加完整的单元测试

---

## 🔗 相关文档

- [待实现功能清单](./TODO_FEATURES.md)
- [功能对比总览](./CLAUDE_CODE_COMPARISON.md)
- [架构文档](./architecture.md)
- [工具系统详解](./tool-system.md)
