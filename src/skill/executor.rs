use crate::skill::Skill;
use anyhow::{bail, Result};
use regex::Regex;
use std::collections::HashMap;

/// Skill 执行器
pub struct SkillExecutor;

impl SkillExecutor {
    /// 执行 Skill 并返回渲染后的提示词
    pub fn execute(skill: &Skill, args_str: &str) -> Result<String> {
        // 解析命令行参数
        let parsed_args = Self::parse_args(skill, args_str)?;

        // 验证必需参数
        for arg in &skill.args {
            if arg.required && !parsed_args.contains_key(&arg.name) {
                bail!(
                    "Missing required argument: -{}\n  Description: {}",
                    arg.name,
                    arg.description
                );
            }
        }

        // 渲染模板
        let rendered = Self::render_template(&skill.template, &parsed_args)?;

        Ok(rendered)
    }

    /// 解析命令行参数
    /// 支持格式：-m "message" 或 --name "value"
    fn parse_args(skill: &Skill, args_str: &str) -> Result<HashMap<String, String>> {
        let mut args = HashMap::new();

        // 简单参数解析器：匹配 -key "value" 或 --key "value"
        let re = Regex::new(r#"(-\w+|--\w+)\s+"([^"]+)""#)?;
        for cap in re.captures_iter(args_str) {
            let key = cap[1].trim_start_matches('-').trim_start_matches('-').to_string();
            let value = cap[2].to_string();
            args.insert(key, value);
        }

        // 也支持不带引号的参数（单个词）
        let re_simple = Regex::new(r#"(-\w+|--\w+)\s+(\S+)"#)?;
        for cap in re_simple.captures_iter(args_str) {
            let key = cap[1].trim_start_matches('-').trim_start_matches('-').to_string();
            let value = cap[2].to_string();
            if !args.contains_key(&key) {
                args.insert(key, value);
            }
        }

        // 应用默认值
        for arg in &skill.args {
            if !args.contains_key(&arg.name) {
                if let Some(default) = &arg.default {
                    args.insert(arg.name.clone(), default.clone());
                }
            }
        }

        Ok(args)
    }

    /// 渲染模板（替换变量）
    fn render_template(template: &str, args: &HashMap<String, String>) -> Result<String> {
        let mut rendered = template.to_string();

        // 简单的 {{var}} 替换
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }

        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{SkillArg, SkillSource};

    #[test]
    fn test_parse_args_with_quotes() {
        let skill = Skill {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "Test {{m}}".to_string(),
            args: vec![SkillArg {
                name: "m".to_string(),
                description: "message".to_string(),
                required: false,
                default: None,
            }],
            source: SkillSource::BuiltIn,
        };

        let result = SkillExecutor::parse_args(&skill, r#"-m "Hello World""#);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.get("m"), Some(&"Hello World".to_string()));
    }

    #[test]
    fn test_parse_args_without_quotes() {
        let skill = Skill {
            name: "test".to_string(),
            description: "Test".to_string(),
            template: "Test {{m}}".to_string(),
            args: vec![SkillArg {
                name: "m".to_string(),
                description: "message".to_string(),
                required: false,
                default: None,
            }],
            source: SkillSource::BuiltIn,
        };

        let result = SkillExecutor::parse_args(&skill, "-m hello");
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.get("m"), Some(&"hello".to_string()));
    }

    #[test]
    fn test_template_rendering() {
        let template = "Hello {{name}}, your message is: {{message}}";
        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());
        args.insert("message".to_string(), "Test".to_string());

        let result = SkillExecutor::render_template(template, &args);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Hello Alice, your message is: Test"
        );
    }
}
