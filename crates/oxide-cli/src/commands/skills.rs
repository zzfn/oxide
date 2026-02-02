//! 技能命令
//!
//! 实现 /skills 命令，用于列出和管理技能。

use super::registry::{Command, CommandResult};
use crate::app::SharedAppState;
use anyhow::Result;
use async_trait::async_trait;
use oxide_tools::skill::SkillRegistry;
use std::sync::Arc;

/// 技能列表命令
pub struct SkillsCommand {
    registry: Arc<SkillRegistry>,
}

impl SkillsCommand {
    /// 创建新的技能命令
    pub fn new(registry: Arc<SkillRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl Command for SkillsCommand {
    fn name(&self) -> &str {
        "skills"
    }

    fn description(&self) -> &str {
        "列出和管理技能"
    }

    fn usage(&self) -> &str {
        "/skills [reload]"
    }

    async fn execute(&self, args: &[&str], _state: SharedAppState) -> Result<CommandResult> {
        if let Some(subcmd) = args.first() {
            match *subcmd {
                "reload" => {
                    let (removed, loaded) = self.registry.reload_custom_skills().await?;
                    return Ok(CommandResult::Message(format!(
                        "✓ 已重新加载技能\n- 移除: {} 个\n- 加载: {} 个",
                        removed, loaded
                    )));
                }
                _ => {
                    return Ok(CommandResult::Message(format!(
                        "未知子命令: {}。可用: reload",
                        subcmd
                    )));
                }
            }
        }

        // 列出所有技能
        let mut output = String::from("## 可用技能\n\n");

        // 内置技能
        let builtin = self.registry.list_builtin().await;
        if !builtin.is_empty() {
            output.push_str("### 内置技能\n\n");
            for skill in builtin {
                output.push_str(&format!(
                    "- **{}** - {}\n  用法: `{}`\n",
                    skill.name, skill.description, skill.usage
                ));
            }
            output.push('\n');
        }

        // 自定义技能
        let custom = self.registry.list_custom().await;
        if !custom.is_empty() {
            output.push_str("### 自定义技能\n\n");
            for skill in custom {
                output.push_str(&format!(
                    "- **{}** - {}\n  用法: `{}`\n",
                    skill.name, skill.description, skill.usage
                ));
            }
        } else {
            output.push_str(
                "### 自定义技能\n\n暂无自定义技能。\n"
            );
            output.push_str(
                "你可以在 `~/.oxide/skills/` 目录下创建技能目录来添加自定义技能。\n\n"
            );
            output.push_str(
                "例如创建 `~/.oxide/skills/my-skill/SKILL.md`:\n"
            );
            output.push_str(
                "```markdown\n---\nname: my-skill\ndescription: 描述这个技能的作用\n---\n\n# My Skill\n\n技能内容...\n```\n"
            );
        }

        output.push_str("\n---\n");
        output.push_str("提示: 使用 `/{skill_name}` 执行技能，如 `/commit`\n");
        output.push_str("使用 `/skills reload` 重新加载自定义技能。");

        Ok(CommandResult::Message(output))
    }
}

/// 动态技能命令包装器
///
/// 将 Skill 包装为 Command，使其可以通过 /skill-name 调用
pub struct SkillCommandWrapper {
    skill: Arc<dyn oxide_tools::skill::Skill>,
}

impl SkillCommandWrapper {
    /// 创建新的技能命令包装器
    pub fn new(skill: Arc<dyn oxide_tools::skill::Skill>) -> Self {
        Self { skill }
    }
}

#[async_trait]
impl Command for SkillCommandWrapper {
    fn name(&self) -> &str {
        self.skill.name()
    }

    fn description(&self) -> &str {
        self.skill.description()
    }

    fn usage(&self) -> &str {
        self.skill.usage()
    }

    async fn execute(&self,
        args: &[&str],
        state: SharedAppState,
    ) -> Result<CommandResult> {
        let app_state = state.read().await;

        // 创建技能上下文
        let ctx = oxide_tools::skill::SkillContext::new(app_state.working_dir.clone())
            .with_args(args.iter().map(|s| s.to_string()).collect());

        drop(app_state);

        // 执行技能
        match self.skill.execute(ctx).await {
            Ok(result) => {
                let msg = match result {
                    oxide_tools::skill::SkillResult::Success(msg) => msg,
                    oxide_tools::skill::SkillResult::Error(msg) => {
                        format!("❌ 技能执行失败: {}", msg)
                    }
                    oxide_tools::skill::SkillResult::Message(msg) => msg,
                    oxide_tools::skill::SkillResult::Confirm { message } => {
                        format!("⏸️ 需要确认: {}", message)
                    }
                };
                Ok(CommandResult::Message(msg))
            }
            Err(e) => Ok(CommandResult::Message(format!(
                "❌ 技能执行出错: {}",
                e
            ))),
        }
    }
}

/// 注册所有技能命令到注册表
pub async fn register_skill_commands(
    registry: &mut super::registry::CommandRegistry,
    project_dir: Option<std::path::PathBuf>,
) {
    // 创建技能注册表并加载内置技能和自定义技能
    let skill_registry =
        match oxide_tools::skill::create_registry_with_custom_and_project(project_dir).await {
            Ok(registry) => registry,
            Err(e) => {
                eprintln!("警告: 加载自定义技能失败: {}", e);
                // 如果加载自定义技能失败，至少返回一个包含内置技能的注册表
                oxide_tools::skill::create_registry().await
            }
        };

    // 注册 /skills 命令
    registry.register(Arc::new(SkillsCommand::new(Arc::clone(&skill_registry,
    ))));

    // 注册所有技能为命令
    let skills = skill_registry.list_metadata().await;
    for skill_meta in skills {
        if let Some(skill) = skill_registry.get(&skill_meta.name).await {
            registry.register(Arc::new(SkillCommandWrapper::new(skill)));
        }
    }
}

use std::sync::OnceLock;

/// 全局技能注册表
static GLOBAL_SKILL_REGISTRY: OnceLock<Arc<SkillRegistry>> = OnceLock::new();

/// 初始化全局技能注册表
pub async fn init_global_skill_registry() {
    let registry = oxide_tools::skill::create_registry().await;
    let _ = GLOBAL_SKILL_REGISTRY.set(registry);
}

/// 获取全局技能注册表
pub fn get_global_skill_registry() -> Option<Arc<SkillRegistry>> {
    GLOBAL_SKILL_REGISTRY.get().cloned()
}
