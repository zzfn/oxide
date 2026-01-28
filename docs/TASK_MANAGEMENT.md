# 任务管理系统

> 最后更新: 2026-01-28
> 状态: ✅ 已实现

## 📋 功能概述

任务管理系统是 Oxide 的核心功能之一，允许 Agent 创建、跟踪和管理结构化的任务列表。该系统支持任务依赖关系、状态管理和元数据存储，帮助用户和 Agent 更好地组织复杂的多步骤工作。

## 🎯 核心特性

### 1. 任务生命周期管理
- **创建任务**: 通过 `task_create` 工具创建新任务
- **更新任务**: 通过 `task_update` 工具更新任务状态和属性
- **查询任务**: 通过 `task_list` 和 `task_get` 工具查看任务
- **删除任务**: 将任务状态设置为 `deleted`

### 2. 任务状态流转
```
pending → in_progress → completed
                     → failed
                     → deleted
```

| 状态 | 说明 |
|------|------|
| `pending` | 等待中，任务已创建但未开始 |
| `in_progress` | 进行中，任务正在执行 |
| `completed` | 已完成，任务成功完成 |
| `failed` | 失败，任务执行失败 |
| `deleted` | 已删除，任务不再需要 |

### 3. 任务依赖关系
- **blocks**: 本任务阻塞的其他任务（本任务完成后这些任务才能开始）
- **blocked_by**: 阻塞本任务的其他任务（这些任务完成后本任务才能开始）
- **循环检测**: 系统自动检测并阻止循环依赖

### 4. 持久化存储
- 任务存储在 `.oxide/tasks/` 目录
- 每个任务保存为独立的 JSON 文件
- 支持跨会话持久化

## 🛠️ 工具 API

### TaskCreate - 创建任务

创建新的任务到任务列表中。

**参数**:
| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `subject` | string | ✅ | 任务标题（祈使句形式，如 "Fix authentication bug"） |
| `description` | string | ✅ | 任务详细描述 |
| `active_form` | string | ❌ | 进行中显示文本（如 "Fixing authentication bug"） |
| `metadata` | object | ❌ | 自定义元数据 |

**示例**:
```json
{
  "subject": "实现用户登录功能",
  "description": "添加用户名密码登录，包括表单验证和错误处理",
  "active_form": "实现用户登录功能中"
}
```

**返回**:
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "subject": "实现用户登录功能",
  "success": true,
  "message": "Task '实现用户登录功能' created successfully"
}
```

### TaskUpdate - 更新任务

更新任务的状态、属性或依赖关系。

**参数**:
| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `taskId` | string | ✅ | 任务 ID |
| `status` | string | ❌ | 新状态 (pending/in_progress/completed/failed/deleted) |
| `subject` | string | ❌ | 新标题 |
| `description` | string | ❌ | 新描述 |
| `activeForm` | string | ❌ | 进行中显示文本 |
| `owner` | string | ❌ | 任务所有者 |
| `addBlocks` | string[] | ❌ | 添加本任务阻塞的任务 ID |
| `addBlockedBy` | string[] | ❌ | 添加阻塞本任务的任务 ID |
| `metadata` | object | ❌ | 元数据更新（设置为 null 可删除键） |

**示例 - 开始任务**:
```json
{
  "taskId": "550e8400-e29b-41d4-a716-446655440000",
  "status": "in_progress"
}
```

**示例 - 设置依赖**:
```json
{
  "taskId": "task-2",
  "addBlockedBy": ["task-1"]
}
```

### TaskList - 列出任务

列出所有任务的摘要信息。

**参数**: 无

**返回**:
```json
{
  "tasks": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "subject": "实现用户登录功能",
      "status": "in_progress",
      "owner": null,
      "blocked_by": []
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "subject": "添加单元测试",
      "status": "pending",
      "owner": null,
      "blocked_by": ["550e8400-e29b-41d4-a716-446655440000"]
    }
  ],
  "total": 2,
  "success": true,
  "message": "Found 2 task(s)"
}
```

### TaskGet - 获取任务详情

获取单个任务的完整信息。

**参数**:
| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `taskId` | string | ✅ | 任务 ID |

**返回**:
```json
{
  "task": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "subject": "实现用户登录功能",
    "description": "添加用户名密码登录，包括表单验证和错误处理",
    "status": "in_progress",
    "owner": null,
    "active_form": "实现用户登录功能中",
    "blocks": ["550e8400-e29b-41d4-a716-446655440001"],
    "blocked_by": [],
    "metadata": {},
    "created_at": "2026-01-28T12:00:00Z",
    "updated_at": "2026-01-28T12:30:00Z"
  },
  "success": true,
  "message": "Task retrieved successfully"
}
```

## 🚀 使用方法

### CLI 命令

Oxide CLI 提供了 `/tasks` 命令来管理任务：

```bash
# 列出所有任务
/tasks

# 查看任务详情
/tasks show <task_id>

# 取消任务
/tasks cancel <task_id>
```

### Agent 自动使用

Agent 会在以下场景自动使用任务管理工具：

1. **复杂多步骤任务**: 当任务需要 3 个或更多步骤时
2. **用户明确请求**: 当用户要求创建任务列表时
3. **多任务输入**: 当用户提供多个任务（编号或逗号分隔）时

**示例对话**:
```
用户: 帮我完成以下工作：1. 修复登录bug 2. 添加单元测试 3. 更新文档

Agent: [调用 task_create 创建三个任务]
       [调用 task_update 设置依赖关系]
       [开始执行第一个任务...]
```

### 直接查看任务文件

任务以 JSON 格式存储在 `.oxide/tasks/` 目录：

```bash
# 查看任务目录
ls -la .oxide/tasks/

# 查看任务内容
cat .oxide/tasks/<task_id>.json | jq .
```

## 📁 数据结构

### Task 结构

```rust
pub struct Task {
    pub id: TaskId,                    // 任务 ID (UUID)
    pub name: String,                  // 内部名称
    pub subject: String,               // 显示标题
    pub description: String,           // 详细描述
    pub prompt: String,                // 提示词（用于 Agent 任务）
    pub active_form: Option<String>,   // 进行中显示文本
    pub status: TaskStatus,            // 任务状态
    pub agent_type: AgentType,         // Agent 类型
    pub owner: Option<String>,         // 任务所有者
    pub blocks: Vec<TaskId>,           // 阻塞的任务
    pub blocked_by: Vec<TaskId>,       // 被阻塞的任务
    pub metadata: HashMap<String, Value>, // 自定义元数据
    pub created_at: DateTime<Utc>,     // 创建时间
    pub updated_at: DateTime<Utc>,     // 更新时间
    pub started_at: Option<DateTime<Utc>>,   // 开始时间
    pub completed_at: Option<DateTime<Utc>>, // 完成时间
    pub output_file: Option<PathBuf>,  // 输出文件路径
    pub error: Option<String>,         // 错误信息
}
```

### 任务 JSON 示例

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "实现用户登录功能",
  "subject": "实现用户登录功能",
  "description": "添加用户名密码登录，包括表单验证和错误处理",
  "prompt": "",
  "active_form": "实现用户登录功能中",
  "status": "in_progress",
  "agent_type": "Main",
  "owner": null,
  "blocks": [],
  "blocked_by": [],
  "metadata": {},
  "created_at": "2026-01-28T12:00:00Z",
  "updated_at": "2026-01-28T12:30:00Z",
  "started_at": "2026-01-28T12:30:00Z",
  "completed_at": null,
  "output_file": null,
  "error": null
}
```

## 🔧 实现细节

### 文件结构

```
src/
├── task/
│   └── manager.rs          # TaskManager 和 Task 结构体
└── tools/
    ├── task_create.rs      # TaskCreate 工具
    ├── task_update.rs      # TaskUpdate 工具
    ├── task_list.rs        # TaskList 工具
    └── task_get.rs         # TaskGet 工具
```

### 全局单例

TaskManager 使用全局单例模式，确保所有工具共享同一个任务存储：

```rust
use once_cell::sync::Lazy;

static TASK_MANAGER: Lazy<TaskManager> = Lazy::new(|| {
    let storage_dir = PathBuf::from(".oxide/tasks");
    TaskManager::new(storage_dir).expect("无法初始化任务管理器")
});

pub fn get_task_manager() -> &'static TaskManager {
    &TASK_MANAGER
}
```

### 循环依赖检测

系统使用 DFS 算法检测循环依赖：

```rust
fn would_create_cycle(&self, from_id: &TaskId, to_id: &TaskId) -> Result<bool> {
    // 如果 from_id 已经（直接或间接）依赖于 to_id，
    // 那么添加 to_id -> from_id 的依赖会导致循环
    let mut visited = HashSet::new();
    let mut path = HashSet::new();
    self.detect_cycle_from(from_id, to_id, &mut visited, &mut path)
}
```

## 📊 与 Claude Code 对比

| 功能 | Claude Code | Oxide |
|------|-------------|-------|
| TaskCreate | ✅ | ✅ |
| TaskUpdate | ✅ | ✅ |
| TaskList | ✅ | ✅ |
| TaskGet | ✅ | ✅ |
| 任务依赖 | ✅ | ✅ |
| 循环检测 | ✅ | ✅ |
| 持久化存储 | ✅ | ✅ |
| 元数据支持 | ✅ | ✅ |

## 🧪 测试

运行任务管理相关测试：

```bash
cargo test task
```

测试覆盖：
- 任务创建和状态转换
- 依赖关系管理
- 循环依赖检测
- 任务持久化
- 工具参数序列化/反序列化
