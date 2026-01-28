# Plan 模式功能演示

## 🎯 快速开始

### 1. 启动 Oxide
```bash
cargo run --release
```

### 2. 切换到 Plan 模式
按 **Tab** 键两次，你会看到：

```
📋 已切换到 Plan 模式
   所有任务将使用 PAOR 工作流处理
   Planning → Acting → Observing → Reflecting

plan>
```

### 3. 执行一个任务
```
plan> 帮我分析 src/main.rs 的代码结构并提出改进建议
```

### 4. 查看结果
系统会：
1. 显示 "📋 Plan 模式 - 使用 PAOR 工作流引擎"
2. 执行 PAOR 循环（Planning → Acting → Observing → Reflecting）
3. 显示执行统计
4. 保存计划到 `.oxide/plans/plan_YYYYMMDD_HHMMSS.md`
5. 显示保存路径：`💾 计划已保存到: .oxide/plans/plan_20260128_143022.md`

### 5. 查看保存的计划
```bash
cat .oxide/plans/plan_*.md
```

## 🔄 模式切换演示

### 初始状态（Oxide 模式）
```
oxide> [提示符显示为 oxide]
```

### 按一次 Tab（切换到 Fast 模式）
```
⚡ 已切换到 Fast 模式
   使用快速响应模式

fast>
```

### 再按一次 Tab（切换到 Plan 模式）
```
📋 已切换到 Plan 模式
   所有任务将使用 PAOR 工作流处理
   Planning → Acting → Observing → Reflecting

plan>
```

### 再按一次 Tab（回到 Oxide 模式）
```
🦀 已切换到 Oxide 模式
   使用标准对话模式

oxide>
```

## 📋 完整示例

### 示例 1: 代码重构任务

```
plan> 重构 src/cli/mod.rs，提取重复的错误处理逻辑

📋 Plan 模式 - 使用 PAOR 工作流引擎

📋 PAOR 工作流阶段:
  1. Planning  - 分析任务，制定执行计划
  2. Acting    - 执行计划中的任务
  3. Observing - 收集和分析执行结果
  4. Reflecting - 评估进展，决定下一步

🚀 启动 PAOR 工作流...

🔄 迭代 1/15 | 阶段: Planning
[AI 分析代码结构...]

🔄 迭代 2/15 | 阶段: Acting
[AI 执行重构...]

🔄 迭代 3/15 | 阶段: Observing
[AI 收集结果...]

🔄 迭代 4/15 | 阶段: Reflecting
[AI 评估进展...]

✅ 工作流执行成功

📊 执行统计:
  迭代次数: 4
  最终阶段: Complete

💾 计划已保存到: .oxide/plans/plan_20260128_143530.md

📝 工作流摘要:
[详细的重构计划和执行结果]
```

### 示例 2: 架构设计任务

```
plan> 设计一个插件系统，支持动态加载和热重载

📋 Plan 模式 - 使用 PAOR 工作流引擎

[工作流执行过程...]

✅ 工作流执行成功

💾 计划已保存到: .oxide/plans/plan_20260128_144215.md
```

### 查看保存的计划文件

```bash
$ cat .oxide/plans/plan_20260128_143530.md
```

输出：
```markdown
# 工作流计划

> 生成时间: 2026-01-28 14:35:30
> 会话 ID: abc123def456
> 状态: ✅ 成功

---

## 📊 执行统计

- **迭代次数**: 4
- **最终阶段**: Complete

---

## 📋 工作流摘要

### Planning 阶段
1. 分析 src/cli/mod.rs 的代码结构
2. 识别重复的错误处理模式
3. 设计统一的错误处理函数

### Acting 阶段
1. 创建 handle_error() 辅助函数
2. 重构所有错误处理调用
3. 更新单元测试

### Observing 阶段
1. 验证代码编译通过
2. 运行测试套件
3. 检查代码覆盖率

### Reflecting 阶段
1. 代码重复度降低 40%
2. 错误处理更加一致
3. 可维护性提升

---

## 📝 最终响应

已成功重构 src/cli/mod.rs 的错误处理逻辑：

1. **创建了统一的错误处理函数**
   - `handle_error()` - 统一的错误显示
   - `handle_warning()` - 统一的警告显示

2. **重构了 15 处重复代码**
   - 减少代码行数 120 行
   - 提高代码可读性

3. **改进了错误消息**
   - 更清晰的错误描述
   - 添加了解决建议

建议后续优化：
- 添加错误日志记录
- 实现错误恢复机制
- 添加错误统计功能
```

## 🎨 UI 特性展示

### 模式指示器
- 🦀 **Oxide** - 绿色，标准模式
- ⚡ **Fast** - 黄色，快速模式
- 📋 **Plan** - 青色，计划模式

### 工作流阶段指示
```
🔄 迭代 1/15 | 阶段: Planning   ← 规划中
🔄 迭代 2/15 | 阶段: Acting     ← 执行中
🔄 迭代 3/15 | 阶段: Observing  ← 观察中
🔄 迭代 4/15 | 阶段: Reflecting ← 反思中
```

### 成功/失败指示
```
✅ 工作流执行成功          ← 成功
⚠️  工作流执行未完成        ← 未完成
❌ 工作流执行失败          ← 失败
```

## 📁 文件组织

```
.oxide/
├── plans/                          ← 计划文件目录
│   ├── plan_20260128_143022.md    ← 第一个计划
│   ├── plan_20260128_143530.md    ← 第二个计划
│   └── plan_20260128_144215.md    ← 第三个计划
└── sessions/                       ← 会话文件目录
    └── abc123def456.json
```

## 💡 使用技巧

### 1. 何时使用 Plan 模式
- ✅ 复杂的重构任务
- ✅ 多文件修改
- ✅ 架构设计
- ✅ 需要详细记录的变更

### 2. 何时使用 Oxide 模式
- ✅ 简单问答
- ✅ 快速查询
- ✅ 单文件小改动

### 3. 计划文件管理
```bash
# 列出所有计划
ls -lh .oxide/plans/

# 查看最新计划
cat .oxide/plans/plan_*.md | tail -100

# 搜索特定关键词
grep -r "重构" .oxide/plans/

# 清理旧计划（保留最近 10 个）
cd .oxide/plans && ls -t | tail -n +11 | xargs rm
```

## 🔧 故障排除

### 问题 1: Tab 键不响应
**解决**: 确保终端支持 Tab 键绑定，某些终端可能需要配置

### 问题 2: 计划文件未保存
**检查**:
```bash
# 检查目录权限
ls -ld .oxide/plans/

# 手动创建目录
mkdir -p .oxide/plans
```

### 问题 3: 工作流执行失败
**原因**: 可能是 API 配置问题
**解决**:
```bash
# 检查配置
oxide /config show

# 验证配置
oxide /config validate
```

## 📚 相关命令

```bash
# 查看帮助
/help

# 查看工作流状态
/workflow status

# 列出所有会话
/sessions

# 查看历史
/history
```

## 🎉 总结

Plan 模式提供了：
1. ✅ **强大的规划能力** - PAOR 工作流自动规划和执行
2. ✅ **持久化记录** - 所有计划保存为 Markdown 文件
3. ✅ **简洁的交互** - Tab 键快速切换模式
4. ✅ **清晰的反馈** - 实时显示工作流进度

立即尝试：
```bash
cargo run --release
# 按 Tab 两次切换到 Plan 模式
# 输入你的任务
# 查看 .oxide/plans/ 目录中的计划文件
```

---

**提示**: 更多详细信息请参阅 [Plan 模式使用指南](./PLAN_MODE_GUIDE.md)
