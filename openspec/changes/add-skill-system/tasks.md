# Tasks: Skill System Implementation

## Phase 1: Core Infrastructure
- [x] 创建 `src/skill/` 模块目录结构
- [x] 定义 `Skill` 结构体（name, description, template, args）
- [x] 定义 `SkillLoader` 用于扫描和加载技能文件
- [x] 定义 `SkillExecutor` 用于渲染和执行技能
- [x] 添加 Front matter 解析（使用 `serde_yaml`）

## Phase 2: File System Integration
- [x] 实现 `~/.oxide/skills/` 全局目录扫描
- [x] 实现 `.oxide/skills/` 本地目录扫描（优先级更高）
- [x] 实现技能去重逻辑（本地覆盖全局）
- [x] 添加技能缓存（启动时加载，避免重复 I/O）
- [x] 处理文件系统错误（权限、格式错误等）

## Phase 3: Template Engine
- [x] 集成模板引擎（如 `tera` 或简单的字符串替换）
- [x] 实现变量替换（`{{var}}` → value）
- [x] 实现参数验证（required args 检查）
- [x] 添加模板渲染错误处理

## Phase 4: CLI Integration
- [x] 扩展 `OxideHelper::commands` 包含动态 skills
- [x] 更新命令补全逻辑支持 skills
- [x] 实现通用 skill 命令处理器
- [x] 更新 `/help` 显示可用 skills
- [x] 实现 `/skills [list|show]` 子命令

## Phase 5: Built-in Example Skills
- [x] 创建 `/commit` skill（git commit helper）
- [x] 创建 `/compact` skill（会话压缩）
- [x] 创建 `/review` skill（code review 基础）
- [x] 添加示例技能到项目文档

## Phase 6: Testing & Validation
- [ ] 单元测试：Front matter 解析
- [ ] 单元测试：变量替换逻辑
- [ ] 集成测试：技能加载和执行
- [ ] 手动测试：用户交互流程
- [ ] 错误处理测试：无效格式、缺失参数等

## Phase 7: Documentation
- [x] 更新 README.md 添加 Skill 使用说明
- [ ] 创建 `docs/skills.md` 详细文档
- [x] 添加技能编写最佳实践
- [x] 更新 USAGE.md 示例

## Task Details

### 1.1 Core Data Structures
**File**: `src/skill/mod.rs`
```rust
pub struct Skill {
    pub name: String,
    pub description: String,
    pub template: String,
    pub args: Vec<SkillArg>,
    pub source: SkillSource,
}

pub struct SkillArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

pub enum SkillSource {
    BuiltIn,
    Global,
    Local,
}
```

### 2.1 File Scanning
**Priority**: High
**Dependencies**: None
**Estimated Effort**: 2 hours

### 3.1 Template Engine Integration
**Options**:
- `tera` - 功能强大，但可能过重
- `handlebars` - 适度复杂
- 自定义简单替换 - 轻量，但功能有限

**Recommendation**: 从简单字符串替换开始，未来可升级到 `tera`

### 4.1 CLI Command Routing
**Changes to**: `src/cli/command.rs`
```rust
_ if input.starts_with("/skills") => {
    self.handle_skills_command(input)?;
}
// Dynamic skill routing
_ if let Some(skill_name) = self.extract_skill_name(input) => {
    self.execute_skill(skill_name, input)?;
}
```

## Validation Criteria
- [ ] 可以从 `~/.oxide/skills/` 加载自定义 skill
- [ ] 可以从 `.oxide/skills/` 加载项目 skill
- [ ] `/skills list` 显示所有可用技能
- [ ] `/commit -m "message"` 正确执行
- [ ] 参数传递工作正常
- [ ] 模板变量替换正确
- [ ] 错误处理友好（缺失参数、无效 skill 等）

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| 模板引擎性能问题 | 中 | 使用简单替换，避免复杂模板引擎 |
| Skill 文件格式错误 | 高 | 添加验证和友好的错误提示 |
| 与现有命令冲突 | 中 | skill 优先级低于内置命令 |
| 文件系统权限问题 | 低 | 静默失败，记录警告 |

## Definition of Done
- ✅ 所有单元测试通过
- ✅ 集成测试覆盖主要场景
- ✅ 文档完整（README + Skills 专题文档）
- ✅ 至少 3 个示例技能
- ✅ 代码通过 clippy 检查
- ✅ 用户可以创建和使用自定义 skill
