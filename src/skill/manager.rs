use super::loader::SkillLoader;
use crate::skill::Skill;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// 全局 Skill 缓存
static SKILL_CACHE: Lazy<RwLock<HashMap<String, Skill>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Skill 管理器（负责缓存和访问技能）
pub struct SkillManager {
    loader: SkillLoader,
}

impl SkillManager {
    /// 创建新的 SkillManager
    pub fn new() -> Result<Self> {
        Ok(Self {
            loader: SkillLoader::new()?,
        })
    }

    /// 初始化技能缓存（启动时调用）
    pub fn init(&self) -> Result<()> {
        let skills = self.loader.load_all()?;
        let mut cache = SKILL_CACHE.write().unwrap();
        *cache = skills;
        Ok(())
    }

    /// 获取指定名称的 Skill
    pub fn get_skill(&self, name: &str) -> Option<Skill> {
        let cache = SKILL_CACHE.read().unwrap();
        cache.get(name).cloned()
    }

    /// 列出所有可用的 Skills
    pub fn list_skills(&self) -> Vec<Skill> {
        let cache = SKILL_CACHE.read().unwrap();
        cache.values().cloned().collect()
    }

    /// 检查技能是否存在
    #[allow(dead_code)]
    pub fn skill_exists(&self, name: &str) -> bool {
        let cache = SKILL_CACHE.read().unwrap();
        cache.contains_key(name)
    }

    /// 重新加载所有技能（清除缓存并重新加载）
    #[allow(dead_code)]
    pub fn reload(&self) -> Result<()> {
        self.init()
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            loader: SkillLoader::default(),
        })
    }
}
