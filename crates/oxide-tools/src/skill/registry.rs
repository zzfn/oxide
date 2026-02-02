//! 技能注册表
//!
//! 管理所有可用技能，支持内置技能和自定义技能。

use super::types::{Skill, SkillKind, SkillMetadata};
use super::loader::SkillLoader;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 技能注册表
///
/// 管理所有可用技能的注册和查找
pub struct SkillRegistry {
    /// 已注册的技能
    skills: RwLock<HashMap<String, Arc<dyn Skill>>>,
    /// 技能加载器
    loader: SkillLoader,
    /// 自定义技能目录列表
    custom_skills_dirs: Vec<std::path::PathBuf>,
}

impl SkillRegistry {
    /// 创建新的技能注册表
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(HashMap::new()),
            loader: SkillLoader::new(),
            custom_skills_dirs: Vec::new(),
        }
    }

    /// 添加自定义技能目录
    pub fn with_skills_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.custom_skills_dirs.push(dir);
        self
    }

    /// 添加多个自定义技能目录
    pub fn with_skills_dirs(mut self, dirs: Vec<std::path::PathBuf>) -> Self {
        self.custom_skills_dirs.extend(dirs);
        self
    }

    /// 注册技能
    pub async fn register(&self, skill: Arc<dyn Skill>) {
        let name = skill.name().to_string();
        let mut skills = self.skills.write().await;
        skills.insert(name, skill);
    }

    /// 注册多个技能
    pub async fn register_many(&self, skills: Vec<Arc<dyn Skill>>) {
        for skill in skills {
            self.register(skill).await;
        }
    }

    /// 获取技能
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Skill>> {
        let skills = self.skills.read().await;
        skills.get(name).cloned()
    }

    /// 检查技能是否存在
    pub async fn contains(&self, name: &str) -> bool {
        let skills = self.skills.read().await;
        skills.contains_key(name)
    }

    /// 获取所有技能名称
    pub async fn skill_names(&self) -> Vec<String> {
        let skills = self.skills.read().await;
        skills.keys().cloned().collect()
    }

    /// 获取所有技能元数据
    pub async fn list_metadata(&self) -> Vec<SkillMetadata> {
        let skills = self.skills.read().await;
        skills
            .values()
            .map(|s| SkillMetadata::from_skill(s.as_ref()))
            .collect()
    }

    /// 获取内置技能列表
    pub async fn list_builtin(&self) -> Vec<SkillMetadata> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|s| s.kind() == SkillKind::Builtin)
            .map(|s| SkillMetadata::from_skill(s.as_ref()))
            .collect()
    }

    /// 获取自定义技能列表
    pub async fn list_custom(&self) -> Vec<SkillMetadata> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|s| s.kind() == SkillKind::Custom)
            .map(|s| SkillMetadata::from_skill(s.as_ref()))
            .collect()
    }

    /// 卸载技能
    pub async fn unregister(&self, name: &str) -> bool {
        let mut skills = self.skills.write().await;
        skills.remove(name).is_some()
    }

    /// 加载自定义技能
    pub async fn load_custom_skills(&self) -> Result<usize> {
        let mut count = 0;

        for dir in &self.custom_skills_dirs {
            if !dir.exists() {
                continue;
            }

            let definitions = self.loader.load_from_directory(dir)?;

            for (def, path) in definitions {
                let skill = super::loader::CustomSkill::new(def, path);
                self.register(Arc::new(skill)).await;
                count += 1;
            }
        }

        Ok(count)
    }

    /// 重新加载所有自定义技能
    pub async fn reload_custom_skills(&self) -> Result<(usize, usize)> {
        // 先移除所有自定义技能
        let custom_names: Vec<String> = {
            let skills = self.skills.read().await;
            skills
                .values()
                .filter(|s| s.kind() == SkillKind::Custom)
                .map(|s| s.name().to_string())
                .collect()
        };

        for name in &custom_names {
            self.unregister(name).await;
        }

        // 重新加载
        let loaded = self.load_custom_skills().await?;

        Ok((custom_names.len(), loaded))
    }

    /// 获取技能数量
    pub async fn len(&self) -> usize {
        let skills = self.skills.read().await;
        skills.len()
    }

    /// 是否为空
    pub async fn is_empty(&self) -> bool {
        let skills = self.skills.read().await;
        skills.is_empty()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建包含所有内置技能的注册表
pub async fn create_registry_with_builtin() -> Arc<SkillRegistry> {
    let registry = SkillRegistry::new();

    // 注册所有内置技能
    registry.register(Arc::new(super::builtin::CommitSkill::new())).await;
    registry.register(Arc::new(super::builtin::FeatureDevSkill::new())).await;

    Arc::new(registry)
}
