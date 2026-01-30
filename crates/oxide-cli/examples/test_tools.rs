//! 测试工具注册表和 schema 生成

use oxide_tools::ToolRegistry;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== 测试工具注册表 ===\n");

    // 创建工具注册表
    let working_dir = PathBuf::from(".");
    let mut registry = ToolRegistry::new();

    // 创建共享的任务管理器
    let task_manager = oxide_tools::create_task_manager();

    // 注册文件操作工具
    registry.register(Arc::new(oxide_tools::ReadTool::new(working_dir.clone())));
    registry.register(Arc::new(oxide_tools::WriteTool::new(working_dir.clone())));
    registry.register(Arc::new(oxide_tools::EditTool::new(working_dir.clone())));

    // 注册搜索工具
    registry.register(Arc::new(oxide_tools::GlobTool::new(working_dir.clone())));
    registry.register(Arc::new(oxide_tools::GrepTool::new(working_dir.clone())));

    // 注册执行工具
    registry.register(Arc::new(oxide_tools::BashTool::with_task_manager(
        working_dir.clone(),
        task_manager.clone(),
    )));
    registry.register(Arc::new(oxide_tools::TaskOutputTool::new(task_manager.clone())));
    registry.register(Arc::new(oxide_tools::TaskStopTool::new(task_manager)));

    // 获取所有工具 schema
    let schemas = registry.schemas();

    println!("注册的工具数量: {}\n", schemas.len());

    // 打印每个工具的 schema
    for schema in &schemas {
        println!("工具名称: {}", schema.name);
        println!("描述: {}", schema.description);
        println!("参数 Schema:");
        println!("{}\n", serde_json::to_string_pretty(&schema.parameters)?);
        println!("---\n");
    }

    // 生成 Anthropic API 格式的工具定义
    println!("=== Anthropic API 格式 ===\n");
    let tool_schemas: Vec<serde_json::Value> = schemas
        .into_iter()
        .map(|schema| {
            serde_json::json!({
                "name": schema.name,
                "description": schema.description,
                "input_schema": schema.parameters,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&tool_schemas)?);

    Ok(())
}
