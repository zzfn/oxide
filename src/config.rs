use anyhow::{Context, Result};
use std::env;

const API_URL: &str = "https://api.deepseek.com/v1/chat/completions";
const MODEL: &str = "deepseek-chat";
const MAX_TOKENS: u32 = 4096;

#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
    pub max_tokens: u32,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();

        let api_key = env::var("DEEPSEEK_API_KEY")
            .context("未找到 DEEPSEEK_API_KEY 环境变量，请设置该变量后再运行程序")?;

        Ok(Config {
            api_key,
            api_url: API_URL.to_string(),
            model: MODEL.to_string(),
            max_tokens: MAX_TOKENS,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            anyhow::bail!("API Key 不能为空");
        }

        if !self.api_key.starts_with("sk-") {
            anyhow::bail!("API Key 格式不正确，应以 'sk-' 开头");
        }

        if self.api_key.len() < 20 {
            anyhow::bail!("API Key 长度不正确");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation_success() {
        let config = Config {
            api_key: "sk-test12345678901234567890".to_string(),
            api_url: API_URL.to_string(),
            model: MODEL.to_string(),
            max_tokens: MAX_TOKENS,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_key() {
        let config = Config {
            api_key: "".to_string(),
            api_url: API_URL.to_string(),
            model: MODEL.to_string(),
            max_tokens: MAX_TOKENS,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_prefix() {
        let config = Config {
            api_key: "invalid-key".to_string(),
            api_url: API_URL.to_string(),
            model: MODEL.to_string(),
            max_tokens: MAX_TOKENS,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_too_short() {
        let config = Config {
            api_key: "sk-short".to_string(),
            api_url: API_URL.to_string(),
            model: MODEL.to_string(),
            max_tokens: MAX_TOKENS,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_load_from_env() {
        dotenv::dotenv().ok();
        env::set_var("DEEPSEEK_API_KEY", "sk-test12345678901234567890");
        let result = Config::load();
        assert!(result.is_ok(), "Failed to load config: {:?}", result);
        let config = result.unwrap();
        assert_eq!(config.api_key, "sk-test12345678901234567890");
        env::remove_var("DEEPSEEK_API_KEY");
    }
}
