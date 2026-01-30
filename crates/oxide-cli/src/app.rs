//! 应用状态管理
//!
//! 管理 CLI 的全局状态，包括运行模式、Token 使用统计、会话信息等。

use oxide_core::types::Conversation;
use oxide_provider::LLMProvider;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// CLI 运行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CliMode {
    /// 普通模式 - 完整功能
    #[default]
    Normal,
    /// 快速模式 - 简化交互
    Fast,
    /// 计划模式 - 仅生成计划不执行
    Plan,
}

impl CliMode {
    /// 获取模式的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            CliMode::Normal => "Normal",
            CliMode::Fast => "Fast",
            CliMode::Plan => "Plan",
        }
    }

    /// 获取模式的简短标识
    pub fn short_name(&self) -> &'static str {
        match self {
            CliMode::Normal => "N",
            CliMode::Fast => "F",
            CliMode::Plan => "P",
        }
    }

    /// 切换到下一个模式
    pub fn next(&self) -> Self {
        match self {
            CliMode::Normal => CliMode::Fast,
            CliMode::Fast => CliMode::Plan,
            CliMode::Plan => CliMode::Normal,
        }
    }
}

/// Token 使用统计
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    /// 输入 Token 数量
    pub input_tokens: u64,
    /// 输出 Token 数量
    pub output_tokens: u64,
    /// 缓存命中的 Token 数量
    pub cached_tokens: u64,
}

impl TokenUsage {
    /// 创建新的 Token 统计
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加使用量
    pub fn add(&mut self, input: u64, output: u64, cached: u64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.cached_tokens += cached;
    }

    /// 获取总 Token 数量
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    /// 重置统计
    pub fn reset(&mut self) {
        self.input_tokens = 0;
        self.output_tokens = 0;
        self.cached_tokens = 0;
    }

    /// 格式化显示
    pub fn format(&self) -> String {
        format!(
            "{}↑ {}↓ ({}cached)",
            self.input_tokens, self.output_tokens, self.cached_tokens
        )
    }
}

/// 后台任务状态
#[derive(Debug, Clone, Default)]
pub struct BackgroundTasks {
    /// 运行中的任务数量
    pub running: usize,
    /// 已完成的任务数量
    pub completed: usize,
}

/// 应用状态
pub struct AppState {
    /// 当前运行模式
    pub mode: CliMode,
    /// Token 使用统计
    pub token_usage: TokenUsage,
    /// 当前工作目录
    pub working_dir: PathBuf,
    /// 会话 ID
    pub session_id: Option<String>,
    /// 后台任务状态
    pub background_tasks: BackgroundTasks,
    /// 是否正在处理请求
    pub is_processing: bool,
    /// 连续 Ctrl+C 计数（用于退出确认）
    pub ctrl_c_count: u8,
    /// 当前会话的对话历史
    pub conversation: Conversation,
    /// LLM Provider
    pub provider: Option<Arc<dyn LLMProvider>>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self {
            mode: CliMode::default(),
            token_usage: TokenUsage::new(),
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            session_id: None,
            background_tasks: BackgroundTasks::default(),
            is_processing: false,
            ctrl_c_count: 0,
            conversation: Conversation::new(),
            provider: None,
        }
    }

    /// 设置 LLM Provider
    pub fn set_provider(&mut self, provider: Arc<dyn LLMProvider>) {
        self.provider = Some(provider);
    }

    /// 切换运行模式
    pub fn toggle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    /// 设置运行模式
    pub fn set_mode(&mut self, mode: CliMode) {
        self.mode = mode;
    }

    /// 重置 Ctrl+C 计数
    pub fn reset_ctrl_c(&mut self) {
        self.ctrl_c_count = 0;
    }

    /// 增加 Ctrl+C 计数，返回是否应该退出
    pub fn increment_ctrl_c(&mut self) -> bool {
        self.ctrl_c_count += 1;
        self.ctrl_c_count >= 2
    }

    /// 开始处理请求
    pub fn start_processing(&mut self) {
        self.is_processing = true;
    }

    /// 结束处理请求
    pub fn end_processing(&mut self) {
        self.is_processing = false;
    }

    /// 更新 Token 使用量
    pub fn update_token_usage(&mut self, input: u64, output: u64, cached: u64) {
        self.token_usage.add(input, output, cached);
    }

    /// 清空会话
    pub fn clear_session(&mut self) {
        self.session_id = None;
        self.token_usage.reset();
        self.conversation = Conversation::new();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// 共享应用状态类型别名
pub type SharedAppState = Arc<RwLock<AppState>>;

/// 创建共享应用状态
pub fn create_shared_state() -> SharedAppState {
    Arc::new(RwLock::new(AppState::new()))
}
