use crate::tui::TuiEvent;
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Copy)]
pub enum Event {
    Input(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    _sender: UnboundedSender<Event>,
    pub receiver: mpsc::UnboundedReceiver<Event>,
    _handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_clone = sender.clone();
        let _handle = tokio::spawn(async move {
            let mut last_tick = std::time::Instant::now();
            let tick_duration = Duration::from_millis(tick_rate);

            loop {
                let timeout = tick_duration
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if crossterm::event::poll(timeout).expect("poll works") {
                    if let Ok(ev) = crossterm::event::read() {
                        match ev {
                            CrosstermEvent::Key(key) => {
                                if key.kind == event::KeyEventKind::Press {
                                    if sender_clone.send(Event::Input(key)).is_err() {
                                        return;
                                    }
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                if sender_clone.send(Event::Mouse(mouse)).is_err() {
                                    return;
                                }
                            }
                            CrosstermEvent::Resize(width, height) => {
                                if sender_clone.send(Event::Resize(width, height)).is_err() {
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if last_tick.elapsed() >= tick_duration {
                    if sender_clone.send(Event::Tick).is_err() {
                        return;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self {
            _sender: sender,
            receiver,
            _handle,
        }
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self._handle.abort();
    }
}

pub fn handle_key_event(key: KeyEvent, tui_sender: &UnboundedSender<TuiEvent>) -> Result<bool> {
    match key.code {
        // Ctrl+C: 退出
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            tui_sender.send(TuiEvent::Exit)?;
            return Ok(true);
        }
        // Enter: 发送消息
        KeyCode::Enter => {
            tui_sender.send(TuiEvent::SendMessage)?;
        }
        // ?: 显示帮助
        KeyCode::Char('?') => {
            tui_sender.send(TuiEvent::Command("/help".to_string()))?;
        }
        // /: 命令模式
        KeyCode::Char('/') => {
            tui_sender.send(TuiEvent::Command("/".to_string()))?;
        }
        // Ctrl+T: 切换工具面板
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            tui_sender.send(TuiEvent::Command("/toggle-tools".to_string()))?;
        }
        // Ctrl+P: 上一条历史记录
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            tui_sender.send(TuiEvent::NavigateHistoryPrev)?;
        }
        // Ctrl+N: 下一条历史记录
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            tui_sender.send(TuiEvent::NavigateHistoryNext)?;
        }
        // 上箭头: 上一条历史记录（当输入框为空时）
        KeyCode::Up => {
            tui_sender.send(TuiEvent::NavigateUp)?;
        }
        // 下箭头: 下一条历史记录（当输入框为空时）
        KeyCode::Down => {
            tui_sender.send(TuiEvent::NavigateDown)?;
        }
        // PageUp/PageDown: 滚动消息
        KeyCode::PageUp => {
            tui_sender.send(TuiEvent::PageUp)?;
        }
        KeyCode::PageDown => {
            tui_sender.send(TuiEvent::PageDown)?;
        }
        // Home: 滚动到顶部
        KeyCode::Home => {
            tui_sender.send(TuiEvent::ScrollToTop)?;
        }
        // End: 滚动到底部
        KeyCode::End => {
            tui_sender.send(TuiEvent::ScrollToBottom)?;
        }
        // Backspace/Delete: 删除字符
        KeyCode::Backspace => {
            tui_sender.send(TuiEvent::Backspace)?;
        }
        KeyCode::Delete => {
            tui_sender.send(TuiEvent::Backspace)?;
        }
        // 其他字符: 输入
        KeyCode::Char(c) => {
            tui_sender.send(TuiEvent::Input(c.to_string()))?;
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_mouse_event(mouse: MouseEvent, tui_sender: &UnboundedSender<TuiEvent>) -> Result<bool> {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            tui_sender.send(TuiEvent::NavigateUp)?;
        }
        MouseEventKind::ScrollDown => {
            tui_sender.send(TuiEvent::NavigateDown)?;
        }
        _ => {}
    }
    Ok(false)
}
