//! REPL 模块
//!
//! 提供交互式命令行界面，使用 ratatui inline viewport 渲染多行输入框。

pub mod completer;
pub mod input;
pub mod input_box;
pub mod keybindings;
pub mod prompt;

pub use completer::OxideCompleter;
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
                        self.handle_user_input(line).await?;
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
    ) -> InputSignal {
        let term_width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
        let initial_height = InputBox::new(editor, mode)
            .completions(completion_items, *completion_index)
            .required_height(term_width);

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(initial_height),
            },
        )
        .expect("failed to create inline terminal");

        // 隐藏真实光标（我们在 widget 中渲染光标块）
        let _ = terminal.hide_cursor();

        // 初始渲染
        self.draw_input(&mut terminal, editor, mode, completion_items, *completion_index);

        let signal = loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    // Tab 补全
                    if matches!(
                        &ev,
                        Event::Key(crossterm::event::KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::NONE,
                            kind: crossterm::event::KeyEventKind::Press,
                            ..
                        })
                    ) {
                        if completion_items.is_empty() {
                            *completion_items = completer.complete(editor.buffer(), editor.cursor());
                            *completion_index = if completion_items.is_empty() {
                                None
                            } else {
                                Some(0)
                            };
                        } else if let Some(ref mut idx) = completion_index {
                            *idx = (*idx + 1) % completion_items.len();
                        }

                        if let Some(idx) = *completion_index {
                            let value = completion_items[idx].value.clone();
                            editor.apply_completion(&value);
                        }

                        self.draw_input(&mut terminal, editor, mode, completion_items, *completion_index);
                        continue;
                    }

                    // 窗口大小变化
                    if matches!(&ev, Event::Resize(..)) {
                        self.draw_input(&mut terminal, editor, mode, completion_items, *completion_index);
                        continue;
                    }

                    // 非 Tab 键清除补全
                    if !completion_items.is_empty() {
                        completion_items.clear();
                        *completion_index = None;
                    }

                    if let Some(signal) = editor.handle_event(&ev) {
                        break signal;
                    }

                    self.draw_input(&mut terminal, editor, mode, completion_items, *completion_index);
                }
            }
        };

        // 清除 inline viewport 区域
        let _ = terminal.clear();
        let _ = terminal.show_cursor();

        signal
    }

    /// 使用 ratatui 渲染输入框
    fn draw_input(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &LineEditor,
        mode: crate::app::CliMode,
        completions: &[completer::Suggestion],
        selected: Option<usize>,
    ) {
        let _ = terminal.draw(|frame| {
            let area = frame.area();
            let widget = InputBox::new(editor, mode)
                .completions(completions, selected);
            frame.render_widget(widget, area);
        });
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
