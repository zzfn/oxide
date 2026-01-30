//! 任务管理系统
//!
//! 提供任务创建、查询、更新和依赖管理功能。

pub mod errors;
pub mod manager;
pub mod tools;
pub mod types;

// 重新导出常用类型
pub use errors::TaskError;
pub use manager::{BackgroundTask, TaskManager};
pub use types::{Task, TaskStatus};
