//! 观察数据收集器
//!
//! 负责收集和管理工作流执行过程中的观察数据。

use super::types::Observation;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 观察数据收集器
#[derive(Debug, Clone)]
pub struct ObservationCollector {
    /// 存储的观察数据
    observations: Arc<RwLock<Vec<Observation>>>,
}

impl ObservationCollector {
    /// 创建新的观察收集器
    pub fn new() -> Self {
        Self {
            observations: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 添加观察数据
    pub fn add(&self, observation: Observation) {
        if let Ok(mut obs) = self.observations.write() {
            obs.push(observation);
        }
    }
    
    /// 添加工具执行观察
    pub fn add_tool_execution(
        &self,
        tool_name: String,
        input: HashMap<String, serde_json::Value>,
        output: Option<serde_json::Value>,
        success: bool,
        error: Option<String>,
        execution_time_ms: Option<u64>,
    ) {
        let observation = Observation::tool_execution(
            tool_name,
            input,
            output,
            success,
            error,
            execution_time_ms,
        );
        self.add(observation);
    }
    
    /// 添加子 agent 结果观察
    pub fn add_subagent_result(
        &self,
        agent_type: String,
        input: String,
        output: String,
        success: bool,
    ) {
        let observation = Observation::subagent_result(agent_type, input, output, success);
        self.add(observation);
    }
    
    /// 获取所有观察数据
    pub fn get_all(&self) -> Vec<Observation> {
        self.observations.read()
            .map(|obs| obs.clone())
            .unwrap_or_default()
    }
    
    /// 获取最近 N 条观察
    pub fn get_recent(&self, n: usize) -> Vec<Observation> {
        self.observations.read()
            .map(|obs| {
                let len = obs.len();
                if len <= n {
                    obs.clone()
                } else {
                    obs[len - n..].to_vec()
                }
            })
            .unwrap_or_default()
    }
    
    /// 获取某个类型的所有观察
    pub fn get_by_type(&self, observation_type: &str) -> Vec<Observation> {
        self.observations.read()
            .map(|obs| {
                obs.iter()
                    .filter(|o| o.observation_type == observation_type)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// 获取失败的观察
    pub fn get_failures(&self) -> Vec<Observation> {
        self.observations.read()
            .map(|obs| {
                obs.iter()
                    .filter(|o| !o.success)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// 清空所有观察数据
    pub fn clear(&self) {
        if let Ok(mut obs) = self.observations.write() {
            obs.clear();
        }
    }
    
    /// 获取观察数量
    pub fn count(&self) -> usize {
        self.observations.read()
            .map(|obs| obs.len())
            .unwrap_or(0)
    }
    
    /// 生成观察摘要
    pub fn summarize(&self) -> ObservationSummary {
        let observations = self.get_all();
        let total = observations.len();
        let successes = observations.iter().filter(|o| o.success).count();
        let failures = total - successes;
        
        let tool_executions = observations.iter()
            .filter(|o| o.observation_type == "tool_execution")
            .count();
        
        let subagent_calls = observations.iter()
            .filter(|o| o.observation_type == "subagent_result")
            .count();
        
        let total_execution_time = observations.iter()
            .filter_map(|o| o.execution_time_ms)
            .sum();
        
        ObservationSummary {
            total_observations: total,
            successful: successes,
            failed: failures,
            tool_executions,
            subagent_calls,
            total_execution_time_ms: total_execution_time,
        }
    }
}

impl Default for ObservationCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 观察摘要
#[derive(Debug, Clone)]
pub struct ObservationSummary {
    /// 总观察数
    pub total_observations: usize,
    
    /// 成功数
    pub successful: usize,
    
    /// 失败数
    pub failed: usize,
    
    /// 工具执行数
    pub tool_executions: usize,
    
    /// 子 agent 调用数
    pub subagent_calls: usize,
    
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_observation_collector() {
        let collector = ObservationCollector::new();
        assert_eq!(collector.count(), 0);
        
        collector.add_tool_execution(
            "read_file".to_string(),
            HashMap::new(),
            Some(serde_json::json!({"content": "test"})),
            true,
            None,
            Some(100),
        );
        
        assert_eq!(collector.count(), 1);
        
        let all = collector.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].source, "read_file");
        assert!(all[0].success);
    }
    
    #[test]
    fn test_observation_filtering() {
        let collector = ObservationCollector::new();
        
        collector.add_tool_execution(
            "tool1".to_string(),
            HashMap::new(),
            None,
            true,
            None,
            None,
        );
        
        collector.add_tool_execution(
            "tool2".to_string(),
            HashMap::new(),
            None,
            false,
            Some("Error".to_string()),
            None,
        );
        
        collector.add_subagent_result(
            "Explore".to_string(),
            "request".to_string(),
            "response".to_string(),
            true,
        );
        
        let tools = collector.get_by_type("tool_execution");
        assert_eq!(tools.len(), 2);
        
        let subagents = collector.get_by_type("subagent_result");
        assert_eq!(subagents.len(), 1);
        
        let failures = collector.get_failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].source, "tool2");
    }
    
    #[test]
    fn test_observation_summary() {
        let collector = ObservationCollector::new();
        
        collector.add_tool_execution(
            "tool1".to_string(),
            HashMap::new(),
            None,
            true,
            None,
            Some(50),
        );
        
        collector.add_tool_execution(
            "tool2".to_string(),
            HashMap::new(),
            None,
            false,
            Some("Error".to_string()),
            Some(30),
        );
        
        let summary = collector.summarize();
        assert_eq!(summary.total_observations, 2);
        assert_eq!(summary.successful, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.tool_executions, 2);
        assert_eq!(summary.total_execution_time_ms, 80);
    }
}
