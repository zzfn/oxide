//! AskUserQuestion 工具
//!
//! 向用户提问并收集答案。

#![allow(dead_code)]

use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};

#[cfg(feature = "cli")]
use dialoguer::{MultiSelect, Select};

/// 问题选项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuestionOption {
    /// 选项标签
    pub label: String,

    /// 选项描述
    pub description: String,
}

/// 单个问题
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Question {
    /// 问题内容
    pub question: String,

    /// 短标题(用于显示)
    pub header: String,

    /// 选项列表
    pub options: Vec<QuestionOption>,

    /// 是否允许多选
    #[serde(default)]
    pub multi_select: bool,
}

/// AskUserQuestion 工具输入参数
#[derive(Deserialize)]
pub struct AskUserQuestionArgs {
    /// 问题列表
    pub questions: Vec<Question>,
}

/// 单个问题的答案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// 问题 header
    pub question_header: String,

    /// 选择的选项索引(单选)或索引列表(多选)
    pub selected: serde_json::Value,

    /// 是否有答案
    pub has_answer: bool,
}

/// AskUserQuestion 工具输出
#[derive(Serialize, Deserialize, Debug)]
pub struct AskUserQuestionOutput {
    /// 所有的答案映射 (header -> answer)
    pub answers: HashMap<String, serde_json::Value>,

    /// 是否成功
    pub success: bool,

    /// 消息
    pub message: String,

    /// 问题总数
    pub total_questions: usize,

    /// 已回答的问题数
    pub answered_questions: usize,
}

/// AskUserQuestion 工具
#[derive(Deserialize, Serialize)]
pub struct AskUserQuestionTool;

impl AskUserQuestionTool {
    /// 显示单个问题并收集答案 (CLI 模式)
    #[allow(dead_code)]
    fn ask_question_cli(question: &Question) -> Result<Answer, FileToolError> {
        println!();
        println!("{}", "═".repeat(80).bright_black());
        println!("{}", question.header.bright_cyan().bold());
        println!("{}", "═".repeat(80).bright_black());
        println!("{}", question.question.white());
        println!();

        for (index, option) in question.options.iter().enumerate() {
            println!(
                "  {} {}. {} - {}",
                "›".bright_cyan(),
                (index + 1).to_string().bright_yellow(),
                option.label.bright_white(),
                option.description.dimmed()
            );
        }
        println!();

        // 读取用户输入
        let prompt = if question.multi_select {
            format!(
                "{} (多个选项用逗号分隔, 例如: 1,3): ",
                "选择".bright_green()
            )
        } else {
            format!("{} (输入数字): ", "选择".bright_green())
        };

        print!("{}", prompt);
        io::stdout().flush().map_err(|e| FileToolError::Io(e))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| FileToolError::Io(e))?;

        let input = input.trim();

        if input.is_empty() {
            // 用户没有输入,返回空答案
            return Ok(Answer {
                question_header: question.header.clone(),
                selected: serde_json::json!(null),
                has_answer: false,
            });
        }

        // 解析用户输入
        if question.multi_select {
            // 多选: 解析逗号分隔的数字
            let selected_indices: Vec<usize> = input
                .split(',')
                .map(|s| s.trim().parse::<usize>().unwrap_or(0))
                .filter(|&i| i >= 1 && i <= question.options.len())
                .map(|i| i - 1) // 转换为 0-based 索引
                .collect();

            let selected_labels: Vec<String> = selected_indices
                .iter()
                .filter_map(|&i| question.options.get(i).map(|o| o.label.clone()))
                .collect();

            Ok(Answer {
                question_header: question.header.clone(),
                selected: serde_json::json!(selected_labels),
                has_answer: !selected_labels.is_empty(),
            })
        } else {
            // 单选: 解析单个数字
            match input.trim().parse::<usize>() {
                Ok(choice) if choice >= 1 && choice <= question.options.len() => {
                    let index = choice - 1;
                    let selected_label = &question.options[index].label;

                    Ok(Answer {
                        question_header: question.header.clone(),
                        selected: serde_json::json!(selected_label),
                        has_answer: true,
                    })
                }
                _ => Ok(Answer {
                    question_header: question.header.clone(),
                    selected: serde_json::json!(null),
                    has_answer: false,
                }),
            }
        }
    }

    /// 显示单个问题并收集答案 (TUI 模式)
    #[cfg(feature = "cli")]
    fn ask_question_tui(question: &Question) -> Result<Answer, FileToolError> {
        // 构建选项显示文本
        let option_items: Vec<String> = question
            .options
            .iter()
            .map(|opt| format!("{} - {}", opt.label, opt.description))
            .collect();

        if question.multi_select {
            // 多选模式
            let selections = MultiSelect::new()
                .with_prompt(format!("{}\n{}", question.question, "(使用空格选择, 回车确认)"))
                .items(&option_items)
                .interact()
                .map_err(|e| FileToolError::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

            if selections.is_empty() {
                Ok(Answer {
                    question_header: question.header.clone(),
                    selected: serde_json::json!(null),
                    has_answer: false,
                })
            } else {
                let selected_labels: Vec<String> = selections
                    .iter()
                    .filter_map(|&i| question.options.get(i).map(|o| o.label.clone()))
                    .collect();

                Ok(Answer {
                    question_header: question.header.clone(),
                    selected: serde_json::json!(selected_labels),
                    has_answer: true,
                })
            }
        } else {
            // 单选模式
            let selection = Select::new()
                .with_prompt(question.question.clone())
                .items(&option_items)
                .interact_opt()
                .map_err(|e| FileToolError::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

            match selection {
                Some(index) => {
                    let selected_label = &question.options[index].label;
                    Ok(Answer {
                        question_header: question.header.clone(),
                        selected: serde_json::json!(selected_label),
                        has_answer: true,
                    })
                }
                None => Ok(Answer {
                    question_header: question.header.clone(),
                    selected: serde_json::json!(null),
                    has_answer: false,
                }),
            }
        }
    }

    /// 显示单个问题并收集答案 (自动选择模式)
    fn ask_question(question: &Question) -> Result<Answer, FileToolError> {
        // 检测是否在 TUI 环境中
        // 如果环境变量 OXIDE_TUI_MODE 设置为 "1"，则使用 TUI 模式
        let is_tui = std::env::var("OXIDE_TUI_MODE")
            .ok()
            .map(|v| v == "1")
            .unwrap_or(false);

        #[cfg(feature = "cli")]
        {
            if is_tui {
                return Self::ask_question_tui(question);
            }
        }

        // 默认使用 CLI 模式
        Self::ask_question_cli(question)
    }
}

impl Tool for AskUserQuestionTool {
    const NAME: &'static str = "ask_user_question";

    type Error = FileToolError;
    type Args = AskUserQuestionArgs;
    type Output = AskUserQuestionOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "ask_user_question".to_string(),
            description: "Ask the user questions and collect their answers. This tool pauses execution to present interactive questions to the user, collects their responses, and returns them. Each question can have multiple options and can be either single-select or multi-select.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "questions": {
                        "type": "array",
                        "description": "List of questions to ask the user",
                        "items": {
                            "type": "object",
                            "properties": {
                                "question": {
                                    "type": "string",
                                    "description": "The complete question text"
                                },
                                "header": {
                                    "type": "string",
                                    "description": "Short header/title for the question (max 12 chars recommended)"
                                },
                                "options": {
                                    "type": "array",
                                    "description": "List of answer options",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "label": {
                                                "type": "string",
                                                "description": "Short option label"
                                            },
                                            "description": {
                                                "type": "string",
                                                "description": "Detailed description of the option"
                                            }
                                        },
                                        "required": ["label", "description"]
                                    }
                                },
                                "multi_select": {
                                    "type": "boolean",
                                    "description": "Whether to allow multiple selections (default: false)"
                                }
                            },
                            "required": ["question", "header", "options", "multi_select"]
                        }
                    }
                },
                "required": ["questions"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let total = args.questions.len();
        let mut answers = HashMap::new();
        let mut answered = 0usize;

        println!();
        println!("{}", "╡ 需要用户输入 ╞".bright_yellow().bold());
        println!(
            "{}",
            "请回答以下问题以继续执行:"
                .bright_black()
                .dimmed()
        );

        for question in &args.questions {
            match Self::ask_question(question) {
                Ok(answer) => {
                    if answer.has_answer {
                        answered += 1;
                        answers.insert(question.header.clone(), answer.selected);
                    } else {
                        // 用户没有回答,插入 null
                        answers.insert(question.header.clone(), serde_json::json!(null));
                    }
                }
                Err(e) => {
                    // 出错时插入 null
                    eprintln!(
                        "{}",
                        format!("错误: 无法获取问题 '{}' 的答案: {}", question.header, e).red()
                    );
                    answers.insert(question.header.clone(), serde_json::json!(null));
                }
            }
        }

        println!();
        println!("{}", "═".repeat(80).bright_black());

        let success = answered > 0;
        let message = if success {
            format!("收集了 {}/{} 个问题的答案", answered, total)
        } else {
            "未收到任何有效答案".to_string()
        };

        Ok(AskUserQuestionOutput {
            answers,
            success,
            message,
            total_questions: total,
            answered_questions: answered,
        })
    }
}

/// AskUserQuestion 工具包装器
#[derive(Deserialize, Serialize)]
pub struct WrappedAskUserQuestionTool {
    inner: AskUserQuestionTool,
}

impl WrappedAskUserQuestionTool {
    pub fn new() -> Self {
        Self {
            inner: AskUserQuestionTool,
        }
    }
}

impl Default for WrappedAskUserQuestionTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for WrappedAskUserQuestionTool {
    const NAME: &'static str = "ask_user_question";

    type Error = FileToolError;
    type Args = <AskUserQuestionTool as Tool>::Args;
    type Output = <AskUserQuestionTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 直接调用内部工具,它已经包含了所有用户交互逻辑
        self.inner.call(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_option_serialization() {
        let option = QuestionOption {
            label: "选项1".to_string(),
            description: "这是选项1的描述".to_string(),
        };

        let json = serde_json::to_string(&option).unwrap();
        assert!(json.contains("label"));
        assert!(json.contains("description"));

        let deserialized: QuestionOption = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label, "选项1");
        assert_eq!(deserialized.description, "这是选项1的描述");
    }

    #[test]
    fn test_question_serialization() {
        let question = Question {
            question: "请选择一个选项".to_string(),
            header: "选择".to_string(),
            options: vec![
                QuestionOption {
                    label: "A".to_string(),
                    description: "选项A".to_string(),
                },
                QuestionOption {
                    label: "B".to_string(),
                    description: "选项B".to_string(),
                },
            ],
            multi_select: false,
        };

        let json = serde_json::to_string(&question).unwrap();
        let deserialized: Question = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.question, "请选择一个选项");
        assert_eq!(deserialized.header, "选择");
        assert_eq!(deserialized.options.len(), 2);
        assert_eq!(deserialized.multi_select, false);
    }

    #[test]
    fn test_ask_user_question_args_deserialization() {
        let json = r#"{
            "questions": [
                {
                    "question": "你最喜欢的编程语言是什么?",
                    "header": "语言",
                    "options": [
                        {"label": "Rust", "description": "安全且高性能"},
                        {"label": "Python", "description": "简洁易用"}
                    ],
                    "multi_select": false
                }
            ]
        }"#;

        let args: AskUserQuestionArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.questions.len(), 1);
        assert_eq!(args.questions[0].header, "语言");
        assert_eq!(args.questions[0].options.len(), 2);
    }

    #[test]
    fn test_ask_user_question_output_serialization() {
        let mut answers = HashMap::new();
        answers.insert("语言".to_string(), serde_json::json!("Rust"));

        let output = AskUserQuestionOutput {
            answers,
            success: true,
            message: "收集了 1/1 个问题的答案".to_string(),
            total_questions: 1,
            answered_questions: 1,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("answers"));
        assert!(json.contains("success"));

        let deserialized: AskUserQuestionOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_questions, 1);
        assert_eq!(deserialized.answered_questions, 1);
        assert!(deserialized.success);
    }

    #[test]
    fn test_answer_creation() {
        let answer = Answer {
            question_header: "测试".to_string(),
            selected: serde_json::json!("选项1"),
            has_answer: true,
        };

        assert_eq!(answer.question_header, "测试");
        assert!(answer.has_answer);
        assert_eq!(answer.selected, serde_json::json!("选项1"));
    }
}
