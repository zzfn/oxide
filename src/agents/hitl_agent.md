# Human-in-the-loop Agent 设计

## 概念

不是每个工具自己决定是否需要确认，而是有一个**统一的 AI Agent** 作为守门员，动态决策是否需要人工介入。

## 架构设计

```
┌──────────────────────────────────────────────────────┐
│  Tool Call                                           │
│  (AI 助手想要执行某个操作)                             │
└────────────────┬─────────────────────────────────────┘
                 ↓
┌──────────────────────────────────────────────────────┐
│  HITL Agent（守门员）                                 │
│  ┌──────────────────────────────────────────────┐   │
│  │ 1. 分析操作内容                              │   │
│  │    - 删除文件？执行命令？修改代码？           │   │
│  │ 2. 评估风险                                  │   │
│  │    - 破坏性？可逆？影响范围？                 │   │
│  │ 3. 检查上下文                                │   │
│  │    - 用户信任度？历史操作？项目重要性？       │   │
│  │ 4. 决策                                      │   │
│  │    - 直接执行？                              │   │
│  │    - 请求确认？                              │   │
│  │    - 要求详细审核？                          │   │
│  └──────────────────────────────────────────────┘   │
└────────────────┬─────────────────────────────────────┘
                 ↓
        ┌────────┴────────┐
        ↓                 ↓
   [需要确认]         [直接执行]
        ↓                 ↓
┌───────────────┐   ┌─────────────┐
│ AskUserQuestion│  │ 执行操作     │
│ 获取用户反馈   │   └─────────────┘
└───────┬───────┘
        ↓
   返回决策结果
        ↓
   执行/拒绝
```

## 工作流程

### 场景 1：删除重要文件

```
AI 助手: "删除 config/database.yml"
         ↓
HITL Agent: 分析 → 这是数据库配置，可能很关键
            → 风险等级：高
            → 决策：必须确认
         ↓
HITL Agent: AskUserQuestion({
    question: "⚠️ 即将删除数据库配置文件，这可能导致应用无法连接数据库",
    options: [
        { label: "取消删除", description: "保留文件" },
        { label: "确认删除", description: "我知道自己在做什么" },
        { label: "先备份再删除", description: "创建备份后删除" }
    ],
    default: "取消删除"
})
         ↓
用户: "先备份再删除"
         ↓
HITL Agent: 执行备份 → 删除原文件 → 返回成功
```

### 场景 2：执行简单查看命令

```
AI 助手: "执行 git status"
         ↓
HITL Agent: 分析 → 这是只读命令，无风险
            → 上下文：用户经常执行此命令
            → 决策：直接执行
         ↓
HITL Agent: 返回 "允许执行"
         ↓
AI 助手: 执行命令 → 返回结果
```

### 场景 3：修改生产环境配置

```
AI 助手: "修改 .env 设置 NODE_ENV=production"
         ↓
HITL Agent: 分析 → 生产环境配置变更
            → 可能影响所有用户
            → 决策：需要详细说明
         ↓
HITL Agent: AskUserQuestion({
    question: "🚨 你即将修改生产环境配置",
    options: [
        { label: "取消", description: "我不想修改生产环境" },
        { label: "继续", description: "我了解风险，继续修改" },
        { label: "查看差异", description: "先看看配置文件的变更" }
    ]
})
         ↓
用户: "查看差异"
         ↓
HITL Agent: 显示 diff → 再次询问是否继续
```

## Agent 设计

### 输入

```rust
pub struct ToolCallRequest {
    /// 工具名称
    pub tool_name: String,

    /// 工具参数
    pub args: serde_json::Value,

    /// 上下文信息
    pub context: OperationContext,
}

pub struct OperationContext {
    /// 用户历史操作（用于建立信任）
    pub user_history: Vec< PastOperation >,

    /// 项目元数据
    pub project_info: ProjectInfo,

    /// 当前任务描述
    pub current_task: String,
}

pub struct ProjectInfo {
    /// 是否有 git 仓库
    pub has_git: bool,

    /// 当前分支
    pub current_branch: Option<String>,

    /// 项目大小（文件数量）
    pub project_size: usize,

    /// 是否有备份
    pub has_backup: bool,
}
```

### 输出

```rust
pub enum HitlDecision {
    /// 直接执行，不需要确认
    ExecuteDirectly,

    /// 需要用户确认（是/否）
    RequireConfirmation {
        reason: String,
        warning_level: WarningLevel,
    },

    /// 需要用户选择（多选一）
    RequireChoice {
        question: String,
        options: Vec<UserChoice>,
        default: String,
    },

    /// 需要用户输入（文本）
    RequireInput {
        prompt: String,
        placeholder: Option<String>,
        validation: Option<InputValidation>,
    },

    /// 拒绝执行
    Reject {
        reason: String,
        suggestion: Option<String>,
    },
}
```

### Agent 实现

```rust
pub struct HitlAgent {
    client: AnthropicClient,
    config: HitlConfig,
    user_trust_score: f32,  // 信任分数 0.0 - 1.0
}

impl HitlAgent {
    /// 评估工具调用
    pub async fn evaluate_tool_call(
        &mut self,
        request: ToolCallRequest,
    ) -> Result<HitlDecision, HitlError> {
        // 1. 快速路径：已知安全的操作
        if let Some(decision) = self.quick_path(&request) {
            return Ok(decision);
        }

        // 2. 使用 AI 分析
        let analysis = self.analyze_with_ai(&request).await?;

        // 3. 根据分析结果决策
        Ok(self.make_decision(analysis))
    }

    /// 快速路径：已知的低风险操作
    fn quick_path(&self, request: &ToolCallRequest) -> Option<HitlDecision> {
        match request.tool_name.as_str() {
            "read_file" | "glob" | "grep_search" => {
                Some(HitlDecision::ExecuteDirectly)
            }
            "shell_execute" => {
                // 检查是否是安全的只读命令
                if let Some(cmd) = extract_command(&request.args) {
                    if is_safe_readonly_command(&cmd) {
                        return Some(HitlDecision::ExecuteDirectly);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// 使用 AI 分析操作
    async fn analyze_with_ai(
        &self,
        request: &ToolCallRequest,
    ) -> Result<OperationAnalysis, HitlError> {
        let prompt = format!(
            "分析以下工具调用是否需要人工确认：

工具: {}
参数: {:?}
上下文: {:?}

请评估：
1. 风险等级（无风险/低/中/高/极高）
2. 是否可逆
3. 影响范围
4. 是否需要用户确认
5. 如果需要，应该使用什么交互方式（确认/选择/输入）

返回 JSON 格式的分析结果。",
            request.tool_name, request.args, request.context
        );

        // 调用 AI
        let response = self.client
            .messages()
            .with_system("你是一个人机交互决策 Agent...")
            .with_user(&prompt)
            .await?;

        serde_json::from_str(&response.content)
    }

    /// 根据分析结果做出决策
    fn make_decision(&self, analysis: OperationAnalysis) -> HitlDecision {
        match (analysis.risk_level, analysis.needs_confirmation) {
            (RiskLevel::None, _) => HitlDecision::ExecuteDirectly,
            (RiskLevel::Low, false) => HitlDecision::ExecuteDirectly,
            (RiskLevel::High, true) => HitlDecision::RequireConfirmation {
                reason: analysis.reason,
                warning_level: WarningLevel::Danger,
            },
            // ... 更多情况
        }
    }
}
```

## 集成点

### 方案 1：在 Agent 层集成

```rust
// 在主 Agent 的 tool 调用前
async fn call_tool(&mut self, tool_name: &str, args: Value) -> Result<Value> {
    let request = ToolCallRequest {
        tool_name: tool_name.to_string(),
        args,
        context: self.build_context(),
    };

    // HITL 决策
    match self.hitl_agent.evaluate_tool_call(request).await? {
        HitlDecision::ExecuteDirectly => {
            // 直接执行工具
            self.tools.get(tool_name)?.call(args).await
        }
        HitlDecision::RequireConfirmation { reason, warning_level } => {
            // 请求用户确认
            let confirmed = self.ask_user_confirmation(&reason, warning_level).await?;
            if confirmed {
                self.tools.get(tool_name)?.call(args).await
            } else {
                Err(Error::Cancelled)
            }
        }
        HitlDecision::RequireChoice { question, options, default } => {
            // 请求用户选择
            let choice = self.ask_user_choice(&question, options, default).await?;
            // 根据选择执行...
        }
        HitlDecision::Reject { reason, suggestion } => {
            // 拒绝执行
            Err(Error::Rejected { reason, suggestion })
        }
    }
}
```

### 方案 2：作为 Tool Wrapper

```rust
pub struct HitlWrappedTool<T> {
    inner: T,
    hitl_agent: Arc<Mutex<HitlAgent>>,
}

impl<T: Tool> Tool for HitlWrappedTool<T> {
    async fn call(&self, args: T::Args) -> Result<T::Output, T::Error> {
        let request = ToolCallRequest {
            tool_name: T::NAME.to_string(),
            args: serde_json::to_value(args)?,
            context: self.hitl_agent.lock().await.get_context(),
        };

        let decision = self.hitl_agent
            .lock()
            .await
            .evaluate_tool_call(request)
            .await?;

        match decision {
            HitlDecision::ExecuteDirectly => {
                self.inner.call(args).await
            }
            HitlDecision::RequireConfirmation { reason, .. } => {
                if confirm(&reason)? {
                    self.inner.call(args).await
                } else {
                    Err(T::Error::cancelled())
                }
            }
            // ... 其他情况
        }
    }
}
```

## 配置

```toml
# .oxide/config.toml

[hitl]
# 全局策略
policy = "balanced"  # "strict" | "balanced" | "permissive"

# 信任度设置
[hitl.trust]
# 成功操作增加信任分数
increment = 0.01
# 拒绝操作减少信任分数
decrement = 0.05
# 初始信任分数
initial = 0.5

# 风险阈值
[hitl.thresholds]
# 信任分数高于此值时，低风险操作不需要确认
low_rust_auto_approve = 0.8
# 信任分数低于此值时，所有操作都需要确认
paranoid_mode = 0.2

# 特定工具规则
[hitl.tools]
delete_file = { always_confirm = true }
shell_execute = {
    safe_commands = ["git status", "git diff", "ls", "pwd"]
    dangerous_patterns = ["rm -rf", ":", "format", "kill"]
}
```

## 优势

1. **AI 决策** - 不是硬编码规则，而是智能判断
2. **上下文感知** - 考虑项目、历史、用户习惯
3. **渐进式信任** - 随时间建立信任，减少打扰
4. **灵活交互** - 确认、选择、输入，按需选择
5. **可解释** - AI 可以解释为什么需要确认
6. **持续学习** - 可以从用户反馈中学习

## 示例对话

```
用户: "删除 src/main.rs"

AI 助手: 🤔 (调用 HITL Agent)

HITL Agent (内部):
- 分析: 删除主程序文件
- 风险: 极高
- 影响: 项目无法编译
- 决策: 拒绝 + 提供替代方案

AI 助手: "我不能直接删除 src/main.rs，这会导致项目无法编译。

可能的操作：
1. 如果你想要重命名，我可以帮你重命名文件
2. 如果你想要替换内容，我可以帮你编辑
3. 如果你确定要删除，请明确说明原因

你想做什么？"

用户: "哦，我其实是想编辑它，不是删除"

AI 助手: "明白了！我可以帮你编辑 src/main.rs，你想修改什么内容？"
```

## 总结

这个方案的核心思想：
- **不是工具自己决定**，而是 **AI Agent 统一决策**
- **不是固定规则**，而是 **智能判断上下文**
- **不是简单确认**，而是 **灵活的人机对话**
