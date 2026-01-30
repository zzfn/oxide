# 权限系统

## 概述

Oxide 实现了两层权限控制：

1. **配置级权限** (`~/.oxide/config.toml`) - 静态配置哪些工具可以使用
2. **计划模式权限声明** - 代理在计划中声明需要的权限

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
}

impl PermissionManager {
    pub async fn is_allowed(&self, tool_name: &str) -> bool;
    pub async fn update_config(&self, config: PermissionsConfig);
}
```

### ToolWrapper 集成

`ToolWrapper` 在执行工具前检查权限：

```rust
let wrapper = ToolWrapper::new(tool)
    .with_permission_manager(permission_manager);
```

如果工具被禁止，会打印警告信息但不会阻止执行（当前实现）。

## 使用方式

### 在代理中启用权限检查

```rust
use oxide_core::config::Config;
use oxide_tools::PermissionManager;

let config = Config::load()?;
let permission_manager = PermissionManager::new(config.permissions);

let tools = OxideToolSetBuilder::new(working_dir)
    .permission_manager(permission_manager)
    .build_boxed();
```

## 限制

当前实现的限制：

- 权限检查只打印警告，不强制阻止执行
- 计划模式的权限声明不会自动应用到配置
- 没有运行时权限请求机制

## 未来改进

- [ ] 强制执行权限检查（返回错误而非警告）
- [ ] 实现运行时权限请求对话框
- [ ] 支持细粒度权限（如 Bash 只允许特定命令）
- [ ] 权限审计日志
- [ ] 临时权限提升机制
