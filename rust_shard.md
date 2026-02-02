# 用 Rust 重写 Claude Code

## 为什么要做这个项目

我想搞懂 Claude Code 到底是怎么工作的——不是用，而是自己亲手实现一遍。只有踩过坑才知道哪些设计是真的重要。

**想搞清楚的东西**：

- **Agent 怎么循环**：LLM 调用工具 → 拿到结果 → 再调 LLM，这个闭环是怎么跑起来的
- **工具系统**：为什么 Claude Code 的核心是那几个读写文件的函数，而不是聊天本身
- **权限怎么处理**：让 AI 自动执行命令很爽，但怎么防止它 `rm -rf /`
- **任务管理**：复杂需求怎么拆成可执行的小任务，还能跟踪依赖关系
- **流式渲染**：LLM 吐字的时候界面怎么不卡、不闪烁
- **上下文压缩**：对话长了怎么丢历史记录，哪些该留哪些该扔

**为什么选 Rust**：

1. **验证 AI 辅助开发到底靠不靠谱**：我根本不会 Rust，从零开始用 AI 写，看能不能搞出一个能用的系统
2. **CLI 工具用 Rust 确实香**：单二进制、启动快、内存占用低

## 系统架构

### Agent 核心循环

```
用户输入
    │
    ▼
┌─────────────────────────────────────┐
│  Orchestrator                        │
│  1. 调用 LLM                         │
│  2. 接收流式响应                     │
│  3. 解析 Tool Call                   │
└─────────────────────────────────────┘
    │
    ▼
    有 Tool Call？
    │
    ├─ NO ──▶ 输出响应 ──▶ 结束
    │
    └─ YES ──▶ Toolbox（权限确认 → 执行 → 收集结果）
                │
                ▼
            注入上下文 ──▶ 回到 Orchestrator
```

**关键代码**：

```rust
while !done {
    response = llm.call(messages + tools);
    if response.has_tool_calls() {
        results = execute_tools(response.tool_calls);
        messages.push(results);  // 执行结果喂给下一轮
    } else {
        done = true;
    }
}
```

### 三大核心机制

#### 1. Tools：从"顾问"到"执行者"

普通 Chatbot 只会给建议，Claude Code 能直接动手：

| 普通 Chatbot        | Claude Code                         |
| ------------------- | ----------------------------------- |
| "你应该修改 A 文件" | `Read` → `Edit` → `Bash` 直接跑测试 |
| 上下文靠对话历史    | 上下文靠**工具执行结果**动态获取    |

**6 个核心工具**：`Read` / `Write` / `Edit` / `Bash` / `Grep` / `Glob`

Bash 尤其关键——它让 Agent 能执行任意命令，真正触达外部环境。

#### 2. 权限控制：Human-in-the-loop

给了 Agent 工具，就给了它搞破坏的能力。必须分级别控制：

| 级别    | 操作                   | 处理方式             |
| ------- | ---------------------- | -------------------- |
| 🟢 安全 | `Read`、`Grep`、`Glob` | 直接执行             |
| 🟡 敏感 | `Edit`                 | 首次确认，会话内记忆 |
| 🔴 危险 | `Write`、`Bash`        | 每次确认             |

为什么要打断它？

- LLM 会胡说，可能"觉得"该删某个重要文件
- Agent 不知道 `.env` 里有密码，对它来说只是文本
- 全部自动很爽，但出一次事故就再也不敢用了

#### 3. 任务管理系统

复杂任务需要拆分和跟踪。任务数据结构：

```rust
pub struct Task {
    pub id: String,
    pub subject: String,        // 标题
    pub status: TaskStatus,     // Pending / InProgress / Completed
    pub blocks: Vec<String>,    // 阻塞哪些任务
    pub blocked_by: Vec<String>, // 被哪些任务阻塞
}
```

任务依赖用 DFS 检测循环，Agent 按依赖顺序执行，不再是"想到哪做到哪"。

## 技术栈

| 类别 | 依赖                     | 用途               |
| ---- | ------------------------ | ------------------ |
| 异步 | `tokio`                  | 高性能 I/O         |
| LLM  | `rig-core`               | 统一 LLM 抽象      |
| UI   | `crossterm`, `reedline`  | 终端控制、命令编辑 |
| 搜索 | `grep-searcher`, `regex` | 代码搜索           |
| 错误 | `anyhow`, `thiserror`    | 错误处理           |

## 踩过的坑

**流式渲染闪烁**

- 问题：LLM 吐字 + Tool 日志混在一起，终端疯狂闪烁
- 解决：`StreamState` 状态机管理渲染优先级，用 `crossterm` 局部重绘

**权限控制**

- 三层防线：配置文件白名单 → `PermissionManager` 运行时拦截 → 会话记忆

**异步任务编排**

- 问题：同时处理用户输入（阻塞）、后台 Tool、LLM 流式响应
- 解决：`tokio` Actor 模式，拆成三个独立任务，Channel 通信

**Edit 工具太糙（待完善）**

- 现状：字符串 `replace`，容易改错位置
- 计划：加 diff 预览、LSP 同步、AST 编辑

## 做到哪了

**已能用**：

- ✅ 基础设施、LLM 集成、核心工具、CLI 界面
- 🚧 任务管理（做完了）、子代理（还没）

**还没做**：

- ⏳ 子代理系统、Hooks、自动摘要、Git 集成、MCP、Skills

## 学到了什么

- **不会 Rust 也能写 Rust**：边做边问 AI，比先看三个月书快多了
- **类型系统真的香**：编译器逼你处理所有分支，很多 Bug 在编译期就解决了
- **Agent 不是魔法**：核心逻辑不复杂，复杂的是把各种边界情况处理好
- **工具比对话重要**：Claude Code 的核心不是聊天，而是那 6 个能操作文件系统的函数
