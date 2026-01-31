//! 配置系统
//!
//! 支持从 `~/.oxide/config.toml` 加载配置，环境变量覆盖，以及会话状态持久化。

use crate::error::{OxideError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 获取 Oxide 主目录 (~/.oxide)
pub fn oxide_home() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".oxide"))
        .ok_or_else(|| OxideError::Config("无法获取用户主目录".into()))
}

/// 获取配置文件路径
pub fn config_path() -> Result<PathBuf> {
    Ok(oxide_home()?.join("config.toml"))
}

/// 获取历史文件路径
pub fn history_path() -> Result<PathBuf> {
    Ok(oxide_home()?.join("history.jsonl"))
}

/// 获取会话环境目录
pub fn session_env_dir() -> Result<PathBuf> {
    Ok(oxide_home()?.join("session-env"))
}

/// 获取任务目录
pub fn tasks_dir() -> Result<PathBuf> {
    Ok(oxide_home()?.join("tasks"))
}

/// 获取计划目录
pub fn plans_dir() -> Result<PathBuf> {
    Ok(oxide_home()?.join("plans"))
}

/// 获取全局 OXIDE.md 路径
pub fn global_oxide_md() -> Result<PathBuf> {
    Ok(oxide_home()?.join("OXIDE.md"))
}

/// 加载指令文件内容
pub fn load_instructions(project_dir: &PathBuf) -> Result<String> {
    let mut instructions = String::new();

    // 加载全局 OXIDE.md
    let global_path = global_oxide_md()?;
    if global_path.exists() {
        let content = std::fs::read_to_string(&global_path)?;
        if !content.trim().is_empty() {
            instructions.push_str(&format!(
                "--- CONTEXT ENTRY BEGIN ---\nContents of {} (global instructions):\n\n{}\n--- CONTEXT ENTRY END ---\n\n",
                global_path.display(),
                content
            ));
        }
    }

    // 加载项目级 OXIDE.md
    let project_path = project_dir.join("OXIDE.md");
    if project_path.exists() {
        let content = std::fs::read_to_string(&project_path)?;
        if !content.trim().is_empty() {
            instructions.push_str(&format!(
                "--- CONTEXT ENTRY BEGIN ---\nContents of {} (project instructions):\n\n{}\n--- CONTEXT ENTRY END ---\n\n",
                project_path.display(),
                content
            ));
        }
    }

    Ok(instructions)
}

/// Oxide 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// 模型配置
    #[serde(default)]
    pub model: ModelConfig,

    /// 权限配置
    #[serde(default)]
    pub permissions: PermissionsConfig,

    /// 行为配置
    #[serde(default)]
    pub behavior: BehaviorConfig,
}

impl Config {
    /// 从默认路径加载配置
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if path.exists() {
            Self::load_from(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// 加载配置（支持项目级覆盖）
    pub fn load_with_project(project_dir: &PathBuf) -> Result<Self> {
        let mut config = Self::load()?;
        let local_path = project_dir.join(".oxide/config.local.toml");
        if local_path.exists() {
            let local = Self::load_from(&local_path)?;
            config.merge(local);
        }
        Ok(config)
    }

    /// 合并配置（local 覆盖 self）
    fn merge(&mut self, local: Self) {
        if local.model.default_model != ModelConfig::default().default_model {
            self.model.default_model = local.model.default_model;
        }
        if local.model.temperature != ModelConfig::default().temperature {
            self.model.temperature = local.model.temperature;
        }
        if local.model.max_tokens != ModelConfig::default().max_tokens {
            self.model.max_tokens = local.model.max_tokens;
        }
        if !local.permissions.allow.is_empty() {
            self.permissions.allow = local.permissions.allow;
        }
        if !local.permissions.deny.is_empty() {
            self.permissions.deny = local.permissions.deny;
        }
        if local.behavior.thinking_mode != BehaviorConfig::default().thinking_mode {
            self.behavior.thinking_mode = local.behavior.thinking_mode;
        }
        if local.behavior.cleanup_period != BehaviorConfig::default().cleanup_period {
            self.behavior.cleanup_period = local.behavior.cleanup_period;
        }
        if local.behavior.auto_save != BehaviorConfig::default().auto_save {
            self.behavior.auto_save = local.behavior.auto_save;
        }
    }

    /// 从指定路径加载配置
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| OxideError::Config(format!("读取配置文件失败: {}", e)))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| OxideError::Config(format!("解析配置文件失败: {}", e)))?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        // 验证模型名称
        if self.model.default_model.is_empty() {
            return Err(OxideError::Config("模型名称不能为空".into()));
        }

        // 验证温度参数
        if self.model.temperature < 0.0 || self.model.temperature > 1.0 {
            return Err(OxideError::Config(format!(
                "温度参数必须在 0.0-1.0 之间，当前值: {}",
                self.model.temperature
            )));
        }

        // 验证 max_tokens
        if self.model.max_tokens == 0 {
            return Err(OxideError::Config("max_tokens 必须大于 0".into()));
        }

        // 验证权限配置冲突
        for tool in &self.permissions.allow {
            if self.permissions.deny.contains(tool) {
                return Err(OxideError::Config(format!(
                    "工具 '{}' 同时出现在 allow 和 deny 列表中",
                    tool
                )));
            }
        }

        Ok(())
    }

    /// 保存配置到默认路径
    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        self.save_to(&path)
    }

    /// 保存配置到指定路径
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| OxideError::Config(format!("序列化配置失败: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 初始化 Oxide 目录结构
    pub fn init_directories() -> Result<()> {
        let dirs = [
            oxide_home()?,
            session_env_dir()?,
            tasks_dir()?,
            plans_dir()?,
        ];
        for dir in dirs {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(())
    }
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 默认模型
    pub default_model: String,
    /// 温度参数
    pub temperature: f32,
    /// 最大 token 数
    pub max_tokens: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_model: "claude-sonnet-4-20250514".to_string(),
            temperature: 0.7,
            max_tokens: 8192,
        }
    }
}

/// 权限配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionsConfig {
    /// 允许的工具列表
    #[serde(default)]
    pub allow: Vec<String>,
    /// 禁止的工具列表
    #[serde(default)]
    pub deny: Vec<String>,
}

impl PermissionsConfig {
    /// 检查工具是否被允许
    pub fn is_allowed(&self, tool: &str) -> bool {
        // 如果在 deny 列表中，禁止
        if self.deny.contains(&tool.to_string()) {
            return false;
        }
        // 如果 allow 列表为空，默认允许所有
        // 如果 allow 列表不为空，只允许列表中的工具
        self.allow.is_empty() || self.allow.contains(&tool.to_string())
    }
}

/// 行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// 是否启用思考模式
    pub thinking_mode: bool,
    /// 会话清理周期（秒）
    pub cleanup_period: u64,
    /// 是否自动保存会话
    pub auto_save: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            thinking_mode: true,
            cleanup_period: 3600,
            auto_save: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions_allow_all() {
        let perms = PermissionsConfig::default();
        assert!(perms.is_allowed("Read"));
        assert!(perms.is_allowed("Bash"));
    }

    #[test]
    fn test_permissions_deny() {
        let perms = PermissionsConfig {
            allow: vec![],
            deny: vec!["Bash".to_string()],
        };
        assert!(perms.is_allowed("Read"));
        assert!(!perms.is_allowed("Bash"));
    }

    #[test]
    fn test_permissions_allow_list() {
        let perms = PermissionsConfig {
            allow: vec!["Read".to_string(), "Write".to_string()],
            deny: vec![],
        };
        assert!(perms.is_allowed("Read"));
        assert!(!perms.is_allowed("Bash"));
    }

    #[test]
    fn test_config_merge() {
        let mut global = Config {
            model: ModelConfig {
                default_model: "claude-sonnet-4".to_string(),
                temperature: 0.7,
                max_tokens: 8192,
            },
            permissions: PermissionsConfig::default(),
            behavior: BehaviorConfig::default(),
        };

        let local = Config {
            model: ModelConfig {
                default_model: "claude-opus-4".to_string(),
                temperature: 0.8,
                max_tokens: 8192,
            },
            permissions: PermissionsConfig {
                allow: vec!["Read".to_string(), "Write".to_string()],
                deny: vec![],
            },
            behavior: BehaviorConfig::default(),
        };

        global.merge(local);

        assert_eq!(global.model.default_model, "claude-opus-4");
        assert_eq!(global.model.temperature, 0.8);
        assert_eq!(global.permissions.allow.len(), 2);
    }

    #[test]
    fn test_load_instructions() {
        use std::fs;
        use std::env;

        let temp_dir = env::temp_dir().join("oxide_test_instructions");
        fs::create_dir_all(&temp_dir).unwrap();

        let project_md = temp_dir.join("OXIDE.md");
        fs::write(&project_md, "# Project Instructions\nTest content").unwrap();

        let result = load_instructions(&temp_dir).unwrap();

        assert!(result.contains("Project Instructions"));
        assert!(result.contains("Test content"));
        assert!(result.contains("CONTEXT ENTRY BEGIN"));

        fs::remove_dir_all(&temp_dir).ok();
    }
}
