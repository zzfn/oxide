//! 技能文件加载器
//!
//! 从 Markdown SKILL.md 文件加载自定义技能定义。
//!
//! 技能目录结构:
//! ~/.oxide/skills/
//! └── {skill-name}/
//!     ├── SKILL.md          # 必需，Markdown 格式，包含 YAML frontmatter
//!     └── resources/         # 可选，技能资源文件
//!         ├── scripts/
//!         ├── references/
//!         └── assets/

use super::types::{Skill, SkillContext, SkillDefinition, SkillKind, SkillResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// YAML frontmatter 元数据
#[derive(Debug, Clone, serde::Deserialize)]
struct SkillFrontmatter {
    /// 技能名称
    name: String,
    /// 技能描述
    description: String,
}

/// 技能加载器
pub struct SkillLoader;

impl SkillLoader {
    /// 创建新的加载器
    pub fn new() -> Self {
        Self
    }

    /// 从技能目录加载所有技能定义
    ///
    /// 目录结构: {base_dir}/{skill-name}/SKILL.md
    pub fn load_from_directory(&self,
        base_dir: &Path,
    ) -> Result<Vec<(SkillDefinition, PathBuf)>> {
        let mut results = Vec::new();

        if !base_dir.exists() {
            return Ok(results);
        }

        // 遍历技能目录下的所有子目录
        for entry in std::fs::read_dir(base_dir)? {
            let entry = entry?;
            let path = entry.path();

            // 只处理目录
            if !path.is_dir() {
                continue;
            }

            // 查找 SKILL.md 文件
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                match self.load_skill_md(&skill_md) {
                    Ok(def) => results.push((def, skill_md)),
                    Err(e) => {
                        eprintln!("警告: 加载技能文件 {:?} 失败: {}", skill_md, e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// 从单个 SKILL.md 文件加载技能定义
    fn load_skill_md(&self,
        path: &Path,
    ) -> Result<SkillDefinition> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("读取技能文件失败: {:?}", path))?;

        // 解析 YAML frontmatter 和 Markdown body
        let (frontmatter, body) = self.parse_skill_md(&content)
            .with_context(|| format!("解析技能文件失败: {:?}", path))?;

        // 验证 frontmatter
        if frontmatter.name.is_empty() {
            return Err(anyhow::anyhow!("技能名称不能为空"));
        }

        if frontmatter.description.is_empty() {
            return Err(anyhow::anyhow!("技能描述不能为空: {}", frontmatter.name));
        }

        // 验证名称格式（只允许字母、数字、连字符、下划线）
        if !frontmatter
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow::anyhow!(
                "技能名称只能包含字母、数字、连字符和下划线: {}",
                frontmatter.name
            ));
        }

        // 获取技能目录路径（SKILL.md 所在目录的父目录）
        let base_path = path.parent().map(|p| p.to_path_buf());

        Ok(SkillDefinition {
            name: frontmatter.name,
            description: frontmatter.description,
            content: body.to_string(),
            base_path,
        })
    }

    /// 解析 Markdown 技能文件
    ///
    /// 格式:
    /// ```markdown
    /// ---
    /// name: skill-name
    /// description: Skill description
    /// ---
    ///
    /// # Skill Content
    /// ...
    /// ```
    fn parse_skill_md<'a>(
        &self,
        content: &'a str,
    ) -> Result<(SkillFrontmatter, &'a str)> {
        // 查找 YAML frontmatter 分隔符
        let Some(start) = content.find("---") else {
            return Err(anyhow::anyhow!("找不到 YAML frontmatter 起始标记 ---"));
        };

        // 查找第二个 ---
        let rest = &content[start + 3..];
        let Some(end) = rest.find("---") else {
            return Err(anyhow::anyhow!("找不到 YAML frontmatter 结束标记 ---"));
        };

        // 提取 YAML frontmatter
        let yaml_str = &rest[..end];
        let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_str)
            .map_err(|e| anyhow::anyhow!("解析 YAML frontmatter 失败: {}", e))?;

        // 提取 Markdown body（frontmatter 之后的内容）
        let body_start = start + 3 + end + 3;
        let body = content[body_start..].trim_start();

        Ok((frontmatter, body))
    }

    /// 从文件路径推断技能名称
    fn skill_name_from_path(&self,
        path: &Path,
    ) -> Option<String> {
        path.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 自定义技能（从 Markdown SKILL.md 加载）
pub struct CustomSkill {
    definition: SkillDefinition,
    source: PathBuf,
}

impl CustomSkill {
    /// 创建新的自定义技能
    pub fn new(definition: SkillDefinition, source: PathBuf) -> Self {
        Self {
            definition,
            source,
        }
    }

    /// 获取源文件路径
    pub fn source(&self) -> &Path {
        &self.source
    }

    /// 获取技能定义
    pub fn definition(&self) -> &SkillDefinition {
        &self.definition
    }
}

#[async_trait]
impl Skill for CustomSkill {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn description(&self) -> &str {
        &self.definition.description
    }

    fn kind(&self) -> SkillKind {
        SkillKind::Custom
    }

    fn usage(&self) -> &str {
        &self.definition.name
    }

    async fn execute(&self, _ctx: SkillContext) -> Result<SkillResult> {
        // 对于自定义 Markdown 技能，返回技能内容作为提示
        // 实际应用中，这应该将内容发送给 LLM 进行处理
        Ok(SkillResult::Success(format!(
            "## 技能: {}\n\n{}\n\n---\n\n技能内容已加载。请根据上述指导执行相应操作。",
            self.definition.name,
            self.definition.content
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_skill_md(dir: &Path, name: &str, content: &str) -> PathBuf {
        let skill_dir = dir.join(name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        let skill_md = skill_dir.join("SKILL.md");
        let mut file = std::fs::File::create(&skill_md).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        skill_md
    }

    #[test]
    fn test_parse_skill_md() {
        let loader = SkillLoader::new();
        let content = r#"---
name: test-skill
description: A test skill
---

# Test Skill

This is the skill content.
"#;

        let (frontmatter, body) = loader.parse_skill_md(content).unwrap();
        assert_eq!(frontmatter.name, "test-skill");
        assert_eq!(frontmatter.description, "A test skill");
        assert!(body.contains("# Test Skill"));
    }

    #[test]
    fn test_load_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path();

        // 创建测试技能
        create_test_skill_md(
            skills_dir,
            "hello",
            r#"---
name: hello
description: Say hello
---

# Hello Skill

Say hello to the user.
"#,
        );

        create_test_skill_md(
            skills_dir,
            "goodbye",
            r#"---
name: goodbye
description: Say goodbye
---

# Goodbye Skill

Say goodbye to the user.
"#,
        );

        let loader = SkillLoader::new();
        let results = loader.load_from_directory(skills_dir).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_validate_invalid_name() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path();

        create_test_skill_md(
            skills_dir,
            "invalid skill!",
            r#"---
name: invalid skill!
description: Invalid name
---

Content.
"#,
        );

        let loader = SkillLoader::new();
        let results = loader.load_from_directory(skills_dir).unwrap();
        assert_eq!(results.len(), 0); // 应该跳过无效的技能
    }
}
