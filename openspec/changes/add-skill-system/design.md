# Design: Skill System

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                        User Input                        │
│                    /commit -m "Fix bug"                   │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                    CLI Command Router                    │
│                 (src/cli/command.rs)                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Is it a built-in command? (/help, /clear, etc)  │   │
│  └───────────────┬─────────────────────────────────┘   │
│                  │ No                                    │
│  ┌───────────────▼─────────────────────────────────┐   │
│  │       Is it a skill command? (/skills, ...)      │   │
│  └───────────────┬─────────────────────────────────┘   │
│                  │ No                                    │
│  ┌───────────────▼─────────────────────────────────┐   │
│  │         Extract skill name and arguments          │   │
│  │         /commit -m "Fix bug" → commit            │   │
│  └───────────────┬─────────────────────────────────┘   │
└──────────────────┼─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│                  Skill Manager                           │
│                  (src/skill/manager.rs)                  │
│  ┌─────────────────────────────────────────────────┐   │
│  │  1. Check cache for skill                       │   │
│  │  2. If not cached, load from SkillLoader        │   │
│  │  3. Validate skill exists                       │   │
│  └───────────────┬─────────────────────────────────┘   │
└──────────────────┼─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│                 Skill Loader                             │
│                (src/skill/loader.rs)                     │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Scan directories in order:                     │   │
│  │  1. .oxide/skills/          (local, highest)    │   │
│  │  2. ~/.oxide/skills/         (global)           │   │
│  │  3. built-in skills          (fallback)         │   │
│  └───────────────┬─────────────────────────────────┘   │
│  ┌───────────────▼─────────────────────────────────┐   │
│  │  Parse Markdown file:                          │   │
│  │  ---                                            │   │
│  │  name: commit                                   │   │
│  │  description: Create git commit                │   │
│  │  args: [...]                                    │   │
│  │  ---                                            │   │
│  │  <template content>                             │   │
│  └───────────────┬─────────────────────────────────┘   │
└──────────────────┼─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│                 Skill Executor                           │
│                (src/skill/executor.rs)                   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  1. Parse arguments from command                │   │
│  │     "-m 'Fix bug'" → {"message": "Fix bug"}     │   │
│  │  2. Validate required arguments                 │   │
│  │  3. Render template (replace {{var}})           │   │
│  │  4. Return rendered prompt                      │   │
│  └───────────────┬─────────────────────────────────┘   │
└──────────────────┼─────────────────────────────────────┘
                   │
                   │ Rendered Prompt
                   ▼
┌─────────────────────────────────────────────────────────┐
│                   Agent                                  │
│  Process as normal user message, execute tools, etc.    │
└─────────────────────────────────────────────────────────┘
```

## Component Design

### 1. Skill Structure

```rust
// src/skill/types.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub template: String,
    pub args: Vec<SkillArg>,
    pub source: SkillSource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillArg {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    BuiltIn,
    Global,
    Local,
}

// Front matter structure for parsing
#[derive(Debug, Deserialize)]
struct SkillFrontMatter {
    name: String,
    description: String,
    #[serde(default)]
    args: Vec<SkillArg>,
}
```

### 2. Skill Loader

```rust
// src/skill/loader.rs
use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SkillLoader {
    local_dir: PathBuf,
    global_dir: PathBuf,
}

impl SkillLoader {
    pub fn new() -> Result<Self> {
        Ok(Self {
            local_dir: PathBuf::from(".oxide/skills"),
            global_dir: dirs::home_dir()
                .map(|p| p.join(".oxide/skills"))
                .unwrap_or_else(|| PathBuf::from("~/.oxide/skills")),
        })
    }

    /// Load all skills, with precedence: local > global > built-in
    pub fn load_all(&self) -> Result<HashMap<String, Skill>> {
        let mut skills = HashMap::new();

        // 1. Load built-in skills (lowest priority)
        self.load_built_in_skills(&mut skills)?;

        // 2. Load global skills
        if self.global_dir.exists() {
            self.load_skills_from_dir(&self.global_dir, &mut skills, SkillSource::Global)?;
        }

        // 3. Load local skills (highest priority, can override)
        if self.local_dir.exists() {
            self.load_skills_from_dir(&self.local_dir, &mut skills, SkillSource::Local)?;
        }

        Ok(skills)
    }

    fn load_skills_from_dir(
        &self,
        dir: &Path,
        skills: &mut HashMap<String, Skill>,
        source: SkillSource,
    ) -> Result<()> {
        for entry in WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            if let Ok(skill) = self.parse_skill_file(path, source.clone()) {
                skills.insert(skill.name.clone(), skill);
            }
        }
        Ok(())
    }

    fn parse_skill_file(&self, path: &Path, source: SkillSource) -> Result<Skill> {
        let content = std::fs::read_to_string(path)?;

        // Split front matter and template
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!("Invalid skill format: missing front matter");
        }

        let front_matter: SkillFrontMatter = serde_yaml::from_str(parts[1])?;
        let template = parts[2].to_string();

        Ok(Skill {
            name: front_matter.name,
            description: front_matter.description,
            template,
            args: front_matter.args,
            source,
        })
    }
}
```

### 3. Skill Executor

```rust
// src/skill/executor.rs
use anyhow::Result;
use regex::Regex;

pub struct SkillExecutor;

impl SkillExecutor {
    pub fn execute(skill: &Skill, args_str: &str) -> Result<String> {
        // Parse command line arguments
        let parsed_args = Self::parse_args(skill, args_str)?;

        // Validate required arguments
        for arg in &skill.args {
            if arg.required && !parsed_args.contains_key(&arg.name) {
                anyhow::bail!("Missing required argument: {}", arg.name);
            }
        }

        // Render template
        let rendered = Self::render_template(&skill.template, &parsed_args)?;

        Ok(rendered)
    }

    fn parse_args(skill: &Skill, args_str: &str) -> Result<HashMap<String, String>> {
        let mut args = HashMap::new();

        // Simple argument parser: -m "message" or --message "value"
        let re = Regex::new(r#"(?m)^-(\w+)\s+"([^"]+)""#)?;
        for cap in re.captures_iter(args_str) {
            let key = cap[1].to_string();
            let value = cap[2].to_string();
            args.insert(key, value);
        }

        // Apply defaults
        for arg in &skill.args {
            if !args.contains_key(&arg.name) {
                if let Some(default) = &arg.default {
                    args.insert(arg.name.clone(), default.clone());
                }
            }
        }

        Ok(args)
    }

    fn render_template(template: &str, args: &HashMap<String, String>) -> Result<String> {
        let mut rendered = template.to_string();

        // Simple {{var}} replacement
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }

        Ok(rendered)
    }
}
```

### 4. Skill Manager (Caching Layer)

```rust
// src/skill/manager.rs
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

use super::loader::SkillLoader;
use super::types::Skill;

pub static SKILL_CACHE: Lazy<RwLock<HashMap<String, Skill>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct SkillManager {
    loader: SkillLoader,
}

impl SkillManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            loader: SkillLoader::new()?,
        })
    }

    /// Initialize skill cache (call on startup)
    pub fn init(&self) -> Result<()> {
        let skills = self.loader.load_all()?;
        let mut cache = SKILL_CACHE.write().unwrap();
        *cache = skills;
        Ok(())
    }

    pub fn get_skill(&self, name: &str) -> Option<Skill> {
        let cache = SKILL_CACHE.read().unwrap();
        cache.get(name).cloned()
    }

    pub fn list_skills(&self) -> Vec<Skill> {
        let cache = SKILL_CACHE.read().unwrap();
        cache.values().cloned().collect()
    }
}
```

## CLI Integration Points

### Command Routing (src/cli/command.rs)

```rust
impl OxideCli {
    pub async fn handle_command(&mut self, input: &str) -> Result<bool> {
        match input {
            // Existing built-in commands...
            "/help" => { /* ... */ }

            // New skill management commands
            "/skills" | "/skills list" => {
                self.list_skills()?;
            }
            _ if input.starts_with("/skills show ") => {
                let skill_name = input.strip_prefix("/skills show ").unwrap().trim();
                self.show_skill(skill_name)?;
            }
            _ if input.starts_with("/skills create ") => {
                let skill_name = input.strip_prefix("/skills create ").unwrap().trim();
                self.create_skill(skill_name)?;
            }

            // Dynamic skill execution
            _ if input.starts_with('/') && !self.is_builtin_command(input) => {
                if let Some((skill_name, args)) = self.parse_skill_command(input) {
                    self.execute_skill(&skill_name, &args)?;
                } else {
                    println!("{} Unknown command: {}", "❌".red(), input);
                }
            }

            // Regular chat message
            _ => { /* existing logic */ }
        }
        Ok(true)
    }

    fn is_builtin_command(&self, input: &str) -> bool {
        matches!(input.split_whitespace().next(),
            Some("/quit" | "/exit" | "/clear" | "/config" | "/help" |
                  "/history" | "/load" | "/sessions" | "/delete" |
                  "/agent" | "/tasks" | "/skills")
        )
    }

    fn parse_skill_command(&self, input: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let skill_name = parts[0].strip_prefix('/')?;
        let args = parts.get(1).unwrap_or(&"");
        Some((skill_name.to_string(), args.to_string()))
    }
}
```

### Auto-completion Support (src/cli/mod.rs)

```rust
impl Completer for OxideHelper {
    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>)
        -> Result<(usize, Vec<Pair>), ReadlineError>
    {
        if line.starts_with('/') {
            let mut matches = Vec::new();

            // Add built-in commands
            for (cmd, info) in &self.commands {
                // ... existing logic ...
            }

            // Add dynamic skills
            if let Ok(manager) = SkillManager::new() {
                for skill in manager.list_skills() {
                    let cmd = format!("/{}", skill.name);
                    if cmd.starts_with(line) {
                        matches.push(Pair {
                            display: format!("{} - {}", cmd, skill.description),
                            replacement: cmd,
                        });
                    }
                }
            }

            matches.sort_by(|a, b| a.display.cmp(&b.display));
            Ok((0, matches))
        } else {
            Ok((pos, vec![]))
        }
    }
}
```

## Example Skills

### /commit (`.oxide/skills/commit.md`)

```markdown
---
name: commit
description: Create a git commit following conventional commits
args:
  - name: m
    description: Commit message
    required: true
---

Create a git commit with the following message following these guidelines:

1. Use conventional commit format (feat:, fix:, docs:, refactor:, etc.)
2. Keep the description concise and focused on the "why" rather than the "what"
3. For complex changes, break into multiple commits
4. Include Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>

The user wants to commit with message: {{m}}

Please run the appropriate git commands to create this commit.
```

### /compact (`.oxide/skills/compact.md`)

```markdown
---
name: compact
description: Compress the current session by creating a summary
---

Please create a concise summary of our current conversation, focusing on:
1. Key topics discussed
2. Important decisions made
3. Code changes implemented
4. Next steps or action items

Format this as a brief session summary that can be used to continue this conversation later with full context.
```

## Error Handling Strategy

| Error Type | Handling |
|------------|----------|
| Skill file not found | Return友好提示，列出可用 skills |
| Invalid front matter | 显示文件路径和具体错误，跳过该 skill |
| Missing required arg | 提示缺少的参数名和描述 |
| Template parse error | 显示模板错误位置和原因 |
| Permission denied | 警告并跳过，继续加载其他 skills |

## Testing Strategy

### Unit Tests
- `test_parse_front_matter()` - Front matter 解析
- `test_template_rendering()` - 变量替换
- `test_arg_parsing()` - 命令行参数解析
- `test_skill_precedence()` - 本地覆盖全局

### Integration Tests
- `test_load_all_skills()` - 完整加载流程
- `test_execute_skill_with_args()` - 执行带参数的 skill
- `test_skill_command_routing()` - CLI 命令路由

### Manual Testing
1. 创建自定义 skill
2. 执行 skill 并验证输出
3. 测试参数传递
4. 测试错误场景

## Future Enhancements
1. 支持更复杂的模板语法（条件、循环）
2. Skill 验证工具（`oxide skills validate`）
3. Skill 编辑器集成
4. Skill 热重载
5. Skill 版本控制
