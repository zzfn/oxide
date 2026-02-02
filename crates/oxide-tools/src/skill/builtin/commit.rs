//! Commit 技能
//!
//! 执行 Git 提交流程：检查变更 → 添加文件 → 提交 → 推送（可选）

use crate::skill::types::{Skill, SkillContext, SkillKind, SkillResult};
use anyhow::Result;
use async_trait::async_trait;

/// Commit 技能
///
/// 自动化 Git 提交流程
pub struct CommitSkill;

impl CommitSkill {
    /// 创建新的 Commit 技能实例
    pub fn new() -> Self {
        Self
    }

    /// 执行 git status 检查变更
    async fn check_changes(&self, ctx: &SkillContext) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "status",
                "--porcelain",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git status 失败: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// 检查是否在 git 仓库中
    async fn is_git_repo(&self, ctx: &SkillContext) -> bool {
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "rev-parse",
                "--git-dir",
            ])
            .output()
            .await;

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// 获取 diff 摘要
    async fn get_diff_summary(&self, ctx: &SkillContext) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "diff",
                "--stat",
                "HEAD",
            ])
            .output()
            .await?;

        if !output.status.success() {
            // 可能是初始提交，没有 HEAD
            let output = tokio::process::Command::new("git")
                .args([
                    "-C",
                    ctx.working_dir().to_str().unwrap_or("."),
                    "diff",
                    "--stat",
                    "--cached",
                ])
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(stdout.to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// 生成提交信息建议
    async fn suggest_commit_message(&self, ctx: &SkillContext) -> Result<String> {
        // 获取暂存区的文件变更
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "diff",
                "--cached",
                "--name-only",
            ])
            .output()
            .await?;

        let files = String::from_utf8_lossy(&output.stdout);
        let files: Vec<&str> = files.lines().collect();

        if files.is_empty() {
            return Ok("更新代码".to_string());
        }

        // 简单的启发式生成提交信息
        let has_tests = files.iter().any(|f| f.contains("test"));
        let has_docs = files.iter().any(|f| f.ends_with(".md") || f.contains("doc"));
        let has_config = files.iter().any(|f| {
            f.ends_with(".toml")
                || f.ends_with(".yaml")
                || f.ends_with(".yml")
                || f.ends_with(".json")
        });

        let prefix = if has_tests {
            "test"
        } else if has_docs {
            "docs"
        } else if has_config {
            "config"
        } else {
            "feat"
        };

        let scope = if files.len() == 1 {
            format!("({})", files[0].split('/').next().unwrap_or(""))
        } else {
            String::new()
        };

        Ok(format!("{}{}: 更新", prefix, scope))
    }

    /// 执行 git add -A
    async fn stage_all(&self, ctx: &SkillContext) -> Result<()> {
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "add",
                "-A",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git add 失败: {}", stderr));
        }

        Ok(())
    }

    /// 执行 git commit
    async fn commit(&self, ctx: &SkillContext, message: &str) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "commit",
                "-m",
                message,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git commit 失败: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// 执行 git push
    async fn push(&self, ctx: &SkillContext) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .args(["-C", ctx.working_dir().to_str().unwrap_or("."), "push"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git push 失败: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}

impl Default for CommitSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CommitSkill {
    fn name(&self) -> &str {
        "commit"
    }

    fn description(&self) -> &str {
        "执行 Git 提交流程"
    }

    fn kind(&self) -> SkillKind {
        SkillKind::Builtin
    }

    fn usage(&self) -> &str {
        "/commit [message] [--push]"
    }

    async fn execute(&self, ctx: SkillContext) -> Result<SkillResult> {
        // 检查是否在 git 仓库中
        if !self.is_git_repo(&ctx).await {
            return Ok(SkillResult::Error(
                "当前目录不是 Git 仓库".to_string(),
            ));
        }

        // 检查是否有变更
        let changes = self.check_changes(&ctx).await?;
        if changes.trim().is_empty() {
            return Ok(SkillResult::Message("没有要提交的变更".to_string()));
        }

        let mut output = Vec::new();
        output.push("## Git 状态\n".to_string());
        output.push(format!("```\n{}\n```", changes));

        // 获取 diff 摘要
        match self.get_diff_summary(&ctx).await {
            Ok(summary) if !summary.is_empty() => {
                output.push("\n### 变更摘要\n".to_string());
                output.push(format!("```\n{}\n```", summary));
            }
            _ => {}
        }

        // 暂存所有变更
        if let Err(e) = self.stage_all(&ctx).await {
            return Ok(SkillResult::Error(format!("暂存变更失败: {}", e)));
        }
        output.push("\n✓ 已暂存所有变更".to_string());

        // 获取或生成提交信息
        let message = ctx
            .get_arg("message", 0)
            .cloned()
            .unwrap_or_else(|| {
                // 尝试生成建议的提交信息
                // 注意：这里简化处理，实际应该在异步上下文中执行
                "更新代码".to_string()
            });

        output.push(format!("\n### 提交信息\n`{}`", message));

        // 执行提交
        match self.commit(&ctx, &message).await {
            Ok(result) => {
                output.push(format!("\n✓ 提交成功\n```\n{}\n```", result));
            }
            Err(e) => {
                return Ok(SkillResult::Error(format!("提交失败: {}", e)));
            }
        }

        // 检查是否需要推送
        let should_push = ctx.args().iter().any(|arg| arg == "--push");
        if should_push {
            match self.push(&ctx).await {
                Ok(result) => {
                    output.push(format!("\n✓ 推送成功\n```\n{}\n```", result));
                }
                Err(e) => {
                    output.push(format!("\n✗ 推送失败: {}", e));
                }
            }
        }

        Ok(SkillResult::Success(output.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_skill_name() {
        let skill = CommitSkill::new();
        assert_eq!(skill.name(), "commit");
    }

    #[test]
    fn test_commit_skill_kind() {
        let skill = CommitSkill::new();
        assert_eq!(skill.kind(), SkillKind::Builtin);
    }
}
