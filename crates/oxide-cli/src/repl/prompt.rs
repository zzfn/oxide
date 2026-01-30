//! 自定义提示符
//!
//! 根据当前模式显示不同的提示符样式。

use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};
use std::borrow::Cow;

use crate::app::{CliMode, SharedAppState};

/// Oxide 自定义提示符
pub struct OxidePrompt {
    /// 当前模式
    mode: CliMode,
}

impl OxidePrompt {
    /// 创建新的提示符
    pub fn new(mode: CliMode) -> Self {
        Self { mode }
    }

    /// 更新模式
    pub fn set_mode(&mut self, mode: CliMode) {
        self.mode = mode;
    }

    /// 从共享状态创建提示符
    pub async fn from_state(state: &SharedAppState) -> Self {
        let state = state.read().await;
        Self::new(state.mode)
    }

    /// 获取模式颜色
    fn mode_color(&self) -> &'static str {
        match self.mode {
            CliMode::Normal => "\x1b[32m", // 绿色
            CliMode::Fast => "\x1b[33m",   // 黄色
            CliMode::Plan => "\x1b[36m",   // 青色
        }
    }
}

impl Prompt for OxidePrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        let color = self.mode_color();
        let mode_char = self.mode.short_name();
        Cow::Owned(format!("{}[{}]\x1b[0m ", color, mode_char))
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("\x1b[32m>\x1b[0m ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("... ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

impl Default for OxidePrompt {
    fn default() -> Self {
        Self::new(CliMode::Normal)
    }
}
