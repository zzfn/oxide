//! 技能系统
//!
//! 提供可扩展的工作流定义和执行系统。
//!
//! ## 核心概念
//!
//! - **Skill**: 预定义工作流，可通过 `/skill-name` 语法调用
//! - **SkillKind**: 技能类型（Builtin/Custom）
//! - **SkillContext**: 执行上下文，包含工作目录、参数等
//! - **SkillResult**: 执行结果
//!
//! ## 使用示例
//!
//! ```rust
//! use oxide_tools::skill::{SkillRegistry, SkillContext, CommitSkill};
//! use std::sync::Arc;
//!
//! async fn example() {
//!     let registry = SkillRegistry::new();
//!     // 注册内置技能
//!     registry.register(Arc::new(CommitSkill::new())).await;
//!
//!     // 执行技能
//!     if let Some(skill) = registry.get("commit").await {
//!         let ctx = SkillContext::new(std::env::current_dir().unwrap());
//!         let result = skill.execute(ctx).await.unwrap();
//!     }
//! }
//! ```

pub mod types;
pub mod registry;
pub mod loader;
pub mod builtin;

// 重新导出核心类型
pub use types::{
    Skill, SkillContext, SkillDefinition, SkillKind, SkillMetadata, SkillResult,
};
pub use registry::SkillRegistry;
pub use loader::SkillLoader;
pub use builtin::{CommitSkill, FeatureDevSkill};

use std::sync::Arc;

/// 创建包含所有内置技能的注册表
pub async fn create_registry() -> Arc<SkillRegistry> {
    let registry = Arc::new(SkillRegistry::new());

    // 注册内置技能
    registry.register(Arc::new(CommitSkill::new())).await;
    registry.register(Arc::new(FeatureDevSkill::new())).await;

    registry
}

/// 创建注册表并加载自定义技能（仅全局）
pub async fn create_registry_with_custom() -> anyhow::Result<Arc<SkillRegistry>> {
    create_registry_with_custom_and_project(None).await
}

/// 创建注册表并加载自定义技能（全局 + 项目级）
pub async fn create_registry_with_custom_and_project(
    project_dir: Option<std::path::PathBuf>,
) -> anyhow::Result<Arc<SkillRegistry>> {
    let global_skills_dir = oxide_core::config::oxide_home()?.join("skills");

    let mut registry = SkillRegistry::new().with_skills_dir(global_skills_dir);

    // 如果提供了项目目录，添加项目技能目录
    if let Some(project_dir) = project_dir {
        let project_skills_dir = oxide_core::config::project_skills_dir(&project_dir);
        registry = registry.with_skills_dir(project_skills_dir);
    }

    let registry = Arc::new(registry);

    // 注册内置技能
    registry.register(Arc::new(CommitSkill::new())).await;
    registry.register(Arc::new(FeatureDevSkill::new())).await;

    // 加载自定义技能（先加载全局，后加载项目，项目技能会覆盖全局同名技能）
    registry.load_custom_skills().await?;

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_registry_with_custom() {
        // 确保测试前技能目录存在
        let skills_dir = oxide_core::config::oxide_home().unwrap().join("skills");
        std::fs::create_dir_all(&skills_dir).ok();

        // 创建一个测试技能
        let test_skill_dir = skills_dir.join("test-skill");
        std::fs::create_dir_all(&test_skill_dir).ok();
        std::fs::write(
            test_skill_dir.join("SKILL.md"),
            "---\nname: test-skill\ndescription: A test skill\n---\n\n# Test\n",
        ).ok();

        // 测试加载
        let registry = create_registry_with_custom().await.unwrap();

        // 检查内置技能
        assert!(registry.get("commit").await.is_some());
        assert!(registry.get("feature-dev").await.is_some());

        // 检查自定义技能
        assert!(registry.get("test-skill").await.is_some());

        // 清理
        std::fs::remove_dir_all(&test_skill_dir).ok();
    }
}
