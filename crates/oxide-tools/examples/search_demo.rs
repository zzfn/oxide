//! Glob 和 Grep 工具使用示例

use oxide_tools::{GlobTool, GrepTool, Tool};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    println!("当前工作目录: {}\n", current_dir.display());

    // 示例 1: 使用 Glob 查找所有 Rust 文件
    println!("=== 示例 1: Glob 查找所有 .rs 文件 ===");
    let glob_tool = GlobTool::new(current_dir.clone());
    let result = glob_tool
        .execute(json!({
            "pattern": "**/*.rs"
        }))
        .await?;
    println!("找到的文件:\n{}\n", result.content);

    // 示例 2: 使用 Glob 查找特定目录下的文件
    println!("=== 示例 2: Glob 查找 src 目录下的 .rs 文件 ===");
    let result = glob_tool
        .execute(json!({
            "pattern": "src/**/*.rs"
        }))
        .await?;
    println!("找到的文件:\n{}\n", result.content);

    // 示例 3: 使用 Grep 搜索包含 "Tool" 的文件
    println!("=== 示例 3: Grep 搜索包含 'Tool' 的文件 ===");
    let grep_tool = GrepTool::new(current_dir.clone());
    let result = grep_tool
        .execute(json!({
            "pattern": "Tool",
            "output_mode": "files_with_matches"
        }))
        .await?;
    println!("包含 'Tool' 的文件:\n{}\n", result.content);

    // 示例 4: 使用 Grep 搜索并显示匹配内容
    println!("=== 示例 4: Grep 搜索 'pub struct' 并显示内容 ===");
    let result = grep_tool
        .execute(json!({
            "pattern": "pub struct",
            "output_mode": "content",
            "type": "rust",
            "head_limit": 5
        }))
        .await?;
    println!("匹配内容:\n{}\n", result.content);

    // 示例 5: 使用 Grep 搜索并显示上下文
    println!("=== 示例 5: Grep 搜索 'async fn execute' 并显示上下文 ===");
    let result = grep_tool
        .execute(json!({
            "pattern": "async fn execute",
            "output_mode": "content",
            "context": 2,
            "type": "rust",
            "head_limit": 3
        }))
        .await?;
    println!("匹配内容（带上下文）:\n{}\n", result.content);

    // 示例 6: 使用 Grep 计数匹配行数
    println!("=== 示例 6: Grep 计数包含 'use' 的行数 ===");
    let result = grep_tool
        .execute(json!({
            "pattern": "^use ",
            "output_mode": "count",
            "type": "rust"
        }))
        .await?;
    println!("匹配计数:\n{}\n", result.content);

    // 示例 7: 使用 Grep 忽略大小写搜索
    println!("=== 示例 7: Grep 忽略大小写搜索 'TOOL' ===");
    let result = grep_tool
        .execute(json!({
            "pattern": "TOOL",
            "-i": true,
            "output_mode": "files_with_matches",
            "head_limit": 3
        }))
        .await?;
    println!("找到的文件:\n{}\n", result.content);

    // 示例 8: 使用 Grep 搜索特定文件
    println!("=== 示例 8: Grep 在 lib.rs 中搜索 'pub' ===");
    let result = grep_tool
        .execute(json!({
            "pattern": "pub",
            "path": "src/lib.rs",
            "output_mode": "content"
        }))
        .await?;
    println!("匹配内容:\n{}\n", result.content);

    Ok(())
}
