pub mod types;
pub mod subagent;
pub mod builder;
pub mod hitl_gatekeeper;
pub mod hitl_integration;

pub use types::AgentType as NewAgentType;
pub use subagent::SubagentManager;
pub use builder::AgentBuilder;
pub use hitl_gatekeeper::{HitlGatekeeper, HitlConfig, HitlDecision, ToolCallRequest};
pub use hitl_integration::{HitlIntegration, HitlResult, build_operation_context};

// 重新导出旧的类型以保持向后兼容
pub use builder::AgentEnum as AgentType;
