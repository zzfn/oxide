# 权限系统

## 概述

Oxide 实现了三层权限控制：

1. **配置级权限** (`~/.oxide/config.toml`) - 静态配置哪些工具可以使用
2. **运行时确认** - 危险工具首次执行时请求用户确认
3. **计划模式权限声明** - 代理在计划中声明需要的权限

## 危险工具

以下工具被标记为"危险工具"，首次执行时需要用户确认：

- `Edit` - 编辑文件
- `Write` - 写入文件
- `Bash` - 执行命令

## 运行时确认

当执行危险工具时，会显示选择菜单：

```
工具 'Edit' 需要权限确认
> 允许本次
  始终允许（本次会话）
  始终允许（记住选择）
  拒绝
```

### 选项说明

| 选项 | 行为 |
|------|------|
| 允许本次 | 执行工具，下次仍会询问 |
| 始终允许（本次会话） | 执行工具，本次会话内不再询问该工具 |
| 始终允许（记住选择） | 执行工具，持久化记住选择（TODO） |
| 拒绝 | 不执行工具，返回错误给 AI |

### 确认结果

```rust
pub enum ConfirmationResult {
    Allow,        // 仅允许本次
    Deny,         // 拒绝
    AllowSession, // 本次会话内始终允许
    AllowAlways,  // 持久化允许
}
```

## 配置级权限

### 配置文件

在 `~/.oxide/config.toml` 中配置：

```toml
[permissions]
# 允许列表（为空表示允许所有）
allow = ["Read", "Glob", "Grep"]

# 禁止列表（优先级高于 allow）
deny = ["Bash", "Write", "Edit"]
```

### 规则

- `deny` 列表优先级最高，在此列表中的工具一律禁止
- 如果 `allow` 列表为空，默认允许所有工具（除了 deny 中的）
- 如果 `allow` 列表不为空，只允许列表中的工具

### 示例

**只读模式**（只允许查看代码）：
```toml
[permissions]
allow = ["Read", "Glob", "Grep"]
deny = []
```

**禁止命令执行**：
```toml
[permissions]
allow = []
deny = ["Bash"]
```

**完全开放**（默认）：
```toml
[permissions]
allow = []
deny = []
```

## 计划模式权限声明

在计划模式中，代理可以声明实现计划所需的权限：

```rust
ExitPlanMode {
    plan_content: "...",
    allowed_prompts: [
        { tool: "Bash", prompt: "run tests" },
        { tool: "Bash", prompt: "install dependencies" }
    ]
}
```

这些权限会保存在计划文件中，供用户审查。

## 实现细节

### PermissionManager

```rust
pub struct PermissionManager {
    config: Arc<RwLock<PermissionsConfig>>,
    approved_tools: Arc<RwLock<HashSet<String>>>,
    require_confirmation: bool,
    confirmation_callback: Option<ConfirmationCallback>,
}

impl PermissionManager {
    pub async fn is_allowed(&self, tool_name: &str) -> bool;
    pub async fn requires_confirmation(&self, tool_name: &str) -> bool;
    pub async fn request_confirmation(&self, tool_name: &str) -> Result<ConfirmationResult, ()>;
    pub async fn approve_tool(&self, tool_name: &str);
}
```

### ConfirmationCallback

```rust
pub type ConfirmationCallback = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = ConfirmationResult> + Send>> + Send + Sync,
>;
```

### ToolWrapper 集成

`ToolWrapper` 在执行工具前检查权限：

```rust
let wrapper = ToolWrapper::new(tool)
    .with_permission_manager(permission_manager);
```

权限检查流程：
1. 检查工具是否被配置禁止 → 返回 `ToolDenied` 错误
2. 检查是否需要用户确认 → 调用确认回调
3. 用户拒绝 → 返回 `UserRejected` 错误
4. 用户同意 → 执行工具

### 错误类型

```rust
pub enum PermissionError {
    ToolDenied(String),           // 被配置禁止
    UserRejected(String),         // 用户拒绝
    NoConfirmationHandler(String), // 未配置确认回调
}
```

## 使用方式

### 在代理中启用权限检查

```rust
use oxide_core::config::PermissionsConfig;
use oxide_tools::{ConfirmationCallback, ConfirmationResult, PermissionManager};

// 创建确认回调
let callback: ConfirmationCallback = Arc::new(|tool_name| {
    Box::pin(async move {
        // 显示确认对话框
        // 返回用户选择
        ConfirmationResult::AllowSession
    })
});

// 创建权限管理器
let pm = PermissionManager::new(PermissionsConfig::default())
    .with_confirmation_callback(callback);

// 构建工具集
let tools = OxideToolSetBuilder::new(working_dir)
    .permission_manager(pm)
    .build_boxed();
```

### 禁用确认（用于测试/自动化）

```rust
let pm = PermissionManager::new(config).without_confirmation();
```

## 已完成功能

- [x] 配置级权限检查（allow/deny 列表）
- [x] 运行时权限确认对话框
- [x] 会话级权限记忆
- [x] 权限错误返回给 AI

## 未来改进

- [ ] 持久化"始终允许"选择到配置文件
- [ ] 支持细粒度权限（如 Bash 只允许特定命令）
- [ ] 权限审计日志
- [ ] 临时权限提升机制
