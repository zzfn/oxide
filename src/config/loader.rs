//! 配置加载器
//!
//! 支持多层次的配置系统：
//! 1. 全局配置：~/.oxide/config.toml
//! 2. 项目配置：.oxide/config.toml
//! 3. 项目指令：.oxide/CONFIG.md
//! 4. 环境变量（覆盖所有配置）

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::secret::Secret;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
#[allow(dead_code)]
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_STREAM_CHARS_PER_TICK: usize = 8;

/// 全局配置目录
fn global_config_dir() -> PathBuf {
    // 优先使用 XDG_CONFIG_HOME，其次使用 ~/.config
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("oxide")
    } else {
        dirs::home_dir()
            .expect("无法确定用户主目录")
            .join(".oxide")
    }
}

/// 项目配置目录
fn project_config_dir() -> PathBuf {
    PathBuf::from(".oxide")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlConfig {
    #[serde(default)]
    pub default: DefaultConfig,

    #[serde(default)]
    pub agent: Option<AgentConfigs>,

    #[serde(default)]
    pub theme: Option<ThemeConfig>,

    #[serde(default)]
    pub features: Option<FeaturesConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default)]
    pub model: Option<String>,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            model: None,
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

fn default_base_url() -> String {
    DEFAULT_BASE_URL.to_string()
}

fn default_max_tokens() -> u32 {
    DEFAULT_MAX_TOKENS
}

fn default_temperature() -> f32 {
    0.7
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigs {
    #[serde(default)]
    pub explore: Option<AgentConfig>,

    #[serde(default)]
    pub plan: Option<AgentConfig>,

    #[serde(default)]
    pub code_reviewer: Option<AgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub mode: String,

    #[serde(default)]
    pub custom_theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default)]
    pub enable_mcp: bool,

    #[serde(default)]
    pub enable_multimodal: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            enable_mcp: false,
            enable_multimodal: false,
        }
    }
}

impl Default for TomlConfig {
    fn default() -> Self {
        Self {
            default: DefaultConfig::default(),
            agent: None,
            theme: None,
            features: None,
        }
    }
}

/// 配置加载器
pub struct ConfigLoader {
    global_config_path: PathBuf,
    project_config_path: PathBuf,
    project_instructions_path: PathBuf,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            global_config_path: global_config_dir().join("config.toml"),
            project_config_path: project_config_dir().join("config.toml"),
            project_instructions_path: project_config_dir().join("CONFIG.md"),
        }
    }

    /// 加载 TOML 配置文件
    fn load_toml(&self, path: &Path) -> Result<TomlConfig> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件: {}", path.display()))?;

        let config: TomlConfig = toml::from_str(&content)
            .with_context(|| format!("解析 TOML 配置失败: {}", path.display()))?;

        Ok(config)
    }

    /// 读取项目指令（CONFIG.md）
    fn read_instructions(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .with_context(|| format!("无法读取项目指令: {}", path.display()))
    }

    /// 合并两个 TOML 配置（后者覆盖前者）
    pub fn merge_configs(mut base: TomlConfig, overlay: TomlConfig) -> TomlConfig {
        // 合并 default 配置
        if overlay.default.base_url != default_base_url() {
            base.default.base_url = overlay.default.base_url;
        }
        if overlay.default.model.is_some() {
            base.default.model = overlay.default.model;
        }
        if overlay.default.max_tokens != default_max_tokens() {
            base.default.max_tokens = overlay.default.max_tokens;
        }
        if overlay.default.temperature != default_temperature() {
            base.default.temperature = overlay.default.temperature;
        }

        // 合并 agent 配置
        if overlay.agent.is_some() {
            base.agent = overlay.agent;
        }

        // 合并 theme 配置
        if overlay.theme.is_some() {
            base.theme = overlay.theme;
        }

        // 合并 features 配置
        if overlay.features.is_some() {
            base.features = overlay.features;
        }

        base
    }

    /// 加载完整配置
    pub fn load(&self) -> Result<LoadedConfig> {
        let mut config = TomlConfig::default();
        let mut project_instructions = None;

        // 1. 加载全局配置
        if self.global_config_path.exists() {
            let global = self.load_toml(&self.global_config_path)?;
            config = global;
        }

        // 2. 加载项目配置（覆盖全局）
        if self.project_config_path.exists() {
            let project = self.load_toml(&self.project_config_path)?;
            config = Self::merge_configs(config, project);
        }

        // 3. 加载项目指令（系统提示词）
        if self.project_instructions_path.exists() {
            project_instructions = Some(self.read_instructions(&self.project_instructions_path)?);
        }

        // 4. 应用环境变量覆盖
        let auth_token = env::var("OXIDE_AUTH_TOKEN")
            .or_else(|_| env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| env::var("API_KEY"))
            .context("未找到 OXIDE_AUTH_TOKEN、ANTHROPIC_API_KEY 或 API_KEY 环境变量")?;
        let auth_token = Secret::new(auth_token);

        let base_url = env::var("OXIDE_BASE_URL")
            .or_else(|_| env::var("API_URL"))
            .unwrap_or_else(|_| config.default.base_url.clone());

        let model = env::var("MODEL_NAME")
            .or_else(|_| env::var("MODEL"))
            .ok()
            .or_else(|| config.default.model.clone());

        let max_tokens = env::var("MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(config.default.max_tokens);

        let temperature = env::var("TEMPERATURE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(config.default.temperature);

        let stream_chars_per_tick = env::var("STREAM_CHARS_PER_TICK")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_STREAM_CHARS_PER_TICK);

        Ok(LoadedConfig {
            base_url,
            auth_token,
            model,
            max_tokens,
            temperature,
            stream_chars_per_tick,
            project_instructions,
            agent_configs: config.agent,
            theme_config: config.theme,
            features_config: config.features.unwrap_or_default(),
        })
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 加载后的完整配置
#[derive(Clone)]
pub struct LoadedConfig {
    pub base_url: String,
    pub auth_token: Secret<String>,
    pub model: Option<String>,
    pub max_tokens: u32,
    #[allow(dead_code)]
    pub temperature: f32,
    pub stream_chars_per_tick: usize,
    #[allow(dead_code)]
    pub project_instructions: Option<String>,
    #[allow(dead_code)]
    pub agent_configs: Option<AgentConfigs>,
    #[allow(dead_code)]
    pub theme_config: Option<ThemeConfig>,
    #[allow(dead_code)]
    pub features_config: FeaturesConfig,
}

// 手动实现 Debug，防止 auth_token 泄露
impl std::fmt::Debug for LoadedConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedConfig")
            .field("base_url", &self.base_url)
            .field("auth_token", &self.auth_token) // Secret 的 Debug 实现会输出 "***"
            .field("model", &self.model)
            .field("max_tokens", &self.max_tokens)
            .field("temperature", &self.temperature)
            .field("stream_chars_per_tick", &self.stream_chars_per_tick)
            .field("project_instructions", &self.project_instructions)
            .field("agent_configs", &self.agent_configs)
            .field("theme_config", &self.theme_config)
            .field("features_config", &self.features_config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_toml_config_default() {
        let config = TomlConfig::default();
        assert_eq!(config.default.base_url, DEFAULT_BASE_URL);
        assert_eq!(config.default.max_tokens, DEFAULT_MAX_TOKENS);
        assert_eq!(config.default.temperature, 0.7);
    }

    #[test]
    fn test_merge_configs() {
        let mut base = TomlConfig::default();
        base.default.max_tokens = 1024;

        let mut overlay = TomlConfig::default();
        overlay.default.max_tokens = 2048;
        overlay.default.temperature = 0.5;

        let merged = ConfigLoader::merge_configs(base, overlay);
        assert_eq!(merged.default.max_tokens, 2048);
        assert_eq!(merged.default.temperature, 0.5);
    }

    #[test]
    fn test_load_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let content = r#"
[default]
base_url = "https://api.example.com"
model = "custom-model"
max_tokens = 2048
temperature = 0.5
"#;

        fs::write(&config_file, content).unwrap();

        let loader = ConfigLoader::new();
        let config = loader.load_toml(&config_file).unwrap();

        assert_eq!(config.default.base_url, "https://api.example.com");
        assert_eq!(config.default.model, Some("custom-model".to_string()));
        assert_eq!(config.default.max_tokens, 2048);
        assert_eq!(config.default.temperature, 0.5);
    }

    #[test]
    fn test_global_config_dir() {
        let dir = global_config_dir();
        assert!(dir.ends_with(".oxide") || dir.ends_with("oxide"));
    }

    #[test]
    fn test_project_config_dir() {
        let dir = project_config_dir();
        assert_eq!(dir, PathBuf::from(".oxide"));
    }
}
