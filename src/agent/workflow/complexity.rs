//! 任务复杂度评估器
//!
//! 用于评估用户请求的复杂度，决定是否需要启用 PAOR 工作流。

use std::collections::HashSet;

/// 复杂度评估结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityLevel {
    /// 简单任务 - 直接对话模式
    Simple,
    /// 中等任务 - 可选工作流模式
    Medium,
    /// 复杂任务 - 强制使用工作流模式
    Complex,
}

impl ComplexityLevel {
    /// 是否应该使用工作流
    pub fn should_use_workflow(&self) -> bool {
        matches!(self, Self::Medium | Self::Complex)
    }

    /// 获取置信度（0.0 - 1.0）
    pub fn confidence(&self) -> f32 {
        match self {
            Self::Simple => 0.3,
            Self::Medium => 0.6,
            Self::Complex => 0.9,
        }
    }
}

/// 任务复杂度评估器
pub struct ComplexityEvaluator {
    /// 复杂任务关键词
    complex_keywords: HashSet<&'static str>,

    /// 多步骤标记关键词
    multi_step_keywords: HashSet<&'static str>,

    /// 简单任务关键词
    simple_keywords: HashSet<&'static str>,
}

impl Default for ComplexityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplexityEvaluator {
    /// 创建新的评估器
    pub fn new() -> Self {
        let complex_keywords = vec![
            // 设计相关
            "设计", "实现", "重构", "架构", "优化",
            // 多步骤
            "研究", "探索", "规划", "方案",
            // 代码库操作
            "代码库", "全部", "所有", "整个", "批量",
            // 复杂操作
            "迁移", "集成", "转换", "重写",
            // 工作流相关
            "工作流", "多步骤", "分阶段", "迭代",
        ].into_iter().collect();

        let multi_step_keywords = vec![
            "然后", "接着", "之后", "最后", "首先",
            "步骤", "阶段", "流程", "序列",
            "多个", "若干", "一系列",
            "分析",
        ].into_iter().collect();

        let simple_keywords = vec![
            "什么是", "怎么", "如何", "解释",
            "查看", "显示", "列出", "读取",
            "单个", "简单",
            "hello", "hi", "test",
        ].into_iter().collect();

        Self {
            complex_keywords,
            multi_step_keywords,
            simple_keywords,
        }
    }

    /// 评估任务复杂度
    pub fn evaluate(&self, user_input: &str) -> ComplexityLevel {
        let input_lower = user_input.to_lowercase();

        // 1. 检查长度阈值
        let length_score = self.evaluate_length(&input_lower);

        // 2. 检查关键词
        let keyword_score = self.evaluate_keywords(&input_lower);

        // 3. 综合评分
        let total_score = length_score + keyword_score;

        // 4. 判断复杂度级别
        if total_score >= 1.5 {
            ComplexityLevel::Complex
        } else if total_score >= 0.5 {
            ComplexityLevel::Medium
        } else {
            ComplexityLevel::Simple
        }
    }

    /// 评估输入长度
    fn evaluate_length(&self, input: &str) -> f32 {
        let char_count = input.chars().count();

        if char_count >= 100 {
            1.0  // 长输入
        } else if char_count >= 50 {
            0.5  // 中等长度
        } else {
            0.0  // 短输入
        }
    }

    /// 评估关键词
    fn evaluate_keywords(&self, input: &str) -> f32 {
        let mut score: f32 = 0.0;

        // 检查复杂关键词（权重高）
        for keyword in &self.complex_keywords {
            if input.contains(keyword) {
                score += 0.8;
            }
        }

        // 检查多步骤关键词（权重中）
        for keyword in &self.multi_step_keywords {
            if input.contains(keyword) {
                score += 0.5;
            }
        }

        // 检查简单关键词（降低复杂度）
        for keyword in &self.simple_keywords {
            if input.contains(keyword) {
                score -= 0.3;
            }
        }

        score.max(0.0).min(2.0)  // 限制在 [0, 2] 范围内
    }

    /// 检查是否包含明确的模式标记
    pub fn has_explicit_mode_marker(&self, input: &str) -> Option<bool> {
        let input_lower = input.to_lowercase();

        // 明确要求使用工作流
        if input_lower.contains("#workflow")
            || input_lower.contains("#multi-step")
            || input_lower.contains("使用工作流")
            || input_lower.contains("启用工作流")
        {
            return Some(true);
        }

        // 明确要求不使用工作流
        if input_lower.contains("#simple")
            || input_lower.contains("#quick")
            || input_lower.contains("简单模式")
            || input_lower.contains("直接回答")
        {
            return Some(false);
        }

        None
    }

    /// 评估并决定是否使用工作流
    pub fn should_use_workflow(&self, user_input: &str) -> bool {
        // 1. 检查明确的模式标记
        if let Some(explicit) = self.has_explicit_mode_marker(user_input) {
            return explicit;
        }

        // 2. 使用复杂度评估
        let complexity = self.evaluate(user_input);
        complexity.should_use_workflow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_task() {
        let evaluator = ComplexityEvaluator::new();

        assert!(!evaluator.should_use_workflow("什么是 Rust?"));
        assert!(!evaluator.should_use_workflow("Hello"));
        assert!(!evaluator.should_use_workflow("列出当前目录的文件"));
    }

    #[test]
    fn test_complex_task() {
        let evaluator = ComplexityEvaluator::new();

        assert!(evaluator.should_use_workflow("帮我设计并实现一个用户认证系统"));
        assert!(evaluator.should_use_workflow("分析整个代码库并重构所有错误处理"));
        assert!(evaluator.should_use_workflow("探索代码库结构，找出所有 TODO 项，然后创建任务列表"));
    }

    #[test]
    fn test_explicit_markers() {
        let evaluator = ComplexityEvaluator::new();

        // 明确使用工作流
        assert!(evaluator.should_use_workflow("帮我实现一个功能 #workflow"));
        assert!(evaluator.should_use_workflow("使用工作流重构代码"));

        // 明确不使用工作流
        assert!(!evaluator.should_use_workflow("什么是闭包？ #simple"));
        assert!(!evaluator.should_use_workflow("简单模式下列出文件"));
    }

    #[test]
    fn test_complexity_levels() {
        let evaluator = ComplexityEvaluator::new();

        let simple = evaluator.evaluate("Hello");
        assert_eq!(simple, ComplexityLevel::Simple);

        let medium = evaluator.evaluate("分析这个函数并给出建议");
        assert_eq!(medium, ComplexityLevel::Medium);

        let complex = evaluator.evaluate("设计并实现一个完整的用户认证系统，包括登录、注册和密码重置功能");
        assert_eq!(complex, ComplexityLevel::Complex);
    }

    #[test]
    fn test_chinese_keywords() {
        let evaluator = ComplexityEvaluator::new();

        assert!(evaluator.should_use_workflow("重构代码库"));
        assert!(evaluator.should_use_workflow("优化性能"));
        assert!(evaluator.should_use_workflow("批量重命名文件"));
    }
}
