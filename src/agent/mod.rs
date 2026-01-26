pub mod types;
pub mod subagent;
pub mod builder;
pub mod hitl_gatekeeper;
pub mod hitl_integration;
pub mod workflow;

pub use types::AgentType as NewAgentType;
pub use subagent::SubagentManager;
pub use builder::AgentBuilder;
#[allow(unused_imports)]
pub use hitl_integration::{HitlResult, MaybeHitlTool, HitlIntegration, build_operation_context};
#[allow(unused_imports)]
pub use hitl_gatekeeper::{HitlGatekeeper, ToolCallRequest, OperationContext, HitlConfig, HitlDecision, WarningLevel};
#[allow(unused_imports)]
pub use workflow::{WorkflowOrchestrator, WorkflowState, WorkflowPhase};

// 重新导出旧的类型以保持向后兼容
pub use builder::AgentEnum as AgentType;
