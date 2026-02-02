//! 技能系统类型定义
//!
//! 定义 Skill trait、上下文和结果类型。

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 技能类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillKind {
    /// 内置技能
    Builtin,
    /// 自定义技能（从文件加载）
    Custom,
}

impl std::fmt::Display for SkillKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillKind::Builtin => write!(f, "builtin"),
            SkillKind::Custom => write!(f, "custom"),
        }
    }
}

/// 技能执行结果
#[derive(Debug, Clone)]
pub enum SkillResult {
    /// 执行成功，返回消息
    Success(String),
    /// 执行失败，返回错误信息
    Error(String),
    /// 返回消息（不表示成功或失败）
    Message(String),
    /// 需要用户确认（返回确认后的消息）
    Confirm { message: String },
}

/// 技能执行上下文
#[derive(Debug, Clone)]
pub struct SkillContext {
    /// 工作目录
    pub working_dir: PathBuf,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 输入参数
    pub args: Vec<String>,
    /// 命名参数
    pub named_args: HashMap<String, String>,
    /// 配置
    pub config: SkillConfig,
}

impl SkillContext {
    /// 创建新的上下文
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            env_vars: std::env::vars().collect(),
            args: Vec::new(),
            named_args: HashMap::new(),
            config: SkillConfig::default(),
        }
    }

    /// 添加参数
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// 添加命名参数
    pub fn with_named_args(mut self, args: HashMap<String, String>) -> Self {
        self.named_args = args;
        self
    }

    /// 获取参数值（支持位置参数和命名参数）
    pub fn get_arg(&self, name: &str, index: usize) -> Option<&String> {
        // 优先从命名参数获取
        if let Some(value) = self.named_args.get(name) {
            return Some(value);
        }
        // 从位置参数获取
        self.args.get(index)
    }

    /// 获取所有参数
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// 获取工作目录
    pub fn working_dir(&self) -> &PathBuf {
        &self.working_dir
    }
}

/// 技能配置
#[derive(Debug, Clone, Default)]
pub struct SkillConfig {
    /// 是否启用调试输出
    pub debug: bool,
    /// 交互模式
    pub interactive: bool,
}

/// Skill trait
///
/// 所有技能（内置或自定义）都需要实现此 trait
#[async_trait]
pub trait Skill: Send + Sync {
    /// 技能名称
    fn name(&self) -> &str;

    /// 技能描述
    fn description(&self) -> &str;

    /// 技能类型
    fn kind(&self) -> SkillKind;

    /// 技能用法说明
    fn usage(&self) -> &str {
        ""
    }

    /// 执行技能
    async fn execute(&self, ctx: SkillContext) -> Result<SkillResult>;

    /// 是否需要确认
    fn needs_confirmation(&self) -> bool {
        false
    }
}

/// 技能定义（用于 Markdown SKILL.md 解析）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// 技能名称（来自 YAML frontmatter）
    pub name: String,
    /// 技能描述（来自 YAML frontmatter）
    pub description: String,
    /// 技能内容（Markdown body）
    pub content: String,
    /// 技能目录路径
    pub base_path: Option<PathBuf>,
}

impl SkillDefinition {
    /// 获取技能资源目录路径
    pub fn resources_dir(&self) -> Option<PathBuf> {
        self.base_path.as_ref().map(|p| p.join("resources"))
    }

    /// 获取特定资源文件路径
    pub fn resource_path(&self, name: &str) -> Option<PathBuf> {
        self.resources_dir().map(|p| p.join(name))
    }
}

/// 技能元数据
#[derive(Debug, Clone)]
pub struct SkillMetadata {
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 技能类型
    pub kind: SkillKind,
    /// 用法说明
    pub usage: String,
    /// 来源文件（自定义技能）
    pub source: Option<PathBuf>,
}

impl SkillMetadata {
    /// 从 Skill 创建元数据
    pub fn from_skill(skill: &dyn Skill) -> Self {
        Self {
            name: skill.name().to_string(),
            description: skill.description().to_string(),
            kind: skill.kind(),
            usage: skill.usage().to_string(),
            source: None,
        }
    }

    /// 从 SkillDefinition 创建元数据
    pub fn from_definition(def: &SkillDefinition, source: PathBuf) -> Self {
        Self {
            name: def.name.clone(),
            description: def.description.clone(),
            kind: SkillKind::Custom,
            usage: format!("/{}", def.name),
            source: Some(source),
        }
    }
}
