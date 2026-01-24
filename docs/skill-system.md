# 技能系统实现详解

## 目录

- [系统概述](#系统概述)
- [架构设计](#架构设计)
- [技能加载](#技能加载)
- [技能格式](#技能格式)
- [技能执行](#技能执行)
- [技能管理](#技能管理)
- [内置技能](#内置技能)
- [使用指南](#使用指南)
- [扩展开发](#扩展开发)

## 系统概述

Oxide 的技能系统是一个灵活的可重用命令模板系统，允许用户创建、管理和执行自定义命令模板。技能系统采用分层加载机制，支持本地、全局和内置技能，为用户提供了强大的命令抽象能力。

### 核心特性

- **分层加载**: 支持本地、全局、内置三个技能来源
- **模板引擎**: 支持参数化模板和变量替换
- **灵活参数**: 支持必需参数、可选参数和默认值
- **优先级覆盖**: 本地技能可覆盖全局和内置技能
- **缓存机制**: 高效的技能加载和缓存
- **易于扩展**: 通过添加 Markdown 文件即可创建新技能

## 架构设计

### 系统组件

```
┌─────────────────────────────────────┐
│      CLI 层                         │
│  - 命令触发                         │
│  - 参数解析                         │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│    技能管理器 (SkillManager)        │
│  - 技能缓存                         │
│  - 技能查询                         │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│    技能加载器 (SkillLoader)         │
│  - 本地技能                         │
│  - 全局技能                         │
│  - 内置技能                         │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│    技能执行器 (SkillExecutor)       │
│  - 参数解析                         │
│  - 模板渲染                         │
└─────────────────────────────────────┘
```

### 数据结构

```rust
/// 技能定义
pub struct Skill {
    pub name: String,              // 技能名称
    pub description: String,       // 技能描述
    pub template: String,          // 模板内容
    pub args: Vec<SkillArg>,       // 参数列表
    pub source: SkillSource,       // 来源标识
}

/// 技能参数
pub struct SkillArg {
    pub name: String,              // 参数名称
    pub description: String,       // 参数描述
    pub required: bool,            // 是否必需
    pub default: Option<String>,   // 默认值
}

/// 技能来源
pub enum SkillSource {
    BuiltIn,   // 内置技能
    Global,    // 全局技能 (~/.oxide/skills/)
    Local,     // 本地技能 (.oxide/skills/)
}
```

## 技能加载

### 加载优先级

技能系统采用三层加载机制，优先级从高到低：

1. **本地技能** (`.oxide/skills/`) - 最高优先级
2. **全局技能** (`~/.oxide/skills/`)
3. **内置技能** (代码中硬编码) - 最低优先级

高优先级的技能会覆盖低优先级的同名技能。

### 加载流程

```rust
impl SkillLoader {
    pub fn load_all(&self) -> Result<HashMap<String, Skill>> {
        let mut skills = HashMap::new();

        // 1. 先加载内置技能（最低优先级）
        self.load_built_in_skills(&mut skills)?;

        // 2. 加载全局技能
        if self.global_dir.exists() {
            self.load_skills_from_dir(
                &self.global_dir,
                &mut skills,
                SkillSource::Global
            )?;
        }

        // 3. 加载本地技能（最高优先级，可覆盖）
        if self.local_dir.exists() {
            self.load_skills_from_dir(
                &self.local_dir,
                &mut skills,
                SkillSource::Local
            )?;
        }

        Ok(skills)
    }

    fn load_skills_from_dir(
        &self,
        dir: &Path,
        skills: &mut HashMap<String, Skill>,
        source: SkillSource,
    ) -> Result<()> {
        // 读取目录中的所有 .md 文件
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                // 解析技能文件
                let skill = self.parse_skill_file(&path, source)?;
                skills.insert(skill.name.clone(), skill);
            }
        }

        Ok(())
    }
}
```

### 缓存机制

使用 `once_cell` 和 `RwLock` 实现线程安全的缓存：

```rust
use once_cell::sync::Lazy;
use std::sync::RwLock;

static SKILL_CACHE: Lazy<RwLock<HashMap<String, Skill>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

impl SkillManager {
    pub fn init(&self) -> Result<()> {
        // 加载所有技能
        let skills = self.loader.load_all()?;

        // 更新缓存
        let mut cache = SKILL_CACHE.write().unwrap();
        *cache = skills;

        Ok(())
    }

    pub fn get_skill(&self, name: &str) -> Option<Skill> {
        let cache = SKILL_CACHE.read().unwrap();
        cache.get(name).cloned()
    }

    pub fn reload(&self) -> Result<()> {
        // 重新加载技能
        self.init()
    }
}
```

## 技能格式

### 文件格式

技能文件使用 **Markdown + YAML Front Matter** 格式：

```markdown
---
name: skill-name
description: 技能的简短描述
args:
  - name: param1
    description: 第一个参数
    required: true
  - name: param2
    description: 第二个参数
    required: false
    default: "default_value"
---

这里是技能的模板内容。
使用 {{param1}} 和 {{param2}} 作为占位符。

用户提供的内容：{{param1}} 和 {{param2}}
```

### Front Matter 字段

| 字段 | 类型 | 必需 | 说明 |
|-----|------|------|------|
| `name` | String | ✅ | 技能名称（用于 `/skill-name` 调用） |
| `description` | String | ✅ | 技能描述（显示在技能列表中） |
| `args` | Array | ❌ | 参数列表 |
| `args[].name` | String | ✅ | 参数名称 |
| `args[].description` | String | ✅ | 参数描述 |
| `args[].required` | Boolean | ✅ | 是否必需参数 |
| `args[].default` | String | ❌ | 默认值（仅可选参数） |

### 模板语法

使用简单的 `{{variable}}` 语法进行变量替换：

```markdown
分析 {{file}} 中的代码，重点关注：
- {{focus_area}}
- 使用 {{language}} 语言最佳实践

请提供详细的反馈。
```

## 技能执行

### 参数解析

支持多种参数格式：

```rust
fn parse_args(skill: &Skill, args_str: &str) -> Result<HashMap<String, String>> {
    let mut args = HashMap::new();

    // 解析带引号的参数：-m "Hello World"
    let re_quoted = Regex::new(r#"(-\w+|--\w+)\s+"([^"]+)""#)?;
    for cap in re_quoted.captures_iter(args_str) {
        let key = cap[1].trim_start_matches('-').to_string();
        let value = cap[2].to_string();
        args.insert(key, value);
    }

    // 解析不带引号的参数：-m hello
    let re_simple = Regex::new(r#"(-\w+|--\w+)\s+(\S+)"#)?;
    for cap in re_simple.captures_iter(args_str) {
        let key = cap[1].trim_start_matches('-').to_string();
        let value = cap[2].to_string();
        args.entry(key).or_insert(value);
    }

    // 应用默认值
    for arg in &skill.args {
        if !args.contains_key(&arg.name) {
            if let Some(default) = &arg.default {
                args.insert(arg.name.clone(), default.clone());
            } else if arg.required {
                bail!("缺少必需参数: --{}", arg.name);
            }
        }
    }

    Ok(args)
}
```

### 模板渲染

```rust
fn render_template(template: &str, args: &HashMap<String, String>) -> String {
    let mut rendered = template.to_string();

    for (key, value) in args {
        let placeholder = format!("{{{{{}}}}}", key);
        rendered = rendered.replace(&placeholder, value);
    }

    rendered
}
```

### 执行流程

```rust
impl SkillExecutor {
    pub fn execute(skill: &Skill, args_str: &str) -> Result<String> {
        // 1. 解析参数
        let args = parse_args(skill, args_str)?;

        // 2. 渲染模板
        let rendered = render_template(&skill.template, &args);

        // 3. 返回渲染后的提示词
        Ok(rendered)
    }
}
```

### CLI 集成

```rust
impl OxideCli {
    async fn try_execute_skill(&mut self, input: &str) -> Result<bool> {
        // 解析命令：/skillname [args...]
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let skill_name = parts[0].strip_prefix('/')
            .ok_or_else(|| anyhow!("无效的技能命令"))?;

        let args_str = parts.get(1).unwrap_or(&"");

        // 获取技能
        let skill = self.skill_manager
            .get_skill(skill_name)
            .ok_or_else(|| anyhow!("技能不存在: {}", skill_name))?;

        // 执行技能
        let rendered_prompt = SkillExecutor::execute(&skill, args_str)?;

        // 添加到上下文
        self.context_manager
            .add_message(Message::user(&rendered_prompt));

        // 执行 AI 处理
        self.process_ai_response().await?;

        Ok(true)
    }
}
```

## 技能管理

### 列出技能

```bash
/skills list
```

输出示例：

```
可用技能：

🔧 commit - 创建符合 Conventional Commits 规范的 git commit
🔧 compact - 压缩当前会话，创建摘要
🔧 review - 审查代码并提供反馈
🌐 my-custom-skill - 我的自定义全局技能
📁 project-skill - 项目特定技能
```

### 查看技能详情

```bash
/skills show commit
```

输出示例：

```
技能: commit
描述: 创建符合 Conventional Commits 规范的 git commit
来源: 🔧 内置

参数:
  -m, --message: Commit 消息 (必需)

使用示例:
  /commit -m "feat: add new feature"
```

### 重载技能

```bash
/skills reload
```

重新加载所有技能文件，更新缓存。

## 内置技能

### commit - Git 提交

创建符合 Conventional Commits 规范的 Git 提交。

**参数**:
- `-m, --message`: Commit 消息（必需）

**示例**:
```bash
/commit -m "feat: add user authentication"
/commit -m "fix: resolve memory leak"
```

### compact - 会话压缩

压缩当前会话，创建摘要以节省 tokens。

**参数**: 无

**示例**:
```bash
/compact
```

### review - 代码审查

审查代码并提供反馈。

**参数**:
- 无（会自动分析当前上下文）

**示例**:
```bash
/review
```

## 使用指南

### 创建本地技能

1. **创建技能目录**:
```bash
mkdir -p .oxide/skills
```

2. **创建技能文件**:
```bash
cat > .oxide/skills/code-analysis.md << 'EOF'
---
name: analyze
description: 深入分析代码结构和质量
args:
  - name: file
    description: 要分析的文件路径
    required: true
  - name: focus
    description: 分析重点（如：performance, security, readability）
    required: false
    default: "general"
---

请深入分析 {{file}} 的代码结构和质量。

重点关注：{{focus}}

请提供：
1. 整体架构评估
2. 代码质量分析
3. 潜在问题识别
4. 改进建议
EOF
```

3. **使用技能**:
```bash
# 重载技能
/skills reload

# 使用技能
/analyze -file "src/main.rs" -focus "performance"
```

### 创建全局技能

全局技能在所有项目中可用：

```bash
# 创建全局技能目录
mkdir -p ~/.oxide/skills

# 创建技能文件
cat > ~/.oxide/skills/deploy.md << 'EOF'
---
name: deploy
description: 部署应用到生产环境
args:
  - name: env
    description: 部署环境
    required: true
  - name: branch
    description: 要部署的分支
    required: false
    default: "main"
---

执行以下部署步骤：

1. 切换到 {{branch}} 分支
2. 运行测试套件
3. 构建应用
4. 部署到 {{env}} 环境
5. 验证部署状态

请详细报告每个步骤的结果。
EOF
```

### 技能参数技巧

**必需参数**:
```yaml
args:
  - name: file
    description: 文件路径
    required: true
```

**可选参数带默认值**:
```yaml
args:
  - name: language
    description: 编程语言
    required: false
    default: "rust"
```

**多值参数**（用空格分隔）:
```bash
/skill -files "file1.rs file2.rs file3.rs"
```

## 扩展开发

### 高级模板技巧

**条件内容**（通过参数控制）:
```markdown
---
args:
  - name: verbose
    description: 详细输出
    required: false
    default: "false"
---

{{#if verbose}}
请提供详细的分析，包括：
- 每个函数的复杂度
- 所有潜在的性能问题
- 详细的改进建议
{{/if}}

{{#unless verbose}}
请提供简洁的总结。
{{/unless}}
```

**多行模板**:
```markdown
---
name: refactor
description: 重构代码
args:
  - name: file
    required: true
---

请重构 {{file}}：

要求：
1. 提高代码可读性
2. 改善性能
3. 遵循 SOLID 原则
4. 保持功能不变

请先展示重构计划，然后实施重构。
```

### 技能组合

一个技能可以引用另一个技能：

```markdown
---
name: full-review
description: 完整的代码审查流程
args:
  - name: file
    required: true
---

首先执行代码分析：
/analyze -file {{file}} -focus security

然后执行代码审查：
/review

最后提供改进建议。
```

### 技能测试

建议为技能创建测试用例：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_skill_parsing() {
        let content = r#"
---
name: commit
description: Create a commit
args:
  - name: m
    required: true
---
Commit template
"#;

        let skill = parse_skill(content).unwrap();
        assert_eq!(skill.name, "commit");
        assert_eq!(skill.args.len(), 1);
        assert_eq!(skill.args[0].name, "m");
    }
}
```

## 最佳实践

### 技能设计原则

1. **单一职责**: 每个技能只做一件事
2. **清晰命名**: 使用描述性的技能名称
3. **参数化**: 使用参数提高技能灵活性
4. **文档化**: 提供清晰的参数说明和示例
5. **渐进式**: 从简单技能开始，逐步增加复杂度

### 技能组织

```
.oxide/skills/
├── git/           # Git 相关技能
│   ├── commit.md
│   └── pr.md
├── code/          # 代码相关技能
│   ├── analyze.md
│   ├── refactor.md
│   └── test.md
└── deploy/        # 部署相关技能
    ├── staging.md
    └── production.md
```

### 性能优化

1. **使用缓存**: 技能加载后会被缓存，无需重复加载
2. **避免循环引用**: 技能不应引用自身
3. **限制参数数量**: 过多参数会影响用户体验
4. **模板简洁**: 保持模板简洁明了

## 相关文档

- [Agent 系统](./agent-system.md) - Agent 与技能的集成
- [工具系统](./tool-system.md) - 工具调用机制
- [整体架构](./architecture.md) - 项目架构总览
