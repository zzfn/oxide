//! Commit 消息规范验证器
//!
//! 验证 Git 提交消息是否符合 Conventional Commits 规范。

#![allow(dead_code)]

use colored::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Conventional Commits 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitType {
    /// 新功能
    Feat,

    /// Bug 修复
    Fix,

    /// 文档更改
    Docs,

    /// 代码格式化(不影响代码运行)
    Style,

    /// 代码重构(不是新功能也不是修复 Bug)
    Refactor,

    /// 添加测试
    Test,

    /// 构建过程或辅助工具的变动
    Chore,

    /// 性能优化
    Perf,

    /// 不影响代码含义的代码重构(注释、变量名等)
    Ci,

    /// 撤销之前的提交
    Revert,

    /// 构建系统或外部依赖的更改
    Build,
}

impl CommitType {
    /// 所有的提交类型
    pub fn all() -> &'static [CommitType] {
        &[
            CommitType::Feat,
            CommitType::Fix,
            CommitType::Docs,
            CommitType::Style,
            CommitType::Refactor,
            CommitType::Test,
            CommitType::Chore,
            CommitType::Perf,
            CommitType::Ci,
            CommitType::Revert,
            CommitType::Build,
        ]
    }

    /// 获取类型字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            CommitType::Feat => "feat",
            CommitType::Fix => "fix",
            CommitType::Docs => "docs",
            CommitType::Style => "style",
            CommitType::Refactor => "refactor",
            CommitType::Test => "test",
            CommitType::Chore => "chore",
            CommitType::Perf => "perf",
            CommitType::Ci => "ci",
            CommitType::Revert => "revert",
            CommitType::Build => "build",
        }
    }

    /// 获取类型的描述
    pub fn description(&self) -> &'static str {
        match self {
            CommitType::Feat => "新功能",
            CommitType::Fix => "Bug 修复",
            CommitType::Docs => "文档更改",
            CommitType::Style => "代码格式化",
            CommitType::Refactor => "代码重构",
            CommitType::Test => "添加测试",
            CommitType::Chore => "构建过程或辅助工具的变动",
            CommitType::Perf => "性能优化",
            CommitType::Ci => "CI 配置",
            CommitType::Revert => "撤销提交",
            CommitType::Build => "构建系统或依赖更改",
        }
    }

    /// 从字符串解析类型
    pub fn from_str(s: &str) -> Option<CommitType> {
        match s {
            "feat" => Some(CommitType::Feat),
            "fix" => Some(CommitType::Fix),
            "docs" => Some(CommitType::Docs),
            "style" => Some(CommitType::Style),
            "refactor" => Some(CommitType::Refactor),
            "test" => Some(CommitType::Test),
            "chore" => Some(CommitType::Chore),
            "perf" => Some(CommitType::Perf),
            "ci" => Some(CommitType::Ci),
            "revert" => Some(CommitType::Revert),
            "build" => Some(CommitType::Build),
            _ => None,
        }
    }
}

/// 提交消息验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub valid: bool,

    /// 错误消息(如果无效)
    pub errors: Vec<String>,

    /// 警告消息
    pub warnings: Vec<String>,

    /// 解析出的提交类型
    pub commit_type: Option<String>,

    /// 解析出的作用域
    pub scope: Option<String>,

    /// 是否是破坏性变更
    pub breaking: bool,
}

impl ValidationResult {
    /// 创建有效的验证结果
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
            commit_type: None,
            scope: None,
            breaking: false,
        }
    }

    /// 创建无效的验证结果
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: vec![],
            commit_type: None,
            scope: None,
            breaking: false,
        }
    }

    /// 添加警告
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// 设置提交信息
    pub fn with_info(mut self, commit_type: String, scope: Option<String>, breaking: bool) -> Self {
        self.commit_type = Some(commit_type);
        self.scope = scope;
        self.breaking = breaking;
        self
    }
}

/// Commit Linter
///
/// 验证提交消息是否符合 Conventional Commits 规范。
pub struct CommitLinter {
    /// 提交消息正则表达式
    pattern: Regex,
}

impl CommitLinter {
    /// 创建新的 Commit Linter
    pub fn new() -> Result<Self, String> {
        // Conventional Commits 规范:
        // <type>[optional scope]: <description>
        //
        // [optional body]
        //
        // [optional footer(s)]
        let pattern = Regex::new(
            r"^(?P<type>[a-z]+)(\((?P<scope>[a-z0-9-]+)\))?(?P<breaking>!)?: (?P<desc>.+)",
        )
        .map_err(|e| format!("无法编译正则表达式: {}", e))?;

        Ok(Self { pattern })
    }

    /// 验证提交消息
    pub fn validate(&self, message: &str) -> ValidationResult {
        let lines: Vec<&str> = message.lines().collect();

        if lines.is_empty() {
            return ValidationResult::invalid(vec!["提交消息为空".to_string()]);
        }

        let first_line = lines[0];

        // 检查第一行长度(建议不超过 50 个字符)
        if first_line.len() > 50 {
            let mut result = ValidationResult::invalid(vec![]);
            result.warnings.push(format!(
                "第一行超过 50 个字符 (当前 {} 个)",
                first_line.len()
            ));
            result.valid = true; // 仍然是有效的,只是有警告
            return self.validate_with_result(first_line, result);
        }

        // 使用正则表达式验证格式
        if !self.pattern.is_match(first_line) {
            return ValidationResult::invalid(vec![
                "提交消息格式无效".to_string(),
                "期望格式: <type>[optional scope]: <description>".to_string(),
                format!(
                    "示例: feat: add new feature, fix(api): resolve bug",
                ),
                format!("支持的类型: {}", Self::supported_types()),
            ]);
        }

        // 解析并验证各个部分
        self.validate_with_result(first_line, ValidationResult::valid())
    }

    /// 使用已有的验证结果进行解析
    fn validate_with_result(&self, first_line: &str, mut result: ValidationResult) -> ValidationResult {
        let caps = match self.pattern.captures(first_line) {
            Some(c) => c,
            None => return result,
        };

        // 提取提交类型
        if let Some(type_match) = caps.name("type") {
            let type_str = type_match.as_str();
            if CommitType::from_str(type_str).is_none() {
                result.errors.push(format!("未知的提交类型: {}", type_str));
                result.valid = false;
                return result;
            }
            result.commit_type = Some(type_str.to_string());
        }

        // 提取作用域
        if let Some(scope_match) = caps.name("scope") {
            result.scope = Some(scope_match.as_str().to_string());
        }

        // 检查是否是破坏性变更
        let breaking = caps.name("breaking").is_some();
        result.breaking = breaking;

        // 提取描述
        if let Some(desc_match) = caps.name("desc") {
            let desc = desc_match.as_str();
            if desc.len() < 3 {
                result = result.with_warning("描述太短,建议至少 3 个字符".to_string());
            }
            // 检查描述是否以大写字母开头(建议)
            if let Some(first_char) = desc.chars().next() {
                if first_char.is_uppercase() {
                    result = result.with_warning("描述建议以小写字母开头".to_string());
                }
            }
            // 检查描述是否以句号结尾(不推荐)
            if desc.ends_with('.') {
                result = result.with_warning("描述不应以句号结尾".to_string());
            }
        }

        result
    }

    /// 获取支持的提交类型列表
    fn supported_types() -> String {
        CommitType::all()
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 格式化提交消息
    pub fn format_message(&self, commit_type: CommitType, scope: Option<String>, description: &str) -> String {
        match scope {
            Some(s) if !s.is_empty() => format!("{}({}): {}", commit_type.as_str(), s, description),
            _ => format!("{}: {}", commit_type.as_str(), description),
        }
    }

    /// 显示验证结果
    pub fn display_result(&self, result: &ValidationResult) {
        if result.valid {
            println!(
                "{} {}",
                "✓".bright_green(),
                "提交消息格式正确".bright_green()
            );

            if let Some(commit_type) = &result.commit_type {
                println!(
                    "  类型: {} - {}",
                    commit_type.bright_cyan(),
                    CommitType::from_str(commit_type)
                        .map(|t| t.description())
                        .unwrap_or("未知")
                        .bright_black()
                );
            }

            if let Some(scope) = &result.scope {
                println!("  作用域: {}", scope.bright_yellow());
            }

            if result.breaking {
                println!(
                    "  {}",
                    "⚠️ 破坏性变更".bright_yellow().bold()
                );
            }

            for warning in &result.warnings {
                println!(
                    "  {} {}",
                    "⚠️".bright_yellow(),
                    warning.bright_black()
                );
            }
        } else {
            println!(
                "{} {}",
                "✗".bright_red(),
                "提交消息格式无效".bright_red()
            );

            for error in &result.errors {
                println!("  {}", error.bright_red());
            }
        }
    }
}

impl Default for CommitLinter {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_type_from_str() {
        assert_eq!(CommitType::from_str("feat"), Some(CommitType::Feat));
        assert_eq!(CommitType::from_str("fix"), Some(CommitType::Fix));
        assert_eq!(CommitType::from_str("docs"), Some(CommitType::Docs));
        assert_eq!(CommitType::from_str("invalid"), None);
    }

    #[test]
    fn test_commit_type_description() {
        assert_eq!(CommitType::Feat.description(), "新功能");
        assert_eq!(CommitType::Fix.description(), "Bug 修复");
        assert_eq!(CommitType::Docs.description(), "文档更改");
    }

    #[test]
    fn test_valid_commit_messages() {
        let linter = CommitLinter::new().unwrap();

        let valid_messages = vec![
            "feat: add new feature",
            "fix(api): resolve bug in API",
            "docs: update readme",
            "refactor(core): improve performance",
            "test: add unit tests",
        ];

        for msg in valid_messages {
            let result = linter.validate(msg);
            assert!(
                result.valid,
                "消息 '{}' 应该是有效的, 错误: {:?}",
                msg, result.errors
            );
        }
    }

    #[test]
    fn test_invalid_commit_messages() {
        let linter = CommitLinter::new().unwrap();

        let invalid_messages = vec![
            "",
            "invalid message",
            "Add feature", // 缺少类型
            "feature: add new feature", // feature 不是有效的类型
        ];

        for msg in invalid_messages {
            let result = linter.validate(msg);
            assert!(
                !result.valid,
                "消息 '{}' 应该是无效的",
                msg
            );
        }
    }

    #[test]
    fn test_breaking_change() {
        let linter = CommitLinter::new().unwrap();

        let result = linter.validate("feat!: breaking API change");
        assert!(result.valid);
        assert!(result.breaking);
    }

    #[test]
    fn test_scope_parsing() {
        let linter = CommitLinter::new().unwrap();

        let result = linter.validate("feat(api): add new endpoint");
        assert!(result.valid);
        assert_eq!(result.scope, Some("api".to_string()));
    }

    #[test]
    fn test_format_message() {
        let linter = CommitLinter::new().unwrap();

        let msg1 = linter.format_message(CommitType::Feat, None, "add new feature");
        assert_eq!(msg1, "feat: add new feature");

        let msg2 = linter.format_message(CommitType::Fix, Some("api".to_string()), "fix bug");
        assert_eq!(msg2, "fix(api): fix bug");
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec!["Test warning".to_string()],
            commit_type: Some("feat".to_string()),
            scope: Some("api".to_string()),
            breaking: false,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("valid"));
        assert!(json.contains("commit_type"));

        let deserialized: ValidationResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.valid);
        assert_eq!(deserialized.commit_type, Some("feat".to_string()));
    }
}
