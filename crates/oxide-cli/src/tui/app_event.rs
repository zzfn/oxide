use tokio::sync::mpsc;

/// TUI 应用事件
#[derive(Debug)]
pub enum AppEvent {
    /// 用户提交文本
    UserSubmit(String),
    /// AI 流式文本片段
    StreamText(String),
    /// AI 思考/推理内容
    StreamReasoning(String),
    /// 工具调用开始
    ToolCallBegin {
        name: String,
        arguments: String,
    },
    /// 工具调用结束
    ToolCallEnd {
        name: String,
        success: bool,
        summary: Option<String>,
    },
    /// AI 回合结束
    TurnComplete {
        response: String,
    },
    /// 流式错误
    StreamError(String),
    /// 需要用户审批工具调用
    ApprovalRequest {
        id: String,
        tool_name: String,
        description: String,
    },
    /// 用户审批结果
    ApprovalResponse {
        id: String,
        approved: bool,
    },
    /// 插入历史 cell
    InsertHistoryCell(Box<super::history_cell::HistoryCell>),
    /// 请求重绘
    Redraw,
    /// 退出
    Quit,
}

/// 事件发送器（可 Clone）
#[derive(Clone)]
pub struct AppEventSender {
    tx: mpsc::UnboundedSender<AppEvent>,
}

impl AppEventSender {
    pub fn new(tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self { tx }
    }

    pub fn send(&self, event: AppEvent) {
        let _ = self.tx.send(event);
    }
}
