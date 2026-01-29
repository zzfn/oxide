//! 工具系统实现
//!
//! 本模块提供 Oxide 的工具框架和内置工具。

pub mod exec;
pub mod file;
pub mod registry;
pub mod search;
pub mod web;

pub use registry::{Tool, ToolRegistry};
