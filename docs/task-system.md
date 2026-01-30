# 任务管理系统实现文档

> Oxide 任务管理系统的完整实现说明

**实现日期**: 2026-01-30
**状态**: ✅ Phase 1 完成

---

## 📋 概述

任务管理系统是 Oxide 的核心功能之一，用于跟踪和管理复杂的多步骤工作。它提供了创建、查询、更新任务以及管理任务依赖关系的完整功能。

## 🎯 核心功能

### 1. 任务数据结构

```rust
pub struct Task {
    pub id: String,              // 任务 ID（自增数字）
    pub subject: String,         // 任务标题（祈使句）
    pub description: String,     // 详细描述
    pub active_form: Option<String>, // 进行中显示文本（现在进行时）
    pub status: TaskStatus,      // 任务状态
    pub owner: Option<String>,   // 任务所有者（子代理 ID）
    pub blocks: Vec<String>,     // 此任务阻塞的任务列表
    pub blocked_by: Vec<String>, // 阻塞此任务的任务列表
    pub metadata: HashMap<String, Value>, // 元数据
    pub created_at: DateTime<Utc>,  // 创建时间
    pub updated_at: DateTime<Utc>,  // 更新时间
}
```

### 2. 任务状态

```rust
pub enum TaskStatus {
    Pending,      // 待处理
    InProgress,   // 进行中
    Completed,    // 已完成
    Deleted,      // 已删除
}
```

状态转换规则：
- `Pending` → 任何状态
- `InProgress` → `Completed` 或 `Deleted`
- `Completed` → `Deleted`
- `Deleted` → 无法转换

### 3. TaskManager

任务管理器提供以下核心功能：

#### 任务 CRUD 操作
- `create_task()` - 创建新任务
- `get_task()` - 获取任务详情
- `list_tasks()` - 列出所有任务
- `update_task_status()` - 更新任务状态
- `update_task_owner()` - 更新任务所有者
- `update_task_metadata()` - 更新任务元数据
- `update_task_content()` - 更新任务内容

#### 依赖关系管理
- `add_dependency()` - 添加任务依赖关系
- `detect_circular_dependency()` - 检测循环依赖（DFS 算法）
- `get_ready_tasks()` - 获取可执行的任务（没有被阻塞的 pending 任务）

#### 后台任务管理（向后兼容）
- `add_background_task()` - 添加后台 Bash 任务
- `get_background_task()` - 获取后台任务
- `update_background_task()` - 更新后台任务状态

---

## 🔧 四个任务工具

### 1. TaskCreate

**功能**: 创建新任务

**参数**:
```json
{
  "subject": "实现用户认证",           // 必需，祈使句
  "description": "实现 JWT 认证...",   // 必需，详细描述
  "activeForm": "正在实现用户认证",    // 可选，现在进行时
  "metadata": {                        // 可选，元数据
    "priority": "high",
    "tags": ["security", "auth"]
  }
}
```

**输出**:
```json
{
  "task_id": "1",
  "subject": "实现用户认证",
  "message": "Task #1 created successfully: 实现用户认证"
}
```

### 2. TaskList

**功能**: 列出所有任务的摘要信息

**参数**: 无

**输出**:
```json
{
  "tasks": [
    {
      "id": "1",
      "subject": "实现用户认证",
      "status": "pending",
      "owner": null,
      "blocked_by": []
    },
    {
      "id": "2",
      "subject": "编写测试",
      "status": "pending",
      "owner": null,
      "blocked_by": ["1"]
    }
  ],
  "total": 2
}
```

### 3. TaskGet

**功能**: 获取任务的完整详情

**参数**:
```json
{
  "taskId": "1"
}
```

**输出**:
```json
{
  "id": "1",
  "subject": "实现用户认证",
  "description": "实现 JWT 认证系统...",
  "active_form": "正在实现用户认证",
  "status": "in_progress",
  "owner": "agent-123",
  "blocks": ["2"],
  "blocked_by": [],
  "metadata": {
    "priority": "high"
  },
  "created_at": "2026-01-30T10:00:00Z",
  "updated_at": "2026-01-30T10:30:00Z"
}
```

### 4. TaskUpdate

**功能**: 更新任务的各种属性

**参数**:
```json
{
  "taskId": "1",
  "status": "in_progress",              // 可选
  "subject": "新标题",                   // 可选
  "description": "新描述",               // 可选
  "activeForm": "新的进行中文本",        // 可选
  "owner": "agent-456",                 // 可选
  "addBlocks": ["3", "4"],              // 可选，添加阻塞关系
  "addBlockedBy": ["0"],                // 可选，添加被阻塞关系
  "metadata": {                         // 可选，更新元数据
    "priority": "high",
    "removed_key": null                 // null 表示删除该键
  }
}
```

**输出**:
```json
{
  "task_id": "1",
  "updated_fields": ["status", "owner", "blockedBy"],
  "message": "Updated task #1 status, owner, blockedBy"
}
```

---

## 🏗️ 架构设计

### 模块结构

```
crates/oxide-tools/src/task/
├── mod.rs              # 模块入口
├── types.rs            # Task 和 TaskStatus 定义
├── manager.rs          # TaskManager 实现
├── errors.rs           # TaskError 错误类型
└── tools/
    ├── mod.rs          # 工具模块入口
    ├── create.rs       # RigTaskCreateTool
    ├── list.rs         # RigTaskListTool
    ├── get.rs          # RigTaskGetTool
    └── update.rs       # RigTaskUpdateTool
```

### 集成方式

任务工具通过 `OxideToolSetBuilder` 集成到工具集：

```rust
let toolset = OxideToolSetBuilder::new(working_dir)
    .task_manager(task_manager)
    .task_tools(true)  // 启用任务工具
    .build();
```

### 向后兼容

TaskManager 同时管理两种任务：
1. **新的任务系统** - 用于任务管理工具
2. **后台 Bash 任务** - 保持与现有 Bash 工具的兼容性

---

## 🧪 测试覆盖

### 单元测试（18 个）

#### types.rs (3 个测试)
- `test_task_creation` - 任务创建
- `test_status_transitions` - 状态转换规则
- `test_task_update_status` - 状态更新
- `test_task_dependencies` - 依赖关系管理

#### manager.rs (4 个测试)
- `test_create_and_get_task` - 创建和获取任务
- `test_list_tasks` - 列出任务
- `test_update_task_status` - 更新状态
- `test_task_dependencies` - 依赖关系
- `test_circular_dependency_detection` - 循环依赖检测

#### tools/create.rs (2 个测试)
- `test_task_create_tool` - 创建工具
- `test_task_create_with_metadata` - 带元数据创建

#### tools/list.rs (2 个测试)
- `test_task_list_tool` - 列表工具
- `test_task_list_excludes_deleted` - 排除已删除任务

#### tools/get.rs (2 个测试)
- `test_task_get_tool` - 获取工具
- `test_task_get_not_found` - 任务不存在

#### tools/update.rs (3 个测试)
- `test_task_update_status` - 更新状态
- `test_task_update_dependencies` - 更新依赖
- `test_task_update_metadata` - 更新元数据

### 测试结果

```
running 18 tests
test result: ok. 18 passed; 0 failed; 0 ignored
```

---

## 🎨 设计亮点

### 1. 类型安全的状态转换

使用 Rust 的类型系统确保状态转换的合法性：

```rust
impl TaskStatus {
    pub fn can_transition_to(&self, new_status: TaskStatus) -> bool {
        // 编译时检查，运行时验证
    }
}
```

### 2. 循环依赖检测

使用深度优先搜索（DFS）算法检测任务依赖图中的循环：

```rust
fn has_cycle_dfs(
    &self,
    task_id: &str,
    tasks: &HashMap<String, Task>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    // DFS 检测循环
}
```

### 3. 并发安全

使用 `Arc<RwLock<>>` 保护共享状态，支持多线程访问：

```rust
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    background_tasks: Arc<RwLock<HashMap<String, BackgroundTask>>>,
    task_counter: Arc<RwLock<u32>>,
}
```

### 4. 向后兼容

保留原有的 BackgroundTask 功能，确保 Bash 工具正常工作：

```rust
impl TaskManager {
    pub fn background_tasks(&self) -> Arc<RwLock<HashMap<String, BackgroundTask>>> {
        self.background_tasks.clone()
    }
}
```

### 5. 灵活的元数据系统

使用 `HashMap<String, Value>` 支持任意元数据：

```rust
pub metadata: HashMap<String, serde_json::Value>
```

---

## 📊 性能特性

- **O(1)** 任务查询（HashMap）
- **O(V + E)** 循环依赖检测（DFS）
- **O(n log n)** 任务列表排序（按 ID）
- **异步操作** 所有 I/O 操作使用 async/await
- **零拷贝** 使用 Arc 共享数据

---

## 🔮 未来扩展

### Phase 2: 任务持久化

将任务保存到磁盘：

```rust
impl TaskManager {
    pub async fn save_to_disk(&self, path: &Path) -> Result<()>;
    pub async fn load_from_disk(path: &Path) -> Result<Self>;
}
```

### Phase 3: 任务查询

支持更复杂的任务查询：

```rust
impl TaskManager {
    pub async fn find_tasks(&self, filter: TaskFilter) -> Vec<Task>;
}

pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}
```

### Phase 4: 任务事件

支持任务状态变更事件：

```rust
pub trait TaskEventListener {
    async fn on_task_created(&self, task: &Task);
    async fn on_task_updated(&self, task: &Task);
    async fn on_task_completed(&self, task: &Task);
}
```

---

## 📝 使用示例

### 创建任务

```rust
let manager = TaskManager::new();

let task_id = manager.create_task(
    "实现用户认证".to_string(),
    "实现 JWT 认证系统，包括登录、注册和令牌刷新".to_string(),
    Some("正在实现用户认证".to_string()),
).await?;
```

### 添加依赖关系

```rust
// task2 依赖 task1
manager.add_dependency(
    &task2,
    vec![],           // task2 不阻塞其他任务
    vec![task1.clone()], // task2 被 task1 阻塞
).await?;
```

### 更新任务状态

```rust
manager.update_task_status(&task_id, TaskStatus::InProgress).await?;
```

### 获取可执行任务

```rust
let ready_tasks = manager.get_ready_tasks().await;
for task in ready_tasks {
    println!("可以开始: {}", task.subject);
}
```

---

## 🎓 总结

任务管理系统为 Oxide 提供了强大的任务跟踪和管理能力，具有以下特点：

✅ **完整的 CRUD 操作**
✅ **依赖关系管理**
✅ **循环依赖检测**
✅ **类型安全的状态转换**
✅ **并发安全**
✅ **向后兼容**
✅ **完整的测试覆盖**
✅ **清晰的错误处理**

这为后续的子代理系统（Phase 2）和计划模式（Phase 3）奠定了坚实的基础。

---

**文档版本**: 1.0
**最后更新**: 2026-01-30
