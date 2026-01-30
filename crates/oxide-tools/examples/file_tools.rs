//! 文件操作工具示例
//!
//! 演示 Read, Write, Edit 工具的使用

use oxide_tools::{EditTool, ReadTool, Tool, WriteTool};
use serde_json::json;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 测试文件操作工具\n");

    // 创建临时目录
    let temp_dir = TempDir::new()?;
    let working_dir = temp_dir.path().to_path_buf();
    println!("📁 工作目录: {}\n", working_dir.display());

    // 测试 Write 工具
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📝 测试 1: Write 工具 - 创建文件");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let write_tool = WriteTool::new(working_dir.clone());
    let result = write_tool
        .execute(json!({
            "file_path": "hello.txt",
            "content": "Hello, Oxide!\nThis is a test file.\nLine 3"
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试 Read 工具
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 测试 2: Read 工具 - 读取完整文件");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let read_tool = ReadTool::new(working_dir.clone());
    let result = read_tool
        .execute(json!({
            "file_path": "hello.txt"
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试 Read 工具 - 行范围读取
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 测试 3: Read 工具 - 行范围读取");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = read_tool
        .execute(json!({
            "file_path": "hello.txt",
            "offset": 1,
            "limit": 1
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试 Edit 工具
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✏️  测试 4: Edit 工具 - 字符串替换");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let edit_tool = EditTool::new(working_dir.clone());
    let result = edit_tool
        .execute(json!({
            "file_path": "hello.txt",
            "old_string": "Oxide",
            "new_string": "Rust"
        }))
        .await?;

    println!("{}\n", result.content);

    // 读取编辑后的文件
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📖 测试 5: 验证编辑结果");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = read_tool
        .execute(json!({
            "file_path": "hello.txt"
        }))
        .await?;

    println!("{}\n", result.content);

    // 测试错误处理
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("❌ 测试 6: 错误处理 - 读取不存在的文件");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let result = read_tool
        .execute(json!({
            "file_path": "nonexistent.txt"
        }))
        .await?;

    if result.is_error {
        println!("✓ 正确捕获错误: {}\n", result.content);
    }

    // 测试批量替换
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✏️  测试 7: Edit 工具 - 批量替换");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 先创建一个有重复内容的文件
    write_tool
        .execute(json!({
            "file_path": "multi.txt",
            "content": "foo bar foo baz foo"
        }))
        .await?;

    let result = edit_tool
        .execute(json!({
            "file_path": "multi.txt",
            "old_string": "foo",
            "new_string": "FOO",
            "replace_all": true
        }))
        .await?;

    println!("{}\n", result.content);

    // 验证批量替换结果
    let result = read_tool
        .execute(json!({
            "file_path": "multi.txt"
        }))
        .await?;

    println!("{}\n", result.content);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎉 所有测试完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
