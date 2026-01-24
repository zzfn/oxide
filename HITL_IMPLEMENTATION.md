# Human-in-the-Loop Gatekeeper 实现总结

## 🎯 实现完成

已成功实现基于规则的 **HITL Gatekeeper**（人机交互守门员），作为工具调用前的智能决策层。

## 📁 文件结构

```
src/agent/
├── hitl_gatekeeper.rs      # HITL 守门员核心实现
├── hitl_integration.rs      # HITL 集成层（使用示例）
└── mod.rs                    # 导出模块

examples/
└── hitl_demo.rs             # HITL 使用演示

docs/
├── src/agents/hitl_agent.md # AI Agent 设计文档（未来扩展）
└── src/tools/confirmation_docs.md  # 之前的通用确认方案（已废弃）
```

## 🚀 核心功能

### 1. **智能决策引擎**

```rust
pub enum HitlDecision {
    ExecuteDirectly,      // 自动批准（安全操作）
    RequireConfirmation,  // 需要确认（破坏性操作）
    RequireChoice,        // 需要选择（多选一）
    Reject,               // 拒绝执行（危险操作）
}
```

### 2. **快速路径优化**

对于已知的低风险操作，**零开销**自动批准：

- ✅ `read_file`, `glob`, `grep_search` - 只读操作
- ✅ `git status`, `ls`, `pwd` - 安全命令

### 3. **风险分级规则**

| 操作类型 | 风险等级 | 默认行为 |
|---------|---------|---------|
| 删除文件 | High | 总是确认 |
| 执行命令 | Medium | 需要确认 |
| 修改文件 | Low | 信任分数高时自动批准 |
| 危险命令 | Critical | 拒绝执行 |

### 4. **信任度系统**

```rust
pub struct TrustConfig {
    pub initial_score: 0.5,              // 初始信任分数
    pub auto_approve_threshold: 0.8,     // 自动批准阈值
    pub increment: 0.02,                 // 确认后增加
    pub decrement: 0.05,                 // 拒绝后减少
}
```

**工作机制：**
- 用户确认 → 信任分数 +0.02
- 用户拒绝 → 信任分数 -0.05
- 信任分数 ≥ 0.8 → 低风险操作自动批准

### 5. **危险命令检测**

自动拒绝危险命令：
```rust
"rm -rf", "rm -fr",             // 删除所有文件
":(){:|:&};:",                   // fork bomb
"dd if=/dev/zero", "mkfs",      // 破坏磁盘
"format", "shutdown", "reboot",  // 系统操作
"kill -9",                       // 强制杀进程
```

## 📖 使用方法

### 方式 1：独立使用 HITL Gatekeeper

```rust
use oxide::agent::{HitlGatekeeper, HitlConfig, ToolCallRequest};

// 创建守门员
let config = HitlConfig::default();
let gatekeeper = HitlGatekeeper::new(config)?;

// 评估操作
let request = ToolCallRequest {
    tool_name: "delete_file".to_string(),
    args: json!({ "file_path": "/tmp/file.txt" }),
    context: build_operation_context(...),
};

match gatekeeper.evaluate_tool_call(request).await? {
    HitlDecision::ExecuteDirectly { reason } => {
        // 直接执行
    }
    HitlDecision::RequireConfirmation { reason, warning_level } => {
        // 询问用户
        let confirmed = ask_user(&reason)?;
        if confirmed {
            gatekeeper.record_success(operation).await;
        } else {
            gatekeeper.record_rejection().await;
        }
    }
    HitlDecision::Reject { reason, suggestion } => {
        // 拒绝执行
        eprintln!("❌ {}", reason);
    }
}
```

### 方式 2：使用 HitlIntegration（推荐）

```rust
use oxide::agent::{HitlIntegration, HitlResult};

let hitl = HitlIntegration::new()?;

match hitl.evaluate_and_confirm(request).await? {
    HitlResult::Approved => {
        // 用户批准，执行工具
        execute_tool(...).await?;
        hitl.record_success(operation).await;
    }
    HitlResult::Rejected => {
        // 用户拒绝
        println!("操作已取消");
    }
}
```

### 配置选项

```bash
# 环境变量控制
export OXIDE_HITL_ENABLED=true        # 启用 HITL（默认）
export OXIDE_HITL_ENABLED=false       # 禁用 HITL
```

**注意：** HITL Gatekeeper 不需要额外的 API Key，直接使用规则引擎和信任度系统进行决策。如果未来需要 AI 增强决策，可以复用主 Agent 的模型。

## 🔧 集成到主 Agent

### 方案 A：在 CLI 命令处理中集成

**位置：** `src/cli/command.rs`

```rust
pub async fn execute_with_hitl(&self, tool_name: &str, args: Value) -> Result<Value> {
    let hitl = HitlIntegration::new()?;

    let request = ToolCallRequest {
        tool_name: tool_name.to_string(),
        args: args.clone(),
        context: self.build_context().await?,
    };

    match hitl.evaluate_and_confirm(request).await? {
        HitlResult::Approved => {
            // 执行工具
            self.tool.call(args).await
        }
        HitlResult::Rejected => {
            Err(Error::Cancelled)
        }
    }
}
```

### 方案 B：作为 Tool Wrapper

创建通用包装器：

```rust
pub struct HitlWrappedTool<T> {
    inner: T,
    hitl: Arc<HitlIntegration>,
}

impl<T: Tool> Tool for HitlWrappedTool<T> {
    async fn call(&self, args: T::Args) -> Result<T::Output, T::Error> {
        let request = build_request(&args);
        let hitl = self.hitl.evaluate_and_confirm(request).await?;

        match hitl {
            HitlResult::Approved => self.inner.call(args).await,
            HitlResult::Rejected => Err(T::Error::cancelled()),
        }
    }
}
```

## 📊 实际效果

### 场景 1：安全操作（自动批准）

```bash
$ oxide "执行 git status"

✓ 自动批准: 安全的只读命令
[git status 输出...]
```

### 场景 2：删除文件（需要确认）

```bash
$ oxide "删除 /tmp/test.txt"

⚠️ 即将删除文件

═══════════════════════════════════════════════════════════════
确认
═══════════════════════════════════════════════════════════════
确认执行此操作？

  › 1. 确认 - 继续执行操作
  › 2. 取消 - 取消此操作

请选择 (1-2): 1
✓ 文件已删除
```

### 场景 3：危险命令（拒绝）

```bash
$ oxide "执行 rm -rf /"

❌ 操作被拒绝
检测到危险命令

💡 建议:
  请考虑使用更安全的替代方案
```

### 场景 4：高信任分数（自动批准）

```bash
# 信任分数: 0.85 (已自动批准 10 次操作)
$ oxide "编辑 src/main.rs"

✓ 自动批准: 信任分数较高 (0.85)，自动批准低风险操作
[文件编辑中...]
```

## 🎨 架构优势

### 与硬编码方案对比

| 特性 | 硬编码方案 | HITL Gatekeeper |
|-----|----------|-----------------|
| 灵活性 | ❌ 固定规则 | ✅ 动态判断 |
| 上下文感知 | ❌ 无 | ✅ 信任分数 + 历史记录 |
| 可扩展性 | ❌ 每个工具单独实现 | ✅ 统一管理 |
| 用户体验 | ❌ 烦扰多 | ✅ 渐进式信任 |
| AI 集成 | ❌ 不支持 | ✅ 预留接口 |

### 核心优势

1. **统一管理** - 所有工具的人机交互逻辑集中在一处
2. **零成本抽象** - 安全操作无任何开销
3. **渐进式信任** - 随时间学习用户习惯
4. **灵活配置** - 环境变量全局控制
5. **可扩展** - 预留 AI 决策接口

## 🔮 未来扩展

### 阶段 1：AI 增强决策（下一步）

```rust
// 在 hitl_gatekeeper.rs 中添加
async fn analyze_with_ai(&self, request: &ToolCallRequest) -> HitlDecision {
    // 使用 Claude 分析操作风险
    let prompt = format!("分析以下工具调用的风险...");
    let response = self.client.chat(&prompt).await?;
    // 解析响应并返回决策
}
```

### 阶段 2：上下文感知

```rust
pub struct OperationContext {
    pub recent_operations: Vec<String>,
    pub current_task: Option<String>,
    pub has_git: bool,
    pub git_branch: Option<String>,
    pub file_importance: Option<f32>,  // 新增：文件重要性
    pub user_intent: Option<String>,    // 新增：用户意图
}
```

### 阶段 3：机器学习

```rust
// 从历史操作中学习
pub struct HitlLearner {
    model: Option<Box<dyn MLModel>>,
    training_data: Vec<OperationRecord>,
}

impl HitlLearner {
    pub async fn learn_from_history(&mut self) {
        // 训练模型预测用户偏好
    }
}
```

## 🧪 测试

```bash
# 编译检查
cargo check

# 运行示例
cargo run --example hitl_demo

# 单元测试
cargo test hitl

# 集成测试
cargo test --test hitl_integration
```

## 📝 总结

这个实现提供了一个**实用的、生产就绪的** HITL 系统：

✅ **已实现**
- 基于规则的智能决策
- 信任度系统
- 危险操作检测
- 统一的集成接口

🔄 **待实现**
- AI 增强决策
- 上下文感知
- 机器学习优化

核心思想从**硬编码规则**进化到**智能守门员**，为未来的 AI 决策打下基础。
