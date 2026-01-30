//! CLI 用户交互处理

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use oxide_tools::interaction::ask::{
    AskUserQuestionArgs, AskUserQuestionOutput, InteractionHandler, QuestionType,
};

/// CLI 交互处理器
pub struct CliInteractionHandler;

impl CliInteractionHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl InteractionHandler for CliInteractionHandler {
    async fn ask_question(&self, args: &AskUserQuestionArgs) -> Result<AskUserQuestionOutput> {
        let theme = ColorfulTheme::default();

        match args.question_type {
            QuestionType::Single => {
                let mut items: Vec<String> = args
                    .options
                    .iter()
                    .map(|opt| {
                        let mut label = opt.label.clone();
                        if opt.recommended {
                            label = format!("✓ {} (推荐)", label);
                        }
                        if let Some(desc) = &opt.description {
                            label = format!("{} - {}", label, desc);
                        }
                        label
                    })
                    .collect();

                if args.allow_custom {
                    items.push("自定义输入...".to_string());
                }

                let selection = Select::with_theme(&theme)
                    .with_prompt(&args.question)
                    .items(&items)
                    .default(0)
                    .interact()?;

                if args.allow_custom && selection == args.options.len() {
                    let custom: String = Input::with_theme(&theme)
                        .with_prompt("请输入自定义答案")
                        .interact_text()?;
                    Ok(AskUserQuestionOutput {
                        answers: vec![custom],
                        is_custom: true,
                    })
                } else {
                    Ok(AskUserQuestionOutput {
                        answers: vec![args.options[selection].label.clone()],
                        is_custom: false,
                    })
                }
            }
            QuestionType::Multiple => {
                let items: Vec<String> = args
                    .options
                    .iter()
                    .map(|opt| {
                        let mut label = opt.label.clone();
                        if opt.recommended {
                            label = format!("✓ {} (推荐)", label);
                        }
                        if let Some(desc) = &opt.description {
                            label = format!("{} - {}", label, desc);
                        }
                        label
                    })
                    .collect();

                let defaults: Vec<bool> = args.options.iter().map(|opt| opt.recommended).collect();

                let selections = MultiSelect::with_theme(&theme)
                    .with_prompt(&args.question)
                    .items(&items)
                    .defaults(&defaults)
                    .interact()?;

                let mut answers: Vec<String> = selections
                    .iter()
                    .map(|&idx| args.options[idx].label.clone())
                    .collect();

                if args.allow_custom {
                    let add_custom: bool = dialoguer::Confirm::with_theme(&theme)
                        .with_prompt("是否添加自定义选项？")
                        .default(false)
                        .interact()?;

                    if add_custom {
                        let custom: String = Input::with_theme(&theme)
                            .with_prompt("请输入自定义答案")
                            .interact_text()?;
                        answers.push(custom);
                        return Ok(AskUserQuestionOutput {
                            answers,
                            is_custom: true,
                        });
                    }
                }

                Ok(AskUserQuestionOutput {
                    answers,
                    is_custom: false,
                })
            }
        }
    }
}
