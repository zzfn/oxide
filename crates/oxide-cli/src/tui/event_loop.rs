use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, event::EnableBracketedPaste, event::DisableBracketedPaste};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;
use tokio::sync::mpsc;

use super::app_event::{AppEvent, AppEventSender};
use super::approval_overlay::{ApprovalOverlay, ApprovalRequest, ApprovalResult};
use super::bottom_pane::{BottomPane, InputResult};
use super::chat_widget::ChatWidget;
use super::history_cell::HistoryCell;
use crate::app::SharedAppState;
use crate::commands::{CommandRegistry, CommandResult};
use crate::repl::completer::parse_file_references;

/// 初始化终端
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// 恢复终端
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste
    );
    let _ = terminal.show_cursor();
}

/// 主 TUI 入口
pub async fn run_tui(
    state: SharedAppState,
    commands: Arc<CommandRegistry>,
) -> Result<()> {
    let mut terminal = init_terminal()?;

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();
    let event_sender = AppEventSender::new(event_tx);

    let mut chat = ChatWidget::new();
    let mut bottom = BottomPane::new();

    // 初始化模式和状态
    {
        let app_state = state.read().await;
        bottom.set_mode(app_state.mode);
        bottom.set_token_info(format!(
            "{}↑ {}↓",
            app_state.token_usage.input_tokens,
            app_state.token_usage.output_tokens,
        ));
        bottom.set_message_count(app_state.conversation.messages.len());
    }

    let mut should_quit = false;
    let mut spinner_interval = tokio::time::interval(Duration::from_millis(80));

    loop {
        if should_quit {
            break;
        }

        // 绘制 UI
        terminal.draw(|frame| {
            let size = frame.area();
            let bottom_height = bottom.desired_height(size.width);
            let chunks = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(bottom_height),
            ])
            .split(size);

            chat.render(chunks[0], frame.buffer_mut());
            bottom.render(chunks[1], frame.buffer_mut());
        })?;

        // 事件处理（并发：终端事件 + app 事件 + spinner tick）
        tokio::select! {
            biased;

            // App events (from AI agent)
            Some(app_event) = event_rx.recv() => {
                match app_event {
                    AppEvent::StreamText(text) => {
                        chat.push_stream_text(&text);
                    }
                    AppEvent::StreamReasoning(text) => {
                        chat.push_cell(HistoryCell::Reasoning { text });
                    }
                    AppEvent::ToolCallBegin { name, arguments } => {
                        chat.begin_tool_call(name, arguments);
                        bottom.set_task_running(true);
                    }
                    AppEvent::ToolCallEnd { name, success, summary } => {
                        chat.end_tool_call(name, success, summary);
                    }
                    AppEvent::TurnComplete { response } => {
                        chat.end_stream(&response);
                        bottom.set_task_running(false);
                        // Update token info
                        let app_state = state.read().await;
                        bottom.set_token_info(format!(
                            "{}↑ {}↓",
                            app_state.token_usage.input_tokens,
                            app_state.token_usage.output_tokens,
                        ));
                        bottom.set_message_count(app_state.conversation.messages.len());
                    }
                    AppEvent::StreamError(msg) => {
                        chat.push_cell(HistoryCell::Error { message: msg });
                        bottom.set_task_running(false);
                    }
                    AppEvent::ApprovalRequest { id, tool_name, description } => {
                        let overlay = ApprovalOverlay::new(ApprovalRequest {
                            id: id.clone(),
                            tool_name,
                            description,
                        });
                        chat.push_approval(overlay);
                    }
                    AppEvent::InsertHistoryCell(cell) => {
                        chat.push_cell(*cell);
                    }
                    AppEvent::Redraw => {}
                    AppEvent::Quit => {
                        should_quit = true;
                    }
                    _ => {}
                }
            }

            // Spinner tick
            _ = spinner_interval.tick() => {
                chat.tick();
                bottom.tick_spinner();
            }

            // Terminal events
            result = tokio::task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(16)).unwrap_or(false) {
                    event::read().ok()
                } else {
                    None
                }
            }) => {
                if let Ok(Some(ev)) = result {
                    // 只处理按键按下事件
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

                    // 审批弹窗优先处理
                    if chat.has_approval() {
                        if let Event::Key(key_ev) = &ev {
                            if let Some(approval) = chat.approval_mut() {
                                approval.handle_key_event(*key_ev);
                                if approval.is_complete() {
                                    let result = approval.result();
                                    let request_id = approval.request().id.clone();
                                    chat.take_approval();
                                    let approved = matches!(
                                        result,
                                        ApprovalResult::Approved | ApprovalResult::AlwaysApprove
                                    );
                                    event_sender.send(AppEvent::ApprovalResponse {
                                        id: request_id,
                                        approved,
                                    });
                                }
                            }
                        }
                        continue;
                    }

                    // 滚动
                    if let Event::Key(key_ev) = &ev {
                        match (key_ev.modifiers, key_ev.code) {
                            (KeyModifiers::NONE, KeyCode::PageUp) => {
                                let h = terminal.size()?.height as usize;
                                chat.scroll_up(h / 2);
                                continue;
                            }
                            (KeyModifiers::NONE, KeyCode::PageDown) => {
                                let h = terminal.size()?.height as usize;
                                chat.scroll_down(h / 2);
                                continue;
                            }
                            (KeyModifiers::SHIFT, KeyCode::Up) => {
                                chat.scroll_up(3);
                                continue;
                            }
                            (KeyModifiers::SHIFT, KeyCode::Down) => {
                                chat.scroll_down(3);
                                continue;
                            }
                            _ => {}
                        }
                    }

                    // 底部面板处理
                    let input_result = bottom.handle_event(&ev);
                    match input_result {
                        InputResult::Submit(text) => {
                            let text = text.trim().to_string();
                            if text.is_empty() {
                                continue;
                            }

                            bottom.reset_ctrl_c();

                            // 用户消息 cell
                            chat.push_cell(HistoryCell::UserMessage { text: text.clone() });

                            // 命令处理
                            if CommandRegistry::is_command(&text) {
                                // 暂时退出 raw mode 执行命令
                                restore_terminal(&mut terminal);
                                let result = commands.execute(&text, state.clone()).await;
                                terminal = init_terminal()?;

                                match result {
                                    Ok(CommandResult::Exit) => {
                                        should_quit = true;
                                    }
                                    Ok(CommandResult::Message(msg)) => {
                                        chat.push_cell(HistoryCell::Info { message: msg });
                                    }
                                    Ok(CommandResult::Continue) => {}
                                    Err(e) => {
                                        chat.push_cell(HistoryCell::Error {
                                            message: format!("命令执行失败: {}", e),
                                        });
                                    }
                                }

                                // 刷新状态
                                let app_state = state.read().await;
                                bottom.set_mode(app_state.mode);
                                continue;
                            }

                            // AI 对话
                            let working_dir = {
                                let s = state.read().await;
                                s.working_dir.clone()
                            };

                            let (file_refs, cleaned_input) = parse_file_references(&text, &working_dir);
                            let actual_input = if file_refs.is_empty() {
                                text.clone()
                            } else {
                                let mut context_parts = Vec::new();
                                for fref in &file_refs {
                                    if let Some(content) = fref.read_content() {
                                        chat.push_cell(HistoryCell::Info {
                                            message: format!("已引用: {}", fref.path.display()),
                                        });
                                        context_parts.push(content);
                                    } else {
                                        chat.push_cell(HistoryCell::Warning {
                                            message: format!("无法读取: {}", fref.path.display()),
                                        });
                                    }
                                }
                                if context_parts.is_empty() {
                                    cleaned_input
                                } else {
                                    format!("{}\n\n{}", context_parts.join("\n\n"), cleaned_input)
                                }
                            };

                            // 开始流式输出
                            let term_width = terminal.size().map(|s| s.width as usize).ok();
                            chat.begin_stream(term_width);
                            bottom.set_task_running(true);
                            bottom.update_status("Processing".to_string(), None);

                            // 启动 AI agent 任务
                            let state_clone = state.clone();
                            let event_sender_clone = event_sender.clone();
                            tokio::spawn(async move {
                                run_agent_turn(
                                    state_clone,
                                    actual_input,
                                    event_sender_clone,
                                ).await;
                            });
                        }
                        InputResult::Interrupt => {
                            let mut s = state.write().await;
                            if s.is_processing {
                                s.end_processing();
                                chat.push_cell(HistoryCell::Warning {
                                    message: "操作已取消".to_string(),
                                });
                                bottom.set_task_running(false);
                            }
                        }
                        InputResult::Quit => {
                            should_quit = true;
                        }
                        InputResult::None => {}
                    }
                }
            }
        }
    }

    restore_terminal(&mut terminal);
    Ok(())
}

/// 运行一次 AI agent 对话回合
async fn run_agent_turn(
    state: SharedAppState,
    input: String,
    event_sender: AppEventSender,
) {
    use rig::streaming::StreamingPrompt;
    use rig::agent::MultiTurnStreamItem;
    use rig::streaming::StreamedAssistantContent;
    use futures::StreamExt;

    {
        let mut s = state.write().await;
        s.start_processing();
        s.conversation.add_message(oxide_core::types::Message::text(
            oxide_core::types::Role::User,
            &input,
        ));
    }

    let (rig_provider, working_dir, system_prompt, _permissions_config) = {
        let s = state.read().await;
        let provider = s.rig_provider.clone();
        let wd = s.working_dir.clone();

        let context = oxide_core::prompt::RuntimeContext::from_env(wd.clone())
            .with_model("Claude", &s.config.model.default_model);
        let prompt = oxide_core::prompt::PromptBuilder::default_agent()
            .with_context(context)
            .with_user_instructions(&wd)
            .build();

        (provider, wd, prompt.system, s.config.permissions.clone())
    };

    let Some(provider) = rig_provider else {
        event_sender.send(AppEvent::StreamError(
            "AI Provider 未初始化。请设置 ANTHROPIC_API_KEY 环境变量。".to_string(),
        ));
        let mut s = state.write().await;
        s.conversation.messages.pop();
        s.end_processing();
        return;
    };

    let chat_history = {
        let s = state.read().await;
        s.conversation.messages.clone()
    };

    let agent = {
        let mut tools = oxide_tools::rig_tools::OxideToolSetBuilder::new(working_dir.clone())
            .build_boxed();

        let ask_tool = oxide_tools::rig_tools::RigAskUserQuestionTool::new();
        tools.push(Box::new(oxide_tools::rig_tools::ToolWrapper::new(ask_tool)));

        provider.create_agent_with_tools(Some(&system_prompt), tools)
    };

    let prompt_text = if chat_history.is_empty() {
        input.clone()
    } else {
        let mut ctx = String::new();
        for msg in &chat_history {
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
                oxide_core::types::Role::User => ctx.push_str(&format!("用户: {}\n\n", content_text)),
                oxide_core::types::Role::Assistant => ctx.push_str(&format!("助手: {}\n\n", content_text)),
                _ => {}
            }
        }
        format!("{}\n\n用户: {}", ctx, input)
    };

    let mut stream = agent.stream_prompt(&prompt_text).multi_turn(10).await;
    let mut full_response = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                full_response.push_str(&text.text);
                event_sender.send(AppEvent::StreamText(text.text));
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning))) => {
                for r in reasoning.reasoning {
                    event_sender.send(AppEvent::StreamReasoning(r));
                }
            }
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall(tool_call))) => {
                event_sender.send(AppEvent::ToolCallBegin {
                    name: tool_call.function.name.clone(),
                    arguments: format!("{:?}", tool_call.function.arguments),
                });
            }
            Ok(MultiTurnStreamItem::FinalResponse(final_res)) => {
                full_response = final_res.response().to_string();
            }
            Ok(_) => {}
            Err(e) => {
                event_sender.send(AppEvent::StreamError(format!("{}", e)));
                let mut s = state.write().await;
                s.conversation.messages.pop();
                s.end_processing();
                return;
            }
        }
    }

    // 更新状态
    {
        let mut s = state.write().await;
        if !full_response.is_empty() {
            s.conversation.add_message(oxide_core::types::Message::text(
                oxide_core::types::Role::Assistant,
                &full_response,
            ));
        }
        let input_tokens = crate::utils::count_tokens(&input) as u64;
        let output_tokens = crate::utils::count_tokens(&full_response) as u64;
        s.update_token_usage(input_tokens, output_tokens, 0);
        s.end_processing();
    }

    event_sender.send(AppEvent::TurnComplete {
        response: full_response,
    });
}
