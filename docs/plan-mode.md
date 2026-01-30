# 计划模式 (Plan Mode)

## 概述

计划模式是 Oxide 的一个重要功能，允许代理在执行代码修改之前先探索代码库并设计实现方案。这有助于：

- 在动手之前充分理解代码结构
- 设计更合理的实现方案
- 避免不必要的代码修改
- 让用户审查和批准实现计划

## 工作流程

### 1. 进入计划模式

使用 `EnterPlanMode` 工具进入计划模式：

```rust
// 代理调用
EnterPlanMode {}
```

进入计划模式后：
- 代理可以使用所有只读工具（Read, Glob, Grep 等）
- 代理应该探索代码库，理解现有架构
- 代理不应该执行任何代码修改操作

### 2. 探索和设计

在计划模式下，代理应该：

1. **探索代码库**
   - 使用 Glob 查找相关文件
   - 使用 Grep 搜索关键代码
   - 使用 Read 阅读重要文件

2. **理解架构**
   - 识别现有模式和约定
   - 理解模块间的依赖关系
   - 找出需要修改的关键文件

3. **设计方案**
   - 列出需要创建/修改的文件
   - 说明每个文件的修改内容
   - 考虑架构权衡和替代方案

### 3. 退出计划模式

使用 `ExitPlanMode` 工具保存计划并退出：

```rust
ExitPlanMode {
    plan_content: "# 实现计划\n\n## 文件修改清单\n...",
    plan_title: "添加用户认证功能",
    allowed_prompts: [
        { tool: "Bash", prompt: "run tests" },
        { tool: "Bash", prompt: "install dependencies" }
    ]
}
```

计划将保存到 `~/.oxide/plans/{plan_id}.json`。

#### 权限请求系统

`allowedPrompts` 参数允许代理声明实现计划所需的权限：

- **tool**: 工具名称（目前支持 "Bash"）
- **prompt**: 权限描述（如 "run tests", "install dependencies"）

这些权限将保存在计划文件中，供用户审查和批准。

## 计划文件格式

计划内容应使用 Markdown 格式，建议包含：

```markdown
# 实现计划标题

## 概述
简要说明要实现的功能和目标。

## 架构分析
- 现有代码结构
- 相关模块和依赖
- 需要遵循的模式

## 实现步骤

### 1. 创建新文件
- `src/auth/mod.rs` - 认证模块入口
- `src/auth/jwt.rs` - JWT 令牌处理

### 2. 修改现有文件
- `src/main.rs:45` - 添加认证中间件
- `src/config.rs:20` - 添加认证配置

### 3. 依赖添加
```toml
jsonwebtoken = "8.0"
```

## 测试计划
- 单元测试：JWT 令牌生成和验证
- 集成测试：完整的认证流程

## 注意事项
- 确保向后兼容
- 处理边界情况
```

## 实现细节

### PlanManager

`PlanManager` 管理计划模式的状态：

```rust
pub struct PlanManager {
    state: Arc<RwLock<PlanState>>,
}

impl PlanManager {
    pub async fn enter_plan_mode(&self, title: Option<String>) -> Uuid;
    pub async fn exit_plan_mode(&self) -> Option<Uuid>;
    pub async fn is_plan_mode(&self) -> bool;
    pub async fn current_plan_id(&self) -> Option<Uuid>;
}
```

### 工具集成

计划模式工具已集成到 `OxideToolSetBuilder`：

```rust
let tools = OxideToolSetBuilder::new(working_dir)
    .plan_tools(true)  // 启用计划模式工具
    .build();
```

## 使用示例

### 示例 1：添加新功能

```
用户: 我想添加用户认证功能，但先不要实现，帮我制定一个计划。

代理:
1. [调用 EnterPlanMode]
2. [使用 Glob 查找现有的认证相关代码]
3. [使用 Read 阅读配置文件和主入口]
4. [使用 Grep 搜索用户相关的代码]
5. [设计实现方案]
6. [调用 ExitPlanMode 保存计划]

计划已保存到 ~/.oxide/plans/{uuid}.json
```

### 示例 2：重构代码

```
用户: 帮我分析一下如何重构 auth 模块，先做个计划。

代理:
1. [调用 EnterPlanMode]
2. [阅读 auth 模块的所有文件]
3. [分析模块间的依赖关系]
4. [识别可以改进的地方]
5. [设计重构方案]
6. [调用 ExitPlanMode]
```

## 最佳实践

1. **充分探索**：在设计方案前，确保理解了相关的所有代码
2. **遵循约定**：识别并遵循项目现有的代码风格和架构模式
3. **考虑影响**：评估修改对其他模块的影响
4. **详细说明**：计划应该足够详细，让其他开发者能够理解和执行
5. **包含测试**：说明如何测试新功能或修改

## 限制

当前实现的限制：

- 计划模式不会自动限制工具使用（需要代理自觉遵守）
- 权限系统仅支持声明，不强制执行
- 计划文件目前只是简单的 JSON 存储

## 未来改进

- [ ] 自动限制计划模式下的工具使用（禁用 Write, Edit 等修改工具）
- [ ] 实现权限强制执行机制
- [ ] 支持计划的版本管理和比较
- [ ] 提供计划模板
- [ ] 集成到 CLI 命令（如 `/plan`）
- [ ] 扩展权限系统支持更多工具类型
