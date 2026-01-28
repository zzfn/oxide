//! 工作流引擎模块
//!
//! 实现自主的 Plan-Act-Observe-Reflect (PAOR) 循环，支持多步骤任务执行。

pub mod orchestrator;
pub mod state;
pub mod types;
pub mod observation;
pub mod tool_wrapper;
pub mod executor;
pub mod complexity;

pub use orchestrator::{WorkflowOrchestrator, OrchestratorConfig};
pub use state::{WorkflowState, WorkflowPhase};
pub use tool_wrapper::ObservableTool;
#[allow(unused_imports)]
pub use types::{Task, TaskStatus, TaskId, Plan, Observation, Reflection, ExecutionType, ObservationAnalysis};
#[allow(unused_imports)]
pub use observation::ObservationCollector;
pub use executor::{WorkflowExecutor, WorkflowResult, WorkflowProgress, ProgressCallback};
pub use complexity::{ComplexityEvaluator, ComplexityLevel};
