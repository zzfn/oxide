//! REPL 模块
//!
//! 提供交互式命令行界面，使用 ratatui inline viewport 渲染多行输入框。
//! 流式输出通过 ratatui 的 `insert_before` API 插入到 viewport 上方，
//! 输入框始终保持可见。

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
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
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

        enable_raw_mode()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(5),
            },
        )?;
        terminal.hide_cursor()?;

        let result = self.run_with_persistent_terminal(&mut terminal, &mut editor, &completer, &mut completion_items, &mut completion_index, &mut scroll_offset).await;

        terminal.show_cursor().ok();
        terminal.clear().ok();
        disable_raw_mode()?;

        result
    }

    async fn run_with_persistent_terminal(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &mut LineEditor,
        completer: &OxideCompleter,
        completion_items: &mut Vec<completer::Suggestion>,
        completion_index: &mut Option<usize>,
        scroll_offset: &mut usize,
    ) -> Result<()> {
        loop {
            let (mode, message_count, token_info) = {
                let state = self.state.read().await;
                let msg_count = state.conversation.messages.len();
                let token_str = format!(
                    "{}↑ {}↓",
                    state.token_usage.input_tokens,
                    state.token_usage.output_tokens
                );
                (state.mode, msg_count, token_str)
            };

            let signal = self.input_loop(
                terminal,
                editor,
                completer,
                mode,
                completion_items,
                completion_index,
                scroll_offset,
                message_count,
                &token_info,
            ).await;

            match signal {
                InputSignal::Line(line) => {
                    Self::insert_text_before(terminal, &format!(
                        "\x1b[32m>\x1b[0m {}",
                        if line.contains('\n') { line.replace('\n', "\n    ") } else { line.clone() }
                    ));

                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    {
                        let mut state = self.state.write().await;
                        state.reset_ctrl_c();
                    }

                    let should_exit = self.process_line(line, terminal, editor, mode, message_count, &token_info).await?;
                    if should_exit {
                        break;
                    }
                }
                InputSignal::CtrlC => {
                    let should_exit = {
                        let mut state = self.state.write().await;
                        if state.is_processing {
                            state.end_processing();
                            Self::insert_text_before(terminal, "\x1b[33mWarning:\x1b[0m 操作已取消");
                            false
                        } else {
                            state.increment_ctrl_c()
                        }
                    };

                    if should_exit {
                        Self::insert_text_before(terminal, "\x1b[34mInfo:\x1b[0m 再见！");
                        break;
                    } else {
                        Self::insert_text_before(terminal, "\x1b[34mInfo:\x1b[0m 再按一次 Ctrl+C 退出");
                    }
                    editor.clear();
                }
                InputSignal::CtrlD => {
                    Self::insert_text_before(terminal, "\x1b[34mInfo:\x1b[0m 再见！");
                    break;
                }
                InputSignal::ClearScreen => {
                    let _ = terminal.clear();
                }
                InputSignal::TabComplete => {}
            }
        }

        Ok(())
    }

    /// 使用 ratatui insert_before 在 viewport 上方插入文本行。
    /// 支持 ANSI 颜色码，会解析为 ratatui Span 正确渲染（含宽字符处理）。
    fn insert_text_before(
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        text: &str,
    ) {
        let text_lines: Vec<&str> = text.split('\n').collect();
        let height = text_lines.len().max(1) as u16;
        let ratatui_lines: Vec<Line<'static>> = text_lines
            .iter()
            .map(|&s| Line::from(parse_ansi_to_spans(s)))
            .collect();

        let _ = terminal.insert_before(height, |buf| {
            let area = buf.area;
            Paragraph::new(ratatui_lines).render(area, buf);
        });
    }

    /// 使用 ratatui insert_before 在 viewport 上方插入 ratatui Lines
    fn insert_lines_before(
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        lines: Vec<Line<'static>>,
    ) {
        let height = lines.len().max(1) as u16;
        let _ = terminal.insert_before(height, |buf| {
            let area = buf.area;
            let paragraph = Paragraph::new(lines);
            paragraph.render(area, buf);
        });
    }

    /// 输入循环：使用持久化的 ratatui terminal 渲染输入框并处理按键
    async fn input_loop(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &mut LineEditor,
        completer: &OxideCompleter,
        mode: crate::app::CliMode,
        completion_items: &mut Vec<completer::Suggestion>,
        completion_index: &mut Option<usize>,
        scroll_offset: &mut usize,
        message_count: usize,
        token_info: &str,
    ) -> InputSignal {
        let mut current_height: u16 = 0;

        self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
        self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);

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
                        self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                        self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                        continue;
                    }

                    let has_menu = !completion_items.is_empty();

                    if let Event::Key(key_ev) = &ev {
                        let code = key_ev.code;
                        let mods = key_ev.modifiers;

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

                            self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                            self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                            continue;
                        }

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
                                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
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
                                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
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
                                    self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }

                    if has_menu {
                        completion_items.clear();
                        *completion_index = None;
                        *scroll_offset = 0;
                    }

                    if let Some(signal) = editor.handle_event(&ev) {
                        break signal;
                    }

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

                    self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, token_info);
                }
            }
        };

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

    fn ensure_terminal_height(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        current_height: &mut u16,
        editor: &LineEditor,
        mode: crate::app::CliMode,
        completions: &[completer::Suggestion],
        selected: Option<usize>,
        scroll_offset: usize,
        message_count: usize,
        token_info: &str,
    ) {
        let term_width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
        let needed = InputBox::new(editor, mode)
            .completions(completions, selected, scroll_offset)
            .message_count(message_count)
            .token_info(token_info)
            .required_height(term_width);

        if *current_height != needed {
            let _ = terminal.clear();
            let _ = terminal.show_cursor();

            let backend = CrosstermBackend::new(io::stdout());
            match Terminal::with_options(
                backend,
                TerminalOptions {
                    viewport: Viewport::Inline(needed),
                },
            ) {
                Ok(mut new_terminal) => {
                    let _ = new_terminal.hide_cursor();
                    *terminal = new_terminal;
                    *current_height = needed;
                }
                Err(_) => {
                    // Cursor position read can fail when EventStream is active.
                    // Keep using the current terminal with existing height.
                    let _ = terminal.hide_cursor();
                }
            }
        }
    }

    fn redraw(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &LineEditor,
        mode: crate::app::CliMode,
        completions: &[completer::Suggestion],
        selected: Option<usize>,
        scroll_offset: usize,
        message_count: usize,
        token_info: &str,
    ) {
        let _ = terminal.draw(|frame| {
            let area = frame.area();
            let widget = InputBox::new(editor, mode)
                .completions(completions, selected, scroll_offset)
                .message_count(message_count)
                .token_info(token_info);
            frame.render_widget(widget, area);
        });
    }

    /// 处理一行用户输入：命令或 AI 对话。
    /// 返回 true 表示应退出 REPL。
    async fn process_line(
        &mut self,
        line: &str,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &mut LineEditor,
        mode: crate::app::CliMode,
        message_count: usize,
        token_info: &str,
    ) -> Result<bool> {
        if CommandRegistry::is_command(line) {
            disable_raw_mode().ok();
            let result = self.commands.execute(line, self.state.clone()).await;
            enable_raw_mode().ok();

            match result {
                Ok(CommandResult::Exit) => {
                    Self::insert_text_before(terminal, "\x1b[34mInfo:\x1b[0m 再见！");
                    return Ok(true);
                }
                Ok(CommandResult::Message(msg)) => {
                    for msg_line in msg.lines() {
                        Self::insert_text_before(terminal, msg_line);
                    }
                }
                Ok(CommandResult::Continue) => {}
                Err(e) => {
                    Self::insert_text_before(terminal, &format!("\x1b[31mError:\x1b[0m 命令执行失败: {}", e));
                }
            }
            return Ok(false);
        }

        let working_dir = {
            let state = self.state.read().await;
            state.working_dir.clone()
        };
        let (file_refs, cleaned_input) = parse_file_references(line, &working_dir);

        let actual_input = if file_refs.is_empty() {
            line.to_string()
        } else {
            let mut context_parts = Vec::new();
            for fref in &file_refs {
                if let Some(content) = fref.read_content() {
                    Self::insert_text_before(terminal, &format!("\x1b[34mInfo:\x1b[0m 已引用: {}", fref.path.display()));
                    context_parts.push(content);
                } else {
                    Self::insert_text_before(terminal, &format!("\x1b[33mWarning:\x1b[0m 无法读取: {}", fref.path.display()));
                }
            }
            if context_parts.is_empty() {
                cleaned_input
            } else {
                format!("{}\n\n{}", context_parts.join("\n\n"), cleaned_input)
            }
        };

        // handle_user_input may return pending input typed during streaming
        let mut pending = self.handle_user_input(&actual_input, terminal, editor, mode, message_count, token_info.to_string()).await?;

        // Process any pending input that was typed during streaming
        while let Some(queued_line) = pending.take() {
            let queued_trimmed = queued_line.trim();
            if queued_trimmed.is_empty() {
                break;
            }
            Self::insert_text_before(terminal, &format!(
                "\x1b[32m>\x1b[0m {}",
                if queued_line.contains('\n') { queued_line.replace('\n', "\n    ") } else { queued_line.clone() }
            ));

            // Re-fetch state for updated counts
            let (mode, message_count, token_info) = {
                let state = self.state.read().await;
                let msg_count = state.conversation.messages.len();
                let token_str = format!("{}↑ {}↓", state.token_usage.input_tokens, state.token_usage.output_tokens);
                (state.mode, msg_count, token_str)
            };

            if CommandRegistry::is_command(queued_trimmed) {
                disable_raw_mode().ok();
                let result = self.commands.execute(queued_trimmed, self.state.clone()).await;
                enable_raw_mode().ok();
                match result {
                    Ok(CommandResult::Exit) => {
                        Self::insert_text_before(terminal, "\x1b[34mInfo:\x1b[0m 再见！");
                        return Ok(true);
                    }
                    Ok(CommandResult::Message(msg)) => {
                        for msg_line in msg.lines() {
                            Self::insert_text_before(terminal, msg_line);
                        }
                    }
                    Ok(CommandResult::Continue) => {}
                    Err(e) => {
                        Self::insert_text_before(terminal, &format!("\x1b[31mError:\x1b[0m 命令执行失败: {}", e));
                    }
                }
                break;
            }

            pending = self.handle_user_input(queued_trimmed, terminal, editor, mode, message_count, token_info).await?;
        }

        Ok(false)
    }

    /// 处理用户输入（发送给 AI）
    ///
    /// 使用 tokio::select! 同时轮询 AI 流和键盘事件，
    /// 输入框在流式输出期间保持可编辑。
    async fn handle_user_input(
        &mut self,
        input: &str,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &mut LineEditor,
        mode: crate::app::CliMode,
        message_count: usize,
        token_info: String,
    ) -> Result<Option<String>> {
        {
            let mut state = self.state.write().await;
            state.start_processing();
        }

        let rig_provider = {
            let state = self.state.read().await;
            state.rig_provider.clone()
        };

        let Some(provider) = rig_provider else {
            Self::insert_text_before(terminal, "\x1b[31mError:\x1b[0m AI Provider 未初始化。请设置 ANTHROPIC_API_KEY 环境变量。");
            let mut state = self.state.write().await;
            state.end_processing();
            return Ok(None);
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

        let agent_runner = crate::agent::RigAgentRunner::new_with_config(working_dir.clone(), permissions_config)
            .with_multi_progress(self.renderer.multi_progress().clone())
            .with_system_prompt(&system_prompt)
            .with_statusline(self.renderer.statusline_mut().clone());

        Self::insert_lines_before(terminal, vec![
            Line::from(""),
            Line::from(Span::styled("Assistant", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))),
            Line::from(""),
        ]);
        self.redraw(terminal, editor, mode, &[], None, 0, message_count, &token_info);

        use rig::streaming::StreamingPrompt;
        use rig::agent::MultiTurnStreamItem;
        use rig::streaming::StreamedAssistantContent;
        use futures::StreamExt;
        use crossterm::event::EventStream;

        let agent = {
            let mut tools = oxide_tools::rig_tools::OxideToolSetBuilder::new(working_dir.clone())
                .task_manager(agent_runner.task_manager())
                .permission_manager(agent_runner.permission_manager())
                .build_boxed();

            let ask_tool = oxide_tools::rig_tools::RigAskUserQuestionTool::new();
            ask_tool.set_handler(Arc::new(crate::interaction::CliInteractionHandler::new())).await;
            tools.push(Box::new(oxide_tools::rig_tools::ToolWrapper::new(ask_tool)));

            provider.create_agent_with_tools(Some(&system_prompt), tools)
        };

        let prompt = if chat_history.is_empty() {
            input.to_string()
        } else {
            let history_context = self.format_chat_history(&chat_history);
            format!("{}\n\n用户: {}", history_context, input)
        };

        let mut stream = agent.stream_prompt(&prompt).multi_turn(10).await;
        let mut event_stream = EventStream::new();

        let mut full_response = String::new();
        let mut line_buffer = String::new();
        let mut is_thinking = false;
        let mut pending_input: Option<String> = None;
        let mut stream_done = false;

        loop {
            if stream_done {
                break;
            }

            tokio::select! {
                biased;

                // Poll keyboard events (higher priority)
                maybe_event = event_stream.next() => {
                    if let Some(Ok(ev)) = maybe_event {
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
                            self.redraw(terminal, editor, mode, &[], None, 0, message_count, &token_info);
                            continue;
                        }

                        // Ctrl+C during streaming: cancel
                        if let Event::Key(key_ev) = &ev {
                            if key_ev.code == KeyCode::Char('c') && key_ev.modifiers.contains(KeyModifiers::CONTROL) {
                                Self::insert_lines_before(terminal, vec![
                                    Line::from(""),
                                    Line::from(Span::styled("操作已取消", Style::default().fg(Color::Yellow))),
                                ]);
                                // Don't break here; drop the stream to stop it
                                stream_done = true;
                                continue;
                            }
                        }

                        // Forward key events to editor
                        if let Some(signal) = editor.handle_event(&ev) {
                            match signal {
                                InputSignal::Line(line) => {
                                    pending_input = Some(line);
                                }
                                _ => {}
                            }
                        }

                        // Redraw input box to reflect typing
                        self.redraw(terminal, editor, mode, &[], None, 0, message_count, &token_info);
                    }
                }

                // Poll AI stream
                maybe_chunk = stream.next() => {
                    match maybe_chunk {
                        Some(Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text)))) => {
                            if is_thinking {
                                if !line_buffer.is_empty() {
                                    Self::insert_text_before(terminal, &line_buffer);
                                    line_buffer.clear();
                                }
                                Self::insert_text_before(terminal, "");
                                is_thinking = false;
                            }

                            full_response.push_str(&text.text);

                            for ch in text.text.chars() {
                                if ch == '\n' {
                                    Self::insert_text_before(terminal, &line_buffer);
                                    line_buffer.clear();
                                } else {
                                    line_buffer.push(ch);
                                }
                            }
                            // Redraw input box after inserting lines above
                            self.redraw(terminal, editor, mode, &[], None, 0, message_count, &token_info);
                        }
                        Some(Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning)))) => {
                            if !is_thinking {
                                if !line_buffer.is_empty() {
                                    Self::insert_text_before(terminal, &line_buffer);
                                    line_buffer.clear();
                                }
                                Self::insert_lines_before(terminal, vec![
                                    Line::from(""),
                                    Line::from(Span::styled("💭 思考中:", Style::default().fg(Color::DarkGray))),
                                ]);
                                is_thinking = true;
                            }

                            for r in reasoning.reasoning {
                                Self::insert_lines_before(terminal, vec![
                                    Line::from(Span::styled(format!("  {}", r), Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM))),
                                ]);
                            }
                            self.redraw(terminal, editor, mode, &[], None, 0, message_count, &token_info);
                        }
                        Some(Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall(tool_call)))) => {
                            if !line_buffer.is_empty() {
                                Self::insert_text_before(terminal, &line_buffer);
                                line_buffer.clear();
                            }
                            Self::insert_lines_before(terminal, vec![
                                Line::from(""),
                                Line::from(vec![
                                    Span::styled("⏺ ", Style::default().fg(Color::Green)),
                                    Span::styled(
                                        format!("{}({:?})", tool_call.function.name, tool_call.function.arguments),
                                        Style::default(),
                                    ),
                                ]),
                            ]);
                            self.redraw(terminal, editor, mode, &[], None, 0, message_count, &token_info);
                        }
                        Some(Ok(MultiTurnStreamItem::FinalResponse(final_res))) => {
                            full_response = final_res.response().to_string();
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            Self::insert_lines_before(terminal, vec![
                                Line::from(""),
                                Line::from(Span::styled(format!("错误: {}", e), Style::default().fg(Color::Red))),
                            ]);

                            let mut state = self.state.write().await;
                            state.conversation.messages.pop();
                            state.end_processing();

                            return Err(anyhow::anyhow!("流式输出错误: {}", e));
                        }
                        None => {
                            // Stream ended
                            stream_done = true;
                        }
                    }
                }
            }
        }

        if !line_buffer.is_empty() {
            Self::insert_text_before(terminal, &line_buffer);
        }
        Self::insert_text_before(terminal, "");
        self.redraw(terminal, editor, mode, &[], None, 0, message_count, &token_info);

        self.renderer.statusline_mut().finish();

        {
            let mut state = self.state.write().await;
            if !full_response.is_empty() {
                state.conversation.add_message(oxide_core::types::Message::text(
                    oxide_core::types::Role::Assistant,
                    &full_response,
                ));
            }
            let input_tokens = utils::count_tokens(input) as u64;
            let output_tokens = utils::count_tokens(&full_response) as u64;
            state.update_token_usage(input_tokens, output_tokens, 0);
            state.end_processing();
        }

        Ok(pending_input)
    }

    fn format_chat_history(&self, messages: &[oxide_core::types::Message]) -> String {
        let mut context = String::new();

        for msg in messages {
            let content_text = msg.content.iter()
                .filter_map(|block| {
                    if let oxide_core::types::ContentBlock::Text { text } = block {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            match msg.role {
                oxide_core::types::Role::User => {
                    context.push_str(&format!("用户: {}\n\n", content_text));
                }
                oxide_core::types::Role::Assistant => {
                    context.push_str(&format!("助手: {}\n\n", content_text));
                }
                _ => {}
            }
        }

        context
    }
}

/// 解析 ANSI 转义码文本为 ratatui Span 列表。
fn parse_ansi_to_spans(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut style = Style::default();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                let mut params = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == ';' {
                        params.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars.peek() == Some(&'m') {
                    chars.next();
                    if !current.is_empty() {
                        spans.push(Span::styled(std::mem::take(&mut current), style));
                    }
                    style = apply_ansi_params(&params, style);
                }
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        spans.push(Span::styled(current, style));
    }

    spans
}

fn apply_ansi_params(params: &str, mut style: Style) -> Style {
    if params.is_empty() || params == "0" {
        return Style::default();
    }
    for code in params.split(';') {
        match code {
            "0" => style = Style::default(),
            "1" => style = style.add_modifier(Modifier::BOLD),
            "2" => style = style.add_modifier(Modifier::DIM),
            "31" => style = style.fg(Color::Red),
            "32" => style = style.fg(Color::Green),
            "33" => style = style.fg(Color::Yellow),
            "34" => style = style.fg(Color::Blue),
            "35" => style = style.fg(Color::Magenta),
            "36" => style = style.fg(Color::Cyan),
            _ => {}
        }
    }
    style
}
