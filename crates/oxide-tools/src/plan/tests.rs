//! 计划模式测试

#[cfg(test)]
mod tests {
    use crate::plan::{PlanManager, RigEnterPlanModeTool, RigExitPlanModeTool};
    use rig::tool::Tool;

    #[tokio::test]
    async fn test_plan_manager_lifecycle() {
        let manager = PlanManager::new();

        // 初始状态
        assert!(!manager.is_plan_mode().await);
        assert!(manager.current_plan_id().await.is_none());

        // 进入计划模式
        let plan_id = manager.enter_plan_mode(Some("Test Plan".to_string())).await;
        assert!(manager.is_plan_mode().await);
        assert_eq!(manager.current_plan_id().await, Some(plan_id));

        // 退出计划模式
        let exited_id = manager.exit_plan_mode().await;
        assert_eq!(exited_id, Some(plan_id));
        assert!(!manager.is_plan_mode().await);
        assert!(manager.current_plan_id().await.is_none());
    }

    #[tokio::test]
    async fn test_enter_plan_mode_tool() {
        let manager = PlanManager::new();
        let tool = RigEnterPlanModeTool::new(manager.clone());

        let result = tool
            .call(super::super::tools::EnterPlanModeArgs {})
            .await
            .unwrap();

        assert!(!result.plan_id.is_empty());
        assert!(result.message.contains("已进入计划模式"));
        assert!(manager.is_plan_mode().await);
    }

    #[tokio::test]
    async fn test_exit_plan_mode_tool() {
        let manager = PlanManager::new();

        // 先进入计划模式
        manager.enter_plan_mode(None).await;

        let tool = RigExitPlanModeTool::new(manager.clone());

        let result = tool
            .call(super::super::tools::ExitPlanModeArgs {
                plan_content: "# Test Plan\n\n## Steps\n1. Step 1\n2. Step 2".to_string(),
                plan_title: Some("Test Implementation Plan".to_string()),
                allowed_prompts: None,
            })
            .await
            .unwrap();

        assert!(!result.plan_id.is_empty());
        assert!(result.message.contains("计划已保存"));
        assert!(!manager.is_plan_mode().await);
    }

    #[tokio::test]
    async fn test_exit_plan_mode_with_permissions() {
        let manager = PlanManager::new();
        manager.enter_plan_mode(None).await;

        let tool = RigExitPlanModeTool::new(manager.clone());

        let result = tool
            .call(super::super::tools::ExitPlanModeArgs {
                plan_content: "# Test Plan\n\n## Steps\n1. Run tests\n2. Install deps".to_string(),
                plan_title: Some("Test with Permissions".to_string()),
                allowed_prompts: Some(vec![
                    super::super::tools::AllowedPromptArg {
                        tool: "Bash".to_string(),
                        prompt: "run tests".to_string(),
                    },
                    super::super::tools::AllowedPromptArg {
                        tool: "Bash".to_string(),
                        prompt: "install dependencies".to_string(),
                    },
                ]),
            })
            .await
            .unwrap();

        assert!(!result.plan_id.is_empty());
        assert!(result.message.contains("计划已保存"));
        assert!(result.message.contains("请求的权限"));
        assert!(result.message.contains("run tests"));
        assert!(!manager.is_plan_mode().await);
    }
}
