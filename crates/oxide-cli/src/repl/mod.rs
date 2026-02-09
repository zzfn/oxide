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

        // 进入 raw mode 并创建持久化的 terminal（在整个会话期间保持）
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(5), // 初始高度：边框(2) + 内容(1) + 状态栏(1) + 余量(1)
            },
        )?;
        terminal.hide_cursor()?;

        let result = self.run_with_persistent_terminal(&mut terminal, &mut editor, &completer, &mut completion_items, &mut completion_index, &mut scroll_offset).await;

        // 清理：退出 raw mode 并恢复光标
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

            // 渲染输入框并处理输入事件（保持在 raw mode）
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
                        // 暂时退出 raw mode 执行命令
                        disable_raw_mode().ok();
                        
                        let result = self.commands.execute(line, self.state.clone()).await;
                        
                        // 重新进入 raw mode
                        enable_raw_mode().ok();
                        
                        match result {
                            Ok(CommandResult::Exit) => {
                                self.renderer.info("再见！");
                                break;
                            }
                            Ok(CommandResult::Message(msg)) => {
                                disable_raw_mode().ok();
                                self.renderer.markdown(&msg);
                                enable_raw_mode().ok();
                            }
                            Ok(CommandResult::Continue) => {}
                            Err(e) => {
                                disable_raw_mode().ok();
                                self.renderer.error(&format!("命令执行失败: {}", e));
                                enable_raw_mode().ok();
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
                            self.handle_user_input(line, terminal, editor, mode, message_count, token_info.to_string()).await?;
                        } else {
                            let mut context_parts = Vec::new();
                            
                            // 暂时退出 raw mode 显示文件引用信息
                            disable_raw_mode().ok();
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
                            enable_raw_mode().ok();

                            let augmented = if context_parts.is_empty() {
                                cleaned_input
                            } else {
                                format!(
                                    "{}\n\n{}",
                                    context_parts.join("\n\n"),
                                    cleaned_input
                                )
                            };
                            self.handle_user_input(&augmented, terminal, editor, mode, message_count, token_info.to_string()).await?;
                        }
                    }
                }
                InputSignal::CtrlC => {
                    let should_exit = {
                        let mut state = self.state.write().await;
                        if state.is_processing {
                            state.end_processing();
                            disable_raw_mode().ok();
                            self.renderer.warning("操作已取消");
                            enable_raw_mode().ok();
                            false
                        } else {
                            state.increment_ctrl_c()
                        }
                    };

                    if should_exit {
                        disable_raw_mode().ok();
                        self.renderer.info("再见！");
                        break;
                    } else {
                        disable_raw_mode().ok();
                        self.renderer.info("再按一次 Ctrl+C 退出");
                        enable_raw_mode().ok();
                    }
                    editor.clear();
                }
                InputSignal::CtrlD => {
                    disable_raw_mode().ok();
                    self.renderer.info("再见！");
                    break;
                }
                InputSignal::ClearScreen => {
                    disable_raw_mode().ok();
                    let mut stdout = io::stdout();
                    let _ = crossterm::execute!(
                        stdout,
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                        crossterm::cursor::MoveTo(0, 0)
                    );
                    enable_raw_mode().ok();
                }
                InputSignal::TabComplete => {}
            }
        }

        Ok(())
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

        self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
        self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);

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
                        self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
                        self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
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

                            self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
                            self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
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
                                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
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
                                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
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
                                    self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
                                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
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

                    self.ensure_terminal_height(terminal, &mut current_height, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
                    self.redraw(terminal, editor, mode, completion_items, *completion_index, *scroll_offset, message_count, &token_info);
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

    /// 动态调整 inline viewport 高度（如果需要）
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
            // 高度变化时需要重建 terminal
            let _ = terminal.clear();
            let _ = terminal.show_cursor();
            
            let backend = CrosstermBackend::new(io::stdout());
            let mut new_terminal = Terminal::with_options(
                backend,
                TerminalOptions {
                    viewport: Viewport::Inline(needed),
                },
            )
            .expect("failed to create inline terminal");
            let _ = new_terminal.hide_cursor();
            *terminal = new_terminal;
            *current_height = needed;
        }
    }

    /// 渲染输入框 widget
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

    /// 打印用户输入（提交后显示）
    /// 在 raw mode 下需要暂时退出以使用 MultiProgress
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
        
        // 暂时退出 raw mode 以打印
        disable_raw_mode().ok();
        let _ = self.renderer.multi_progress().println(
            format!("{}[{}]\x1b[0m \x1b[32m>\x1b[0m {}", color, mode_char, display)
        );
        enable_raw_mode().ok();
    }

    /// 在 raw mode 下打印到 viewport 上方（使用 ANSI 转义码）
    fn print_above_viewport_raw(&self, text: &str) {
        use std::io::Write;
        
        let mut stdout = io::stdout();
        
        // 使用 ANSI 转义码：
        // \x1b7 - 保存光标位置
        // \x1b[H - 移动到屏幕顶部
        // \x1b[L - 插入一行（将下方内容向下推）
        // 打印内容
        // \x1b8 - 恢复光标位置
        
        print!("\x1b7\x1b[H\x1b[L{}\r\x1b8", text);
        let _ = stdout.flush();
    }

    /// 在 raw mode 下重新渲染输入框
    fn redraw_input_box_raw(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &LineEditor,
        mode: crate::app::CliMode,
        message_count: usize,
        token_info: &str,
    ) {
        let _ = terminal.draw(|frame| {
            let area = frame.area();
            let widget = InputBox::new(editor, mode)
                .message_count(message_count)
                .token_info(token_info);
            frame.render_widget(widget, area);
        });
    }

    /// 处理用户输入（发送给 AI）
    /// 
    /// 在 raw mode 下使用 ANSI 转义码进行流式输出，输入框保持可见。
    async fn handle_user_input(
        &mut self,
        input: &str,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        editor: &LineEditor,
        mode: crate::app::CliMode,
        message_count: usize,
        token_info: String,
    ) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.start_processing();
        }

        // 保持在 raw mode，使用自定义输出
        let _viewport_height = terminal.size().map(|s| s.height).unwrap_or(5);

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

        // 创建 agent_runner（用于获取 task_manager 和 permission_manager）
        let agent_runner = crate::agent::RigAgentRunner::new_with_config(working_dir.clone(), permissions_config)
            .with_multi_progress(self.renderer.multi_progress().clone())
            .with_system_prompt(&system_prompt)
            .with_statusline(self.renderer.statusline_mut().clone());

        // 打印 Assistant 标题
        self.print_above_viewport_raw("\n\x1b[1;34mAssistant\x1b[0m\n");
        self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);

        // 创建自定义流式处理
        use rig::streaming::StreamingPrompt;
        use rig::agent::MultiTurnStreamItem;
        use rig::streaming::StreamedAssistantContent;
        use futures::StreamExt;

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

        // 构建提示
        let prompt = if chat_history.is_empty() {
            input.to_string()
        } else {
            let history_context = self.format_chat_history(&chat_history);
            format!("{}\n\n用户: {}", history_context, input)
        };

        // 打印 Assistant 标题
        self.print_above_viewport_raw("\n\x1b[1;34mAssistant\x1b[0m\n");
        self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);

        // 获取流式响应
        let mut stream = agent.stream_prompt(&prompt).multi_turn(10).await;
        
        let mut full_response = String::new();
        let mut line_buffer = String::new();
        let mut is_thinking = false;

        // 处理流式响应
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                    // 退出思考模式
                    if is_thinking {
                        if !line_buffer.is_empty() {
                            self.print_above_viewport_raw(&line_buffer);
                            line_buffer.clear();
                        }
                        self.print_above_viewport_raw("");
                        is_thinking = false;
                        self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);
                    }

                    full_response.push_str(&text.text);

                    // 按行缓冲输出
                    for ch in text.text.chars() {
                        if ch == '\n' {
                            self.print_above_viewport_raw(&line_buffer);
                            line_buffer.clear();
                            self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);
                        } else {
                            line_buffer.push(ch);
                        }
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning))) => {
                    if !is_thinking {
                        if !line_buffer.is_empty() {
                            self.print_above_viewport_raw(&line_buffer);
                            line_buffer.clear();
                        }
                        self.print_above_viewport_raw("\n💭 思考中:");
                        is_thinking = true;
                        self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);
                    }

                    for r in reasoning.reasoning {
                        self.print_above_viewport_raw(&format!("  \x1b[2m{}\x1b[0m", r));
                        self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall(tool_call))) => {
                    if !line_buffer.is_empty() {
                        self.print_above_viewport_raw(&line_buffer);
                        line_buffer.clear();
                    }
                    self.print_above_viewport_raw(&format!("\n\x1b[32m⏺\x1b[0m {}({:?})", tool_call.function.name, tool_call.function.arguments));
                    self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);
                }
                Ok(MultiTurnStreamItem::FinalResponse(final_res)) => {
                    full_response = final_res.response().to_string();
                }
                Ok(_) => {}
                Err(e) => {
                    self.print_above_viewport_raw(&format!("\n\x1b[31m错误: {}\x1b[0m", e));
                    self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);
                    
                    let mut state = self.state.write().await;
                    state.conversation.messages.pop();
                    state.end_processing();
                    
                    return Err(anyhow::anyhow!("流式输出错误: {}", e));
                }
            }
        }

        // 刷新最后的缓冲
        if !line_buffer.is_empty() {
            self.print_above_viewport_raw(&line_buffer);
        }
        self.print_above_viewport_raw("");
        self.redraw_input_box_raw(terminal, editor, mode, message_count, &token_info);

        let result: Result<String> = Ok(full_response);
        
        match result {
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
                // 在 raw mode 下不能直接 println，需要暂时退出
                disable_raw_mode().ok();
                println!();
                self.renderer.error(&format!("代理执行失败: {}", e));
                enable_raw_mode().ok();

                {
                    let mut state = self.state.write().await;
                    state.conversation.messages.pop();
                    state.end_processing();
                }
            }
        }

        Ok(())
    }

    /// 格式化聊天历史为上下文字符串
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
