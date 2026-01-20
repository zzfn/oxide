//! 任务管理模块
//!
//! 提供后台任务的创建、执行和追踪功能。

pub mod manager;

pub use manager::{Task, TaskId, TaskManager, TaskStatus};
