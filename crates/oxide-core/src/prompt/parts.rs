//! 提示词片段定义
//!
//! 使用 rust-embed 嵌入提示词文件。

use rust_embed::Embed;

/// 嵌入的提示词资源
#[derive(Embed)]
#[folder = "prompts/"]
pub struct Prompts;

impl Prompts {
    /// 获取指定路径的提示词内容
    pub fn get_content(path: &str) -> Option<String> {
        Self::get(path).map(|f| String::from_utf8_lossy(&f.data).into_owned())
    }

    /// 获取所有系统提示词文件名
    pub fn list_system_prompts() -> Vec<String> {
        Self::iter()
            .filter(|p| p.starts_with("system/"))
            .map(|p| p.to_string())
            .collect()
    }
}

/// 提示词片段类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptPart {
    /// 核心行为指令
    Core,
    /// 语气风格
    Tone,
    /// 专业客观性
    Objectivity,
    /// 任务管理
    TaskManagement,
    /// 工具使用策略
    ToolPolicy,
    /// Git 操作指南
    GitOperations,
    /// 执行任务指南
    DoingTasks,
    /// 安全指令
    Security,
    /// 自定义片段
    Custom(String),
}

impl PromptPart {
    /// 获取片段对应的文件路径
    pub fn file_path(&self) -> Option<&'static str> {
        match self {
            Self::Core => Some("system/core.md"),
            Self::Tone => Some("system/tone.md"),
            Self::Objectivity => Some("system/objectivity.md"),
            Self::TaskManagement => Some("system/task-management.md"),
            Self::ToolPolicy => Some("system/tool-policy.md"),
            Self::GitOperations => Some("system/git-operations.md"),
            Self::DoingTasks => Some("system/doing-tasks.md"),
            Self::Security => Some("system/security.md"),
            Self::Custom(_) => None,
        }
    }

    /// 获取片段内容
    pub fn content(&self) -> String {
        match self {
            Self::Custom(content) => content.clone(),
            _ => self
                .file_path()
                .and_then(Prompts::get_content)
                .unwrap_or_default(),
        }
    }

    /// 获取所有标准片段（按推荐顺序）
    pub fn standard_parts() -> Vec<Self> {
        vec![
            Self::Core,
            Self::Tone,
            Self::Objectivity,
            Self::Security,
            Self::TaskManagement,
            Self::DoingTasks,
            Self::ToolPolicy,
            Self::GitOperations,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_embed() {
        // 验证嵌入的文件存在
        assert!(Prompts::get("system/core.md").is_some());
        assert!(Prompts::get("system/tone.md").is_some());
    }

    #[test]
    fn test_prompt_part_content() {
        let core = PromptPart::Core;
        let content = core.content();
        assert!(!content.is_empty());
        assert!(content.contains("interactive CLI tool"));
    }

    #[test]
    fn test_custom_part() {
        let custom = PromptPart::Custom("Custom instructions".to_string());
        assert_eq!(custom.content(), "Custom instructions");
        assert!(custom.file_path().is_none());
    }

    #[test]
    fn test_list_system_prompts() {
        let prompts = Prompts::list_system_prompts();
        assert!(!prompts.is_empty());
        assert!(prompts.iter().any(|p| p.contains("core.md")));
    }

    #[test]
    fn test_standard_parts() {
        let parts = PromptPart::standard_parts();
        assert!(parts.contains(&PromptPart::Core));
        assert!(parts.contains(&PromptPart::Security));
    }
}
