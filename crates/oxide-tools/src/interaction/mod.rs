//! 用户交互工具
//!
//! 定义与用户交互的数据结构和 trait。

pub mod ask;

pub use ask::{AskUserQuestionArgs, AskUserQuestionOutput, InteractionHandler, QuestionOption, QuestionType};
