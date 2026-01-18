use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Normal,
    Processing,
    Error(String),
}

/// 布局模式 - 支持多种界面布局风格
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    /// 标准模式：完整边框消息卡片
    Standard,
    /// 紧凑模式：简化消息格式，无边框
    Compact,
    /// 极简模式：最精简显示
    Minimal,
    /// 分屏模式：工具面板常驻
    Split,
}

impl LayoutMode {
    /// 循环切换到下一个布局模式
    pub fn next(&self) -> Self {
        match self {
            LayoutMode::Standard => LayoutMode::Compact,
            LayoutMode::Compact => LayoutMode::Minimal,
            LayoutMode::Minimal => LayoutMode::Split,
            LayoutMode::Split => LayoutMode::Standard,
        }
    }

    /// 是否显示消息卡片边框
    pub fn show_card_borders(&self) -> bool {
        matches!(self, LayoutMode::Standard | LayoutMode::Split)
    }

    /// 是否显示帮助栏
    pub fn show_help_bar(&self) -> bool {
        !matches!(self, LayoutMode::Minimal)
    }

    /// 消息间距（行数）
    pub fn message_spacing(&self) -> usize {
        match self {
            LayoutMode::Standard | LayoutMode::Split => 2,
            LayoutMode::Compact => 1,
            LayoutMode::Minimal => 0,
        }
    }

    /// 是否常驻工具面板
    pub fn always_show_tool_panel(&self) -> bool {
        matches!(self, LayoutMode::Split)
    }
}

pub const OXIDE_LOGO: &str = r#"
 ██╗  ██╗███████╗███╗   ██╗██████╗ ███████╗ ██████╗ ██████╗ ██████╗ ███████╗
 ██║  ██║██╔════╝████╗  ██║██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝
 ███████║█████╗  ██╔██╗ ██║██║  ██║███████╗██║   ██║██████╔╝██║  ██║█████╗  
 ██╔══██║██╔══╝  ██║╚██╗██║██║  ██║╚════██║██║   ██║██╔══██╗██║  ██║██╔══╝  
 ██║  ██║███████╗██║ ╚████║██████╔╝███████║╚██████╔╝██║  ██║██████╔╝███████╗
 ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
"#;

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
    Backspace,
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
    ToggleLayoutMode, // 新增：切换布局模式
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
    pub show_welcome: bool,
    pub session_id: String,
    pub stream_chars_per_tick: usize,
    pub tick_count: u64,
    pub layout_mode: LayoutMode, // 新增：当前布局模式
    dirty: bool,
    streaming_message: Option<StreamingMessage>,
}

impl App {
    pub fn new(model: String, session_id: String, stream_chars_per_tick: usize) -> Self {
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
            show_welcome: false,
            session_id,
            stream_chars_per_tick,
            tick_count: 0,
            layout_mode: LayoutMode::Standard, // 默认标准模式
            dirty: true,
            streaming_message: None,
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
        self.mark_dirty();
    }

    pub fn add_tool_message(&mut self, tool_name: &str, content: String) {
        self.messages.push(ChatMessage {
            msg_type: MessageType::Tool,
            content,
            tool_name: Some(tool_name.to_string()),
        });
        self.mark_dirty();
    }

    pub fn set_state(&mut self, state: AppState) {
        if self.state != state {
            self.state = state;
            self.mark_dirty();
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        let new_offset = if self.scroll_offset < amount {
            0
        } else {
            self.scroll_offset - amount
        };
        if new_offset != self.scroll_offset {
            self.scroll_offset = new_offset;
            self.mark_dirty();
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
        self.mark_dirty();
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.scroll_offset != 0 {
            self.scroll_offset = 0;
            self.mark_dirty();
        }
    }

    pub fn scroll_to_top(&mut self) {
        if self.scroll_offset != usize::MAX {
            self.scroll_offset = usize::MAX;
            self.mark_dirty();
        }
    }

    pub fn clear_input(&mut self) {
        if !self.input.is_empty() {
            self.input.clear();
            self.mark_dirty();
        }
    }

    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    pub fn set_input(&mut self, input: String) {
        if self.input != input {
            self.input = input;
            self.mark_dirty();
        }
    }

    pub fn append_input(&mut self, ch: char) {
        self.input.push(ch);
        self.mark_dirty();
    }

    pub fn remove_last_char(&mut self) {
        if self.input.pop().is_some() {
            self.mark_dirty();
        }
    }

    pub fn toggle_tool_panel(&mut self) {
        // 在 Split 模式下，切换到其他模式
        if self.layout_mode.always_show_tool_panel() {
            self.layout_mode = LayoutMode::Standard;
            self.show_tool_panel = false;
        } else {
            self.show_tool_panel = !self.show_tool_panel;
        }
        self.mark_dirty();
    }

    /// 切换布局模式
    pub fn toggle_layout_mode(&mut self) {
        self.layout_mode = self.layout_mode.next();
        // 如果是 Split 模式，自动显示工具面板
        if self.layout_mode.always_show_tool_panel() {
            self.show_tool_panel = true;
        }
        self.mark_dirty();
    }

    pub fn update_tool_status(&mut self, tool_name: String, status: String) {
        if let Some(item) = self
            .tool_status
            .iter_mut()
            .find(|(name, _)| name == &tool_name)
        {
            if item.1 != status {
                item.1 = status;
                self.mark_dirty();
            }
        } else {
            self.tool_status.push((tool_name, status));
            self.mark_dirty();
        }
    }

    pub fn clear_tool_status(&mut self) {
        if !self.tool_status.is_empty() {
            self.tool_status.clear();
            self.mark_dirty();
        }
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.message_count = 0;
        self.scroll_offset = 0;
        self.streaming_message = None;
        self.mark_dirty();
    }

    pub fn tick(&mut self) -> bool {
        self.tick_count = self.tick_count.wrapping_add(1);
        let streaming_changed = self.advance_streaming();
        let spinner_active = self
            .tool_status
            .iter()
            .any(|(_, status)| status.contains("执行中"));
        if spinner_active {
            self.mark_dirty();
        }
        streaming_changed || spinner_active
    }

    pub fn start_streaming_message(&mut self, content: String) {
        let message_index = self.messages.len();
        self.messages.push(ChatMessage {
            msg_type: MessageType::Assistant,
            content: String::new(),
            tool_name: None,
        });

        let chars: Vec<char> = content.chars().collect();
        self.streaming_message = Some(StreamingMessage {
            full: content,
            chars,
            visible_chars: 0,
            message_index,
        });
        self.set_state(AppState::Processing);
        self.scroll_to_bottom();
        self.mark_dirty();
    }

    pub fn advance_streaming(&mut self) -> bool {
        let mut updated = false;
        if let Some(streaming) = self.streaming_message.as_mut() {
            let next_visible = (streaming.visible_chars + self.stream_chars_per_tick)
                .min(streaming.chars.len());
            if next_visible != streaming.visible_chars {
                let content: String = streaming.chars[..next_visible].iter().collect();
                if let Some(message) = self.messages.get_mut(streaming.message_index) {
                    message.content = content;
                }
                streaming.visible_chars = next_visible;
                updated = true;
            }

            if streaming.visible_chars >= streaming.chars.len() {
                if let Some(message) = self.messages.get_mut(streaming.message_index) {
                    message.content = streaming.full.clone();
                }
                self.streaming_message = None;
                self.set_state(AppState::Normal);
                updated = true;
            }
        }

        if updated {
            self.mark_dirty();
        }
        updated
    }

    pub fn has_active_streaming(&self) -> bool {
        self.streaming_message.is_some()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn take_dirty(&mut self) -> bool {
        let dirty = self.dirty;
        self.dirty = false;
        dirty
    }
}

struct StreamingMessage {
    full: String,
    chars: Vec<char>,
    visible_chars: usize,
    message_index: usize,
}
