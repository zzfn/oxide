use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Normal,
    Processing,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub msg_type: MessageType,
    pub content: String,
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TuiEvent {
    Input(String),
    SendMessage,
    Command(String),
    NavigateUp,
    NavigateDown,
    PageUp,
    PageDown,
    ScrollToTop,
    ScrollToBottom,
    Exit,
    Resize(u16, u16),
    Tick,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll_offset: usize,
    pub state: AppState,
    pub model: String,
    pub message_count: usize,
    pub event_sender: Option<UnboundedSender<TuiEvent>>,
    pub tool_status: Vec<(String, String)>,
    pub show_tool_panel: bool,
}

impl App {
    pub fn new(model: String) -> Self {
        App {
            messages: Vec::new(),
            input: String::new(),
            scroll_offset: 0,
            state: AppState::Normal,
            model,
            message_count: 0,
            event_sender: None,
            tool_status: Vec::new(),
            show_tool_panel: false,
        }
    }

    pub fn set_event_sender(&mut self, sender: UnboundedSender<TuiEvent>) {
        self.event_sender = Some(sender);
    }

    pub fn send_event(&self, event: TuiEvent) -> anyhow::Result<()> {
        if let Some(ref sender) = self.event_sender {
            sender.send(event)?;
        }
        Ok(())
    }

    pub fn add_message(&mut self, msg_type: MessageType, content: String) {
        self.messages.push(ChatMessage {
            msg_type: msg_type.clone(),
            content,
            tool_name: None,
        });
        if msg_type == MessageType::User {
            self.message_count += 1;
        }
        self.scroll_to_bottom();
    }

    pub fn add_tool_message(&mut self, tool_name: &str, content: String) {
        self.messages.push(ChatMessage {
            msg_type: MessageType::Tool,
            content,
            tool_name: Some(tool_name.to_string()),
        });
    }

    pub fn set_state(&mut self, state: AppState) {
        self.state = state;
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if self.scroll_offset < amount {
            self.scroll_offset = 0;
        } else {
            self.scroll_offset -= amount;
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset += amount;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = usize::MAX;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn append_input(&mut self, ch: char) {
        self.input.push(ch);
    }

    pub fn remove_last_char(&mut self) {
        self.input.pop();
    }

    pub fn toggle_tool_panel(&mut self) {
        self.show_tool_panel = !self.show_tool_panel;
    }

    pub fn update_tool_status(&mut self, tool_name: String, status: String) {
        if let Some(item) = self
            .tool_status
            .iter_mut()
            .find(|(name, _)| name == &tool_name)
        {
            item.1 = status;
        } else {
            self.tool_status.push((tool_name, status));
        }
    }

    pub fn clear_tool_status(&mut self) {
        self.tool_status.clear();
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.message_count = 0;
        self.scroll_offset = 0;
    }
}
