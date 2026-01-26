//! 工具观察包装器
//!
//! 包装 rig::Tool，在执行时自动记录观察数据。

use super::observation::ObservationCollector;
use anyhow::Result;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// 可观察的工具包装器
pub struct ObservableTool<T: Tool> {
    inner: T,
    collector: ObservationCollector,
}

impl<T: Tool> ObservableTool<T> {
    /// 创建新的可观察工具包装器
    pub fn new(inner: T, collector: ObservationCollector) -> Self {
        Self { inner, collector }
    }
}

impl<T: Tool + Send + Sync> Tool for ObservableTool<T>
where
    T::Args: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    T::Output: Serialize + Send + Sync,
{
    const NAME: &'static str = T::NAME;

    type Error = T::Error;
    type Args = T::Args;
    type Output = T::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let start = Instant::now();
        
        // 记录输入数据
        let input_json = serde_json::to_value(&args).unwrap_or(serde_json::Value::Null);
        let mut input_map = HashMap::new();
        if let serde_json::Value::Object(map) = input_json {
            for (k, v) in map {
                input_map.insert(k, v);
            }
        } else {
            input_map.insert("args".to_string(), input_json);
        }

        // 执行工具
        let result = self.inner.call(args).await;
        
        // 计算耗时
        let elapsed_ms = start.elapsed().as_millis() as u64;

        // 记录输出和成功状态
        match &result {
            Ok(output) => {
                let output_json = serde_json::to_value(output).unwrap_or(serde_json::Value::Null);
                self.collector.add_tool_execution(
                    T::NAME.to_string(),
                    input_map,
                    Some(output_json),
                    true,
                    None,
                    Some(elapsed_ms),
                );
            }
            Err(e) => {
                self.collector.add_tool_execution(
                    T::NAME.to_string(),
                    input_map,
                    None,
                    false,
                    Some(e.to_string()),
                    Some(elapsed_ms),
                );
            }
        }

        result
    }
}
