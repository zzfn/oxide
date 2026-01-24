use crate::skill::{Skill, SkillArg, SkillSource};
use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Front matter 结构（用于解析 Markdown 文件头部）
#[derive(Debug, Deserialize)]
struct SkillFrontMatter {
    name: String,
    description: String,
    #[serde(default)]
    args: Vec<SkillArg>,
}

/// Skill 文件加载器
pub struct SkillLoader {
    local_dir: PathBuf,
    global_dir: PathBuf,
}

impl SkillLoader {
    /// 创建新的 SkillLoader
    pub fn new() -> Result<Self> {
        let global_dir = dirs::home_dir()
            .map(|p| p.join(".oxide/skills"))
            .unwrap_or_else(|| PathBuf::from("~/.oxide/skills"));

        Ok(Self {
            local_dir: PathBuf::from(".oxide/skills"),
            global_dir,
        })
    }

    /// 加载所有技能，优先级：本地 > 全局 > 内置
    pub fn load_all(&self) -> Result<HashMap<String, Skill>> {
        let mut skills = HashMap::new();

        // 1. 加载内置技能（最低优先级）
        self.load_built_in_skills(&mut skills)?;

        // 2. 加载全局技能
        if self.global_dir.exists() {
            self.load_skills_from_dir(&self.global_dir, &mut skills, SkillSource::Global)?;
        }

        // 3. 加载本地技能（最高优先级，可覆盖）
        if self.local_dir.exists() {
            self.load_skills_from_dir(&self.local_dir, &mut skills, SkillSource::Local)?;
        }

        Ok(skills)
    }

    /// 从指定目录加载技能
    fn load_skills_from_dir(
        &self,
        dir: &Path,
        skills: &mut HashMap<String, Skill>,
        source: SkillSource,
    ) -> Result<()> {
        for entry in WalkDir::new(dir)
            .max_depth(1)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            // 只处理 .md 文件
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            match self.parse_skill_file(path, source.clone()) {
                Ok(skill) => {
                    skills.insert(skill.name.clone(), skill);
                }
                Err(e) => {
                    eprintln!(
                        "{} Warning: Failed to load skill '{}': {}",
                        "⚠️".yellow(),
                        path.display(),
                        e
                    );
                }
            }
        }
        Ok(())
    }

    /// 解析单个 Skill 文件
    fn parse_skill_file(&self, path: &Path, source: SkillSource) -> Result<Skill> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read skill file: {}", path.display()))?;

        // 分割 front matter 和模板内容
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!(
                "Invalid skill format: missing front matter. Expected: ---\nfront matter\n---\ntemplate"
            );
        }

        // 解析 front matter
        let front_matter: SkillFrontMatter = serde_yaml::from_str(parts[1])
            .with_context(|| format!("Failed to parse front matter in: {}", path.display()))?;

        // 提取模板内容
        let template = parts[2].to_string();

        Ok(Skill::from_parts(
            front_matter.name,
            front_matter.description,
            template,
            front_matter.args,
            source,
        ))
    }

    /// 加载内置技能
    fn load_built_in_skills(&self, skills: &mut HashMap<String, Skill>) -> Result<()> {
        // 提供一些内置的示例技能
        // 这些技能会被用户自定义的技能覆盖

        // /commit 技能
        let commit_skill = Skill {
            name: "commit".to_string(),
            description: "Create a git commit following conventional commits".to_string(),
            template: r#"Create a git commit with the following message following these guidelines:

1. Use conventional commit format (feat:, fix:, docs:, refactor:, etc.)
2. Keep the description concise and focused on the "why" rather than the "what"
3. For complex changes, break into multiple commits
4. Include Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>

The user wants to commit with message: {{m}}

Please run the appropriate git commands to create this commit."#
                .to_string(),
            args: vec![SkillArg {
                name: "m".to_string(),
                description: "Commit message".to_string(),
                required: true,
                default: None,
            }],
            source: SkillSource::BuiltIn,
        };
        skills.insert(commit_skill.name.clone(), commit_skill);

        // /compact 技能
        let compact_skill = Skill {
            name: "compact".to_string(),
            description: "Compress the current session by creating a summary".to_string(),
            template: r#"Please create a concise summary of our current conversation, focusing on:
1. Key topics discussed
2. Important decisions made
3. Code changes implemented
4. Next steps or action items

Format this as a brief session summary that can be used to continue this conversation later with full context."#
                .to_string(),
            args: vec![],
            source: SkillSource::BuiltIn,
        };
        skills.insert(compact_skill.name.clone(), compact_skill);

        // /review 技能
        let review_skill = Skill {
            name: "review".to_string(),
            description: "Review code changes and provide feedback".to_string(),
            template: r#"Please review the current code changes and provide feedback on:

1. **Correctness**: Are there any bugs or logic errors?
2. **Style**: Does the code follow best practices and style guidelines?
3. **Performance**: Are there any performance issues or optimizations?
4. **Security**: Are there any security vulnerabilities?
5. **Maintainability**: Is the code easy to understand and maintain?

{{scope}}

Please provide specific, actionable feedback with examples where appropriate."#
                .to_string(),
            args: vec![SkillArg {
                name: "scope".to_string(),
                description: "Specific scope to review (optional)".to_string(),
                required: false,
                default: None,
            }],
            source: SkillSource::BuiltIn,
        };
        skills.insert(review_skill.name.clone(), review_skill);

        Ok(())
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            local_dir: PathBuf::from(".oxide/skills"),
            global_dir: PathBuf::from("~/.oxide/skills"),
        })
    }
}
