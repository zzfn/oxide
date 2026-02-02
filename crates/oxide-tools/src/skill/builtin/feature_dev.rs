//! Feature Dev 技能
//!
//! 引导用户完成功能开发流程：
//! 理解需求 → 探索代码 → 规划实现 → 执行 → 测试 → 提交

use crate::skill::types::{Skill, SkillContext, SkillKind, SkillResult};
use anyhow::Result;
use async_trait::async_trait;

/// Feature Dev 技能
///
/// 提供结构化的功能开发工作流
pub struct FeatureDevSkill;

impl FeatureDevSkill {
    /// 创建新的 Feature Dev 技能实例
    pub fn new() -> Self {
        Self
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

    /// 获取当前分支名
    async fn get_current_branch(&self, ctx: &SkillContext) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "branch",
                "--show-current",
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Ok("unknown".to_string());
        }

        let branch = String::from_utf8_lossy(&output.stdout);
        Ok(branch.trim().to_string())
    }

    /// 检查工作目录是否干净
    async fn is_working_directory_clean(&self, ctx: &SkillContext) -> bool {
        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "status",
                "--porcelain",
            ])
            .output()
            .await;

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.trim().is_empty()
            }
            Err(_) => false,
        }
    }

    /// 生成开发计划模板
    fn generate_plan_template(&self, feature_name: &str) -> String {
        format!(
            r#"# 功能开发计划: {}

## 1. 理解需求
- [ ] 明确功能目标
- [ ] 识别关键用户场景
- [ ] 确定验收标准

## 2. 探索代码
- [ ] 查找相关模块和文件
- [ ] 理解现有架构
- [ ] 识别需要修改的组件

## 3. 规划设计
- [ ] 设计 API 接口（如需要）
- [ ] 规划数据模型变更
- [ ] 确定测试策略

## 4. 实现
- [ ] 编写核心逻辑
- [ ] 添加单元测试
- [ ] 更新文档

## 5. 验证
- [ ] 运行测试套件
- [ ] 手动验证功能
- [ ] 代码审查

## 6. 提交
- [ ] 整理提交信息
- [ ] 创建 PR/MR
"#,
            feature_name
        )
    }

    /// 创建功能分支
    async fn create_feature_branch(
        &self,
        ctx: &SkillContext,
        feature_name: &str,
    ) -> Result<String> {
        // 规范化分支名
        let branch_name = format!(
            "feature/{}-{}",
            feature_name
                .to_lowercase()
                .replace(" ", "-")
                .replace("_", "-")
                .replace("/", "-"),
            std::process::id()
        );

        let output = tokio::process::Command::new("git")
            .args([
                "-C",
                ctx.working_dir().to_str().unwrap_or("."),
                "checkout",
                "-b",
                &branch_name,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("创建分支失败: {}", stderr));
        }

        Ok(branch_name)
    }
}

impl Default for FeatureDevSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for FeatureDevSkill {
    fn name(&self) -> &str {
        "feature-dev"
    }

    fn description(&self) -> &str {
        "引导完成功能开发流程"
    }

    fn kind(&self) -> SkillKind {
        SkillKind::Builtin
    }

    fn usage(&self) -> &str {
        "/feature-dev [feature-name] [--create-branch]"
    }

    async fn execute(&self, ctx: SkillContext) -> Result<SkillResult> {
        let mut output = Vec::new();
        output.push("# 🚀 功能开发工作流\n".to_string());

        // 获取功能名称
        let feature_name = ctx
            .get_arg("feature-name", 0)
            .cloned()
            .unwrap_or_else(|| "new-feature".to_string());

        output.push(format!("## 功能: {}\n", feature_name));

        // 检查 Git 状态
        if self.is_git_repo(&ctx).await {
            let branch = self.get_current_branch(&ctx).await.unwrap_or_default();
            output.push(format!("当前分支: `{}`", branch));

            // 检查工作目录是否干净
            if !self.is_working_directory_clean(&ctx).await {
                output.push("\n⚠️ **警告**: 工作目录有未提交的变更".to_string());
                output.push("建议先提交或暂存当前变更".to_string());
            }

            // 是否创建新分支
            let create_branch = ctx.args().iter().any(|arg| arg == "--create-branch");
            if create_branch {
                match self.create_feature_branch(&ctx, &feature_name).await {
                    Ok(new_branch) => {
                        output.push(format!("\n✓ 已创建并切换到分支: `{}`", new_branch));
                    }
                    Err(e) => {
                        output.push(format!("\n✗ 创建分支失败: {}", e));
                    }
                }
            }
        } else {
            output.push("⚠️ 当前目录不是 Git 仓库".to_string());
        }

        // 生成开发计划
        output.push("\n---\n".to_string());
        output.push("## 📋 开发计划模板\n".to_string());
        output.push("你可以复制以下内容到计划文件：\n".to_string());
        output.push("```markdown".to_string());
        output.push(self.generate_plan_template(&feature_name));
        output.push("```".to_string());

        // 提供下一步建议
        output.push("\n---\n".to_string());
        output.push("## 💡 下一步建议\n".to_string());
        output.push("1. 使用 `/plan` 进入计划模式，详细规划实现步骤".to_string());
        output.push("2. 描述你的功能需求，让我帮你分析代码结构".to_string());
        output.push("3. 运行测试确保现有功能正常".to_string());

        Ok(SkillResult::Success(output.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_dev_skill_name() {
        let skill = FeatureDevSkill::new();
        assert_eq!(skill.name(), "feature-dev");
    }

    #[test]
    fn test_feature_dev_skill_kind() {
        let skill = FeatureDevSkill::new();
        assert_eq!(skill.kind(), SkillKind::Builtin);
    }

    #[test]
    fn test_generate_plan_template() {
        let skill = FeatureDevSkill::new();
        let template = skill.generate_plan_template("test-feature");
        assert!(template.contains("功能开发计划: test-feature"));
        assert!(template.contains("理解需求"));
        assert!(template.contains("探索代码"));
        assert!(template.contains("规划设计"));
    }
}
