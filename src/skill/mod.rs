pub mod executor;
pub mod loader;
pub mod manager;

pub use executor::SkillExecutor;
pub use manager::SkillManager;

use serde::Deserialize;

/// 表示一个 Skill 技能
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub template: String,
    pub args: Vec<SkillArg>,
    pub source: SkillSource,
}

/// Skill 参数定义
#[derive(Debug, Clone, Deserialize)]
pub struct SkillArg {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
}

/// Skill 来源
#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    /// 内置技能
    BuiltIn,
    /// 全局技能 ( ~/.oxide/skills/ )
    Global,
    /// 本地技能 ( .oxide/skills/ )
    Local,
}

impl Skill {
    /// 从组件创建 Skill
    pub(crate) fn from_parts(
        name: String,
        description: String,
        template: String,
        args: Vec<SkillArg>,
        source: SkillSource,
    ) -> Self {
        Self {
            name,
            description,
            template,
            args,
            source,
        }
    }
}
