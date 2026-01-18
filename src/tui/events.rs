use crate::tui::TuiEvent;
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Copy)]
pub enum Event {
    Input(KeyEvent),
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
                        if let CrosstermEvent::Key(key) = ev {
                            if key.kind == event::KeyEventKind::Press {
                                if sender_clone.send(Event::Input(key)).is_err() {
                                    return;
                                }
                            }
                        } else if let CrosstermEvent::Resize(width, height) = ev {
                            if sender_clone.send(Event::Resize(width, height)).is_err() {
                                return;
                            }
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
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            tui_sender.send(TuiEvent::Exit)?;
            return Ok(true);
        }
        KeyCode::Enter => {
            tui_sender.send(TuiEvent::SendMessage)?;
        }
        KeyCode::Char('/') => {
            tui_sender.send(TuiEvent::Command("/".to_string()))?;
        }
        KeyCode::Up => {
            tui_sender.send(TuiEvent::NavigateUp)?;
        }
        KeyCode::Down => {
            tui_sender.send(TuiEvent::NavigateDown)?;
        }
        KeyCode::PageUp => {
            tui_sender.send(TuiEvent::PageUp)?;
        }
        KeyCode::PageDown => {
            tui_sender.send(TuiEvent::PageDown)?;
        }
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            tui_sender.send(TuiEvent::Command("/toggle-tools".to_string()))?;
        }
        KeyCode::Backspace => {}
        KeyCode::Char(c) => {
            tui_sender.send(TuiEvent::Input(c.to_string()))?;
        }
        _ => {}
    }
    Ok(false)
}
