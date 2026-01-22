//! Subagent 管理
//!
//! 提供多 Agent 系统的管理功能，包括 Agent 注册、切换和能力查询。

use crate::agent::types::{AgentCapability, AgentType};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Subagent 管理器
///
/// 负责管理不同类型的 Agent，提供注册、切换和查询功能。
pub struct SubagentManager {
    /// 当前激活的 Agent 类型
    #[allow(dead_code)]
    current_agent: Arc<RwLock<AgentType>>,

    /// Agent 能力映射
    capabilities: HashMap<AgentType, AgentCapability>,
}

impl SubagentManager {
    /// 创建新的 Subagent 管理器
    pub fn new() -> Self {
        let capabilities = AgentCapability::all_capabilities();
        let current_agent = Arc::new(RwLock::new(AgentType::Main));

        Self {
            current_agent,
            capabilities,
        }
    }

    /// 注册 Agent 能力
    ///
    /// 通常不需要手动调用，因为 `all_capabilities()` 已经预注册了所有 Agent。
    #[allow(dead_code)]
    pub fn register(&mut self, agent_type: AgentType, capability: AgentCapability) {
        self.capabilities.insert(agent_type, capability);
    }

    /// 切换到指定的 Agent 类型
    ///
    /// # 参数
    ///
    /// * `agent_type` - 要切换到的 Agent 类型
    ///
    /// # 返回
    ///
    /// 返回之前的 Agent 类型
    #[allow(dead_code)]
    pub fn switch_to(&self, agent_type: AgentType) -> Result<AgentType> {
        // 验证 Agent 类型是否已注册
        if !self.capabilities.contains_key(&agent_type) {
            anyhow::bail!(
                "未注册的 Agent 类型: {:?}，请先注册该 Agent",
                agent_type
            );
        }

        let mut current = self
            .current_agent
            .write()
            .map_err(|e| anyhow::anyhow!("获取写锁失败: {}", e))?;
        let previous = *current;
        *current = agent_type;

        Ok(previous)
    }

    /// 获取当前激活的 Agent 类型
    pub fn current(&self) -> Result<AgentType> {
        self.current_agent
            .read()
            .map_err(|e| anyhow::anyhow!("获取读锁失败: {}", e))
            .map(|agent| *agent)
    }

    /// 检查指定的 Agent 类型是否为当前激活的 Agent
    #[allow(dead_code)]
    pub fn is_current(&self, agent_type: AgentType) -> Result<bool> {
        Ok(self.current()? == agent_type)
    }

    /// 列出所有已注册的 Agent 能力
    pub fn list_capabilities(&self) -> Vec<AgentCapability> {
        self.capabilities.values().cloned().collect()
    }

    /// 获取指定 Agent 类型的能力描述
    #[allow(dead_code)]
    pub fn get_capability(&self, agent_type: AgentType) -> Option<AgentCapability> {
        self.capabilities.get(&agent_type).cloned()
    }

    /// 获取指定 Agent 类型的系统提示词
    #[allow(dead_code)]
    pub fn get_system_prompt(&self, agent_type: AgentType) -> Option<String> {
        self.capabilities
            .get(&agent_type)
            .map(|cap| cap.system_prompt.clone())
    }

    /// 获取指定 Agent 类型的工具列表
    #[allow(dead_code)]
    pub fn get_tools(&self, agent_type: AgentType) -> Option<Vec<String>> {
        self.capabilities
            .get(&agent_type)
            .map(|cap| cap.tools.clone())
    }

    /// 检查指定 Agent 是否为只读模式
    #[allow(dead_code)]
    pub fn is_read_only(&self, agent_type: AgentType) -> bool {
        self.capabilities
            .get(&agent_type)
            .map(|cap| cap.read_only)
            .unwrap_or(false)
    }

    /// 获取所有已注册的 Agent 类型
    #[allow(dead_code)]
    pub fn registered_agent_types(&self) -> Vec<AgentType> {
        self.capabilities.keys().copied().collect()
    }
}

impl Default for SubagentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_manager_creation() {
        let manager = SubagentManager::new();
        assert_eq!(manager.current().unwrap(), AgentType::Main);
    }

    #[test]
    fn test_switch_agent() {
        let manager = SubagentManager::new();

        // 切换到 Explore Agent
        let previous = manager.switch_to(AgentType::Explore).unwrap();
        assert_eq!(previous, AgentType::Main);
        assert_eq!(manager.current().unwrap(), AgentType::Explore);
        assert!(manager.is_current(AgentType::Explore).unwrap());

        // 切换到 Plan Agent
        let previous = manager.switch_to(AgentType::Plan).unwrap();
        assert_eq!(previous, AgentType::Explore);
        assert_eq!(manager.current().unwrap(), AgentType::Plan);
    }

    #[test]
    fn test_switch_to_invalid_agent() {
        let manager = SubagentManager::new();

        // General Agent 在 all_capabilities() 中没有单独注册，
        // 而是在 for_agent_type() 中动态使用 Main 的能力
        // 所以切换应该失败
        let result = manager.switch_to(AgentType::General);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_capabilities() {
        let manager = SubagentManager::new();
        let capabilities = manager.list_capabilities();

        // 至少应该有 5 个 Agent: Main, Explore, Plan, CodeReviewer, FrontendDeveloper
        assert!(capabilities.len() >= 5);

        // 检查每个 Agent 都有必要的字段
        for cap in &capabilities {
            assert!(!cap.name.is_empty());
            assert!(!cap.description.is_empty());
            assert!(!cap.tools.is_empty());
            assert!(!cap.system_prompt.is_empty());
        }
    }

    #[test]
    fn test_get_capability() {
        let manager = SubagentManager::new();

        // 获取 Explore Agent 的能力
        let explore_cap = manager.get_capability(AgentType::Explore);
        assert!(explore_cap.is_some());
        let explore_cap = explore_cap.unwrap();
        assert_eq!(explore_cap.agent_type, AgentType::Explore);
        assert!(explore_cap.read_only);

        // General Agent 在默认注册表中不存在（通过 for_agent_type 动态创建）
        let nonexistent = manager.get_capability(AgentType::General);
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_is_read_only() {
        let manager = SubagentManager::new();

        // Explore Agent 应该是只读的
        assert!(manager.is_read_only(AgentType::Explore));

        // Code Reviewer Agent 应该是只读的
        assert!(manager.is_read_only(AgentType::CodeReviewer));

        // Main Agent 不应该是只读的
        assert!(!manager.is_read_only(AgentType::Main));

        // Frontend Developer Agent 不应该是只读的
        assert!(!manager.is_read_only(AgentType::FrontendDeveloper));
    }

    #[test]
    fn test_get_system_prompt() {
        let manager = SubagentManager::new();

        // 获取 Main Agent 的系统提示词
        let main_prompt = manager.get_system_prompt(AgentType::Main);
        assert!(main_prompt.is_some());
        assert!(main_prompt.unwrap().contains("Oxide"));

        // 获取 Explore Agent 的系统提示词
        let explore_prompt = manager.get_system_prompt(AgentType::Explore);
        assert!(explore_prompt.is_some());
        assert!(explore_prompt.unwrap().contains("Explore"));
    }

    #[test]
    fn test_get_tools() {
        let manager = SubagentManager::new();

        // Main Agent 应该有所有工具
        let main_tools = manager.get_tools(AgentType::Main);
        assert!(main_tools.is_some());
        let main_tools = main_tools.unwrap();
        assert!(main_tools.contains(&"read_file".to_string()));
        assert!(main_tools.contains(&"write_file".to_string()));
        assert!(main_tools.contains(&"edit_file".to_string()));

        // Explore Agent 应该只有只读工具
        let explore_tools = manager.get_tools(AgentType::Explore);
        assert!(explore_tools.is_some());
        let explore_tools = explore_tools.unwrap();
        assert!(explore_tools.contains(&"read_file".to_string()));
        assert!(!explore_tools.contains(&"write_file".to_string()));
        assert!(!explore_tools.contains(&"edit_file".to_string()));
    }

    #[test]
    fn test_registered_agent_types() {
        let manager = SubagentManager::new();
        let types = manager.registered_agent_types();

        // 应该至少包含 5 个 Agent 类型
        assert!(types.len() >= 5);
        assert!(types.contains(&AgentType::Main));
        assert!(types.contains(&AgentType::Explore));
        assert!(types.contains(&AgentType::Plan));
        assert!(types.contains(&AgentType::CodeReviewer));
        assert!(types.contains(&AgentType::FrontendDeveloper));
    }

    #[test]
    fn test_register_custom_agent() {
        let mut manager = SubagentManager::new();

        // 创建一个自定义 Agent 能力
        let custom_capability = AgentCapability::new(
            AgentType::General,
            "Custom Agent".to_string(),
            "自定义测试 Agent".to_string(),
            vec!["read_file".to_string()],
            "You are a custom agent".to_string(),
            true,
        );

        // 注册自定义 Agent
        manager.register(AgentType::General, custom_capability);

        // 验证注册成功
        let capability = manager.get_capability(AgentType::General);
        assert!(capability.is_some());
        assert_eq!(capability.unwrap().name, "Custom Agent");
    }
}
