//! REPL 模块
//!
//! 提供交互式命令行界面，使用 ratatui inline viewport 渲染多行输入框。

pub mod completer;
pub mod input;
pub mod input_box;
pub mod keybindings;
pub mod prompt;

pub use completer::{parse_file_references, OxideCompleter};
pub use input::{InputSignal, LineEditor};
pub use input_box::InputBox;
pub use prompt::OxidePrompt;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::io;
use std::sync::Arc;
use std::time::Duration;

use crate::app::SharedAppState;
use crate::commands::{CommandRegistry, CommandResult};
use crate::render::Renderer;
use crate::utils;

/// REPL 主循环
pub struct Repl {
    state: SharedAppState,
    commands: Arc<CommandRegistry>,
    renderer: Renderer,
}

impl Repl {
    pub fn new(state: SharedAppState, commands: Arc<CommandRegistry>) -> Self {
        Self {
            state,
            commands,
            renderer: Renderer::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.renderer.welcome();

        let working_dir = {
            let state = self.state.read().await;
            state.working_dir.clone()
        };

        let mut editor = LineEditor::new();
        let completer = OxideCompleter::new(self.commands.clone(), working_dir);
        let mut completion_items: Vec<completer::Suggestion> = Vec::new();
        let mut completion_index: Option<usize> = None;
        let mut scroll_offset: usize = 0;

        loop {
            let mode = {
                let state = self.state.read().await;
                state.mode
            };

            // 进入 raw mode，使用 ratatui inline viewport 渲染输入框
            enable_raw_mode()?;
            let signal = self.input_loop(
                &mut editor,
                &completer,
                mode,
                &mut completion_items,
                &mut completion_index,
                &mut scroll_offset,
            ).await;
            disable_raw_mode()?;

            match signal {
                InputSignal::Line(line) => {
                    // 打印用户输入
                    self.print_user_input(&line, mode);

                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    {
                        let mut state = self.state.write().await;
                        state.reset_ctrl_c();
                    }

                    if CommandRegistry::is_command(line) {
                        match self.commands.execute(line, self.state.clone()).await {
                            Ok(CommandResult::Exit) => {
                                self.renderer.info("再见！");
                                break;
                            }
                            Ok(CommandResult::Message(msg)) => {
                                self.renderer.markdown(&msg);
                            }
                            Ok(CommandResult::Continue) => {}
                            Err(e) => {
                                self.renderer.error(&format!("命令执行失败: {}", e));
                            }
                        }
                    } else {
                        let working_dir = {
                            let state = self.state.read().await;
                            state.working_dir.clone()
                        };
                        let (file_refs, cleaned_input) =
                            parse_file_references(line, &working_dir);

                        if file_refs.is_empty() {
                            self.handle_user_input(line).await?;
                        } else {
                            let mut context_parts = Vec::new();
                            for fref in &file_refs {
                                if let Some(content) = fref.read_content() {
                                    self.renderer.info(&format!(
                                        "已引用: {}",
                                        fref.path.display()
                                    ));
                                    context_parts.push(content);
                                } else {
                                    self.renderer.warning(&format!(
                                        "无法读取: {}",
                                        fref.path.display()
                                    ));
                                }
                            }

                            let augmented = if context_parts.is_empty() {
                                cleaned_input
                            } else {
                                format!(
                                    "{}\n\n{}",
                                    context_parts.join("\n\n"),
                                    cleaned_input
                                )
                            };
                            self.handle_user_input(&augmented).await?;
                        }
                    }
                }
                InputSignal::CtrlC => {
                    let should_exit = {
                        let mut state = self.state.write().await;
                        if state.is_processing {
                            state.end_processing();
                            self.renderer.warning("操作已取消");
                            false
                        } else {
                            state.increment_ctrl_c()
                        }
                    };

                    if should_exit {
                        self.renderer.info("再见！");
                        break;
                    } else {
                        self.renderer.info("再按一次 Ctrl+C 退出");
                    }
                    editor.clear();
                }
                InputSignal::CtrlD => {
                    self.renderer.info("再见！");
                    break;
                }
                InputSignal::ClearScreen => {
                    let mut stdout = io::stdout();
                    let _ = crossterm::execute!(
                        stdout,
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                        crossterm::cursor::MoveTo(0, 0)
                    );
                }
                InputSignal::TabComplete => {}
            }
        }

        Ok(())
    }

    /// 输入循环：使用 ratatui inline viewport 渲染输入框并处理按键
    async fn input_loop(
        &self,
        editor: &mut LineEditor,
        completer: &OxideCompleter,
        mode: crate::app::CliMode,
        completion_items: &mut Vec<completer::Suggestion>,
        completion_index: &mut Option<usize>,
        scroll_offset: &mut usize,
    ) -> InputSignal {
        let mut current_height: u16 = 0;
        let mut terminal: Option<Terminal<CrosstermBackend<io::Stdout>>> = None;

        self.ensure_terminal(&mut terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset);
        self.redraw(&mut terminal, editor, mode, completion_items, *completion_index, *scroll_offset);

        let signal = loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    let is_key_press = matches!(
                        &ev,
                        Event::Key(crossterm::event::KeyEvent {
                            kind: crossterm::event::KeyEventKind::Press,
                            ..
                        })
                    );
                    if !is_key_press && !matches!(&ev, Event::Resize(..) | Event::Paste(_)) {
                        continue;
                    }

                    if matches!(&ev, Event::Resize(..)) {
                        self.ensure_terminal(&mut terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset);
                        self.redraw(&mut terminal, editor, mode, completion_items, *completion_index, *scroll_offset);
                        continue;
                    }

                    let has_menu = !completion_items.is_empty();

                    if let Event::Key(key_ev) = &ev {
                        let code = key_ev.code;
                        let mods = key_ev.modifiers;

                        // Tab: trigger or cycle completions
                        if code == KeyCode::Tab && mods == KeyModifiers::NONE {
                            if completion_items.is_empty() {
                                *completion_items =
                                    completer.complete(editor.buffer(), editor.cursor());
                                *completion_index = if completion_items.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                };
                                *scroll_offset = 0;
                            } else if let Some(ref mut idx) = completion_index {
                                *idx = (*idx + 1) % completion_items.len();
                                Self::adjust_scroll(scroll_offset, *idx, completion_items.len());
                            }

                            if let Some(idx) = *completion_index {
                                let value = completion_items[idx].value.clone();
                                editor.apply_completion(&value);
                            }

                            self.ensure_terminal(&mut terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset);
                            self.redraw(&mut terminal, editor, mode, completion_items, *completion_index, *scroll_offset);
                            continue;
                        }

                        // When any menu is open, handle navigation
                        if has_menu {
                            match (mods, code) {
                                (KeyModifiers::NONE, KeyCode::Up) => {
                                    if let Some(ref mut idx) = completion_index {
                                        if *idx > 0 {
                                            *idx -= 1;
                                        } else {
                                            *idx = completion_items.len() - 1;
                                        }
                                        Self::adjust_scroll(scroll_offset, *idx, completion_items.len());
                                    }
                                    if let Some(idx) = *completion_index {
                                        let value = completion_items[idx].value.clone();
                                        editor.apply_completion(&value);
                                    }
                                    self.redraw(&mut terminal, editor, mode, completion_items, *completion_index, *scroll_offset);
                                    continue;
                                }
                                (KeyModifiers::NONE, KeyCode::Down) => {
                                    if let Some(ref mut idx) = completion_index {
                                        *idx = (*idx + 1) % completion_items.len();
                                        Self::adjust_scroll(scroll_offset, *idx, completion_items.len());
                                    }
                                    if let Some(idx) = *completion_index {
                                        let value = completion_items[idx].value.clone();
                                        editor.apply_completion(&value);
                                    }
                                    self.redraw(&mut terminal, editor, mode, completion_items, *completion_index, *scroll_offset);
                                    continue;
                                }
                                (KeyModifiers::NONE, KeyCode::Enter) => {
                                    if let Some(idx) = *completion_index {
                                        let value = completion_items[idx].value.clone();
                                        editor.apply_completion(&value);
                                    }
                                    completion_items.clear();
                                    *completion_index = None;
                                    *scroll_offset = 0;
                                    let line = editor.buffer().to_string();
                                    editor.clear();
                                    break InputSignal::Line(line);
                                }
                                (KeyModifiers::NONE, KeyCode::Esc) => {
                                    completion_items.clear();
                                    *completion_index = None;
                                    *scroll_offset = 0;
                                    self.ensure_terminal(&mut terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset);
                                    self.redraw(&mut terminal, editor, mode, completion_items, *completion_index, *scroll_offset);
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }

                    // Clear completions on any other key
                    if has_menu {
                        completion_items.clear();
                        *completion_index = None;
                        *scroll_offset = 0;
                    }

                    if let Some(signal) = editor.handle_event(&ev) {
                        break signal;
                    }

                    // Auto-trigger completions when typing `/` or `@`
                    let buf = editor.buffer().to_string();
                    let pos = editor.cursor();
                    if pos > 0 {
                        let word_start = buf[..pos]
                            .rfind(|c: char| c.is_whitespace())
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        let current_word = &buf[word_start..pos];
                        if current_word.starts_with('/') || current_word.starts_with('@') {
                            *completion_items = completer.complete(&buf, pos);
                            *completion_index = if completion_items.is_empty() {
                                None
                            } else {
                                Some(0)
                            };
                            *scroll_offset = 0;
                        }
                    }

                    self.ensure_terminal(&mut terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset);
                    self.redraw(&mut terminal, editor, mode, completion_items, *completion_index, *scroll_offset);
                }
            }
        };

        if let Some(ref mut t) = terminal {
            let _ = t.clear();
            let _ = t.show_cursor();
        }

        signal
    }

    fn adjust_scroll(scroll_offset: &mut usize, selected: usize, total: usize) {
        let max_visible = total.min(input_box::MAX_VISIBLE_ITEMS);
        if selected < *scroll_offset {
            *scroll_offset = selected;
        } else if selected >= *scroll_offset + max_visible {
            *scroll_offset = selected + 1 - max_visible;
        }
    }

    /// Recreate the inline terminal if the required height changed
    fn ensure_terminal(
        &self,
        terminal: &mut Option<Terminal<CrosstermBackend<io::Stdout>>>,
        current_height: &mut u16,
        editor: &LineEditor,
        mode: crate::app::CliMode,
        completions: &[completer::Suggestion],
        selected: Option<usize>,
        scroll_offset: usize,
    ) {
        let term_width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
        let needed = InputBox::new(editor, mode)
            .completions(completions, selected, scroll_offset)
            .required_height(term_width);

        if *current_height != needed || terminal.is_none() {
            if let Some(ref mut t) = terminal {
                let _ = t.clear();
                let _ = t.show_cursor();
            }
            let backend = CrosstermBackend::new(io::stdout());
            let mut t = Terminal::with_options(
                backend,
                TerminalOptions {
                    viewport: Viewport::Inline(needed),
                },
            )
            .expect("failed to create inline terminal");
            let _ = t.hide_cursor();
            *terminal = Some(t);
            *current_height = needed;
        }
    }

    /// Render the input box widget
    fn redraw(
        &self,
        terminal: &mut Option<Terminal<CrosstermBackend<io::Stdout>>>,
        editor: &LineEditor,
        mode: crate::app::CliMode,
        completions: &[completer::Suggestion],
        selected: Option<usize>,
        scroll_offset: usize,
    ) {
        if let Some(ref mut t) = terminal {
            let _ = t.draw(|frame| {
                let area = frame.area();
                let widget = InputBox::new(editor, mode)
                    .completions(completions, selected, scroll_offset);
                frame.render_widget(widget, area);
            });
        }
    }

    /// 打印用户输入（提交后显示）
    fn print_user_input(&self, input: &str, mode: crate::app::CliMode) {
        let (mode_char, color) = match mode {
            crate::app::CliMode::Normal => ("N", "\x1b[32m"),
            crate::app::CliMode::Fast => ("F", "\x1b[33m"),
            crate::app::CliMode::Plan => ("P", "\x1b[36m"),
        };
        let display = if input.contains('\n') {
            input.replace('\n', "\n    ")
        } else {
            input.to_string()
        };
        println!("{}[{}]\x1b[0m \x1b[32m>\x1b[0m {}", color, mode_char, display);
    }

    /// 处理用户输入（发送给 AI）
    async fn handle_user_input(&mut self, input: &str) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.start_processing();
        }

        self.renderer.statusline_mut().start("Thinking");

        let rig_provider = {
            let state = self.state.read().await;
            state.rig_provider.clone()
        };

        let Some(provider) = rig_provider else {
            self.renderer.statusline_mut().clear();
            self.renderer.error("AI Provider 未初始化。请设置 ANTHROPIC_API_KEY 环境变量。");
            let mut state = self.state.write().await;
            state.end_processing();
            return Ok(());
        };

        {
            let mut state = self.state.write().await;
            state.conversation.add_message(oxide_core::types::Message::text(
                oxide_core::types::Role::User,
                input,
            ));
        }

        let (chat_history, working_dir) = {
            let state = self.state.read().await;
            (state.conversation.messages.clone(), state.working_dir.clone())
        };

        let (system_prompt, permissions_config) = {
            let state = self.state.read().await;

            let context = oxide_core::prompt::RuntimeContext::from_env(working_dir.clone())
                .with_model("Claude", &state.config.model.default_model);

            let prompt = oxide_core::prompt::PromptBuilder::default_agent()
                .with_context(context)
                .with_user_instructions(&working_dir)
                .build();

            (prompt.system, state.config.permissions.clone())
        };

        let agent_runner = crate::agent::RigAgentRunner::new_with_config(working_dir, permissions_config)
            .with_multi_progress(self.renderer.multi_progress().clone())
            .with_system_prompt(&system_prompt)
            .with_statusline(self.renderer.statusline_mut().clone());

        self.renderer.assistant_header();
        self.renderer.statusline_mut().update("Processing", 0);

        match agent_runner.run_stream(&provider, input, chat_history).await {
            Ok(response) => {
                self.renderer.statusline_mut().finish();

                {
                    let mut state = self.state.write().await;
                    state.conversation.add_message(oxide_core::types::Message::text(
                        oxide_core::types::Role::Assistant,
                        &response,
                    ));
                    let input_tokens = utils::count_tokens(input) as u64;
                    let output_tokens = utils::count_tokens(&response) as u64;
                    state.update_token_usage(input_tokens, output_tokens, 0);
                    state.end_processing();
                }
            }
            Err(e) => {
                self.renderer.statusline_mut().clear();
                println!();
                self.renderer.error(&format!("代理执行失败: {}", e));

                {
                    let mut state = self.state.write().await;
                    state.conversation.messages.pop();
                    state.end_processing();
                }
            }
        }

        Ok(())
    }
}
