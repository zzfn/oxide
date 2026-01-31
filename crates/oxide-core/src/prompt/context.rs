//! 运行时上下文
//!
//! 包含环境信息，用于动态注入到提示词中。

use std::path::PathBuf;

/// 运行时上下文
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// 工作目录
    pub working_dir: PathBuf,
    /// 是否是 Git 仓库
    pub is_git_repo: bool,
    /// 操作系统平台
    pub platform: String,
    /// 操作系统版本
    pub os_version: String,
    /// 当前日期
    pub today: String,
    /// 模型名称
    pub model_name: String,
    /// 模型 ID
    pub model_id: String,
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_default(),
            is_git_repo: false,
            platform: std::env::consts::OS.to_string(),
            os_version: String::new(),
            today: chrono::Local::now().format("%Y-%m-%d").to_string(),
            model_name: "Claude".to_string(),
            model_id: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

impl RuntimeContext {
    /// 从当前环境创建上下文
    pub fn from_env(working_dir: PathBuf) -> Self {
        let is_git_repo = working_dir.join(".git").exists();
        let os_version = Self::get_os_version();

        Self {
            working_dir,
            is_git_repo,
            platform: std::env::consts::OS.to_string(),
            os_version,
            today: chrono::Local::now().format("%Y-%m-%d").to_string(),
            ..Default::default()
        }
    }

    /// 设置模型信息
    pub fn with_model(mut self, name: &str, id: &str) -> Self {
        self.model_name = name.to_string();
        self.model_id = id.to_string();
        self
    }

    /// 生成环境信息段落
    pub fn to_env_section(&self) -> String {
        format!(
            r#"Here is useful information about the environment you are running in:
<env>
Working directory: {}
Is directory a git repo: {}
Platform: {}
OS Version: {}
Today's date: {}
</env>
You are powered by the model named {}. The exact model ID is {}."#,
            self.working_dir.display(),
            if self.is_git_repo { "Yes" } else { "No" },
            self.platform,
            self.os_version,
            self.today,
            self.model_name,
            self.model_id
        )
    }

    /// 获取操作系统版本
    fn get_os_version() -> String {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| format!("Darwin {}", s.trim()))
                .unwrap_or_else(|| "Darwin".to_string())
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| format!("Linux {}", s.trim()))
                .unwrap_or_else(|| "Linux".to_string())
        }
        #[cfg(target_os = "windows")]
        {
            "Windows".to_string()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            "Unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_context() {
        let ctx = RuntimeContext::default();
        assert!(!ctx.platform.is_empty());
        assert!(!ctx.today.is_empty());
    }

    #[test]
    fn test_from_env() {
        let ctx = RuntimeContext::from_env(std::env::current_dir().unwrap());
        assert!(!ctx.platform.is_empty());
    }

    #[test]
    fn test_with_model() {
        let ctx = RuntimeContext::default().with_model("Claude Opus", "claude-opus-4");
        assert_eq!(ctx.model_name, "Claude Opus");
        assert_eq!(ctx.model_id, "claude-opus-4");
    }

    #[test]
    fn test_env_section() {
        let ctx = RuntimeContext {
            working_dir: PathBuf::from("/test/project"),
            is_git_repo: true,
            platform: "darwin".to_string(),
            os_version: "Darwin 24.0".to_string(),
            today: "2025-01-31".to_string(),
            model_name: "Claude".to_string(),
            model_id: "claude-sonnet-4".to_string(),
        };

        let section = ctx.to_env_section();
        assert!(section.contains("/test/project"));
        assert!(section.contains("git repo: Yes"));
        assert!(section.contains("darwin"));
    }
}
