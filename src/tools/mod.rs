use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

pub mod create_directory;
pub mod delete_file;
pub mod edit_file;
pub mod grep_search;
pub mod scan_codebase;

pub use create_directory::create_directory;
pub use delete_file::delete_file;
pub use edit_file::edit_file;
pub use grep_search::grep_search;
pub use scan_codebase::scan_codebase;

pub fn read_file(path: &str) -> Result<Value> {
    let content = fs::read_to_string(path).with_context(|| format!("无法读取文件: {}", path))?;
    Ok(json!({
        "result": content,
        "error": null
    }))
}

pub fn write_file(path: &str, content: &str) -> Result<Value> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("无法写入文件: {}", path))?;
    Ok(json!({
        "result": format!("文件已写入: {}", path),
        "error": null
    }))
}

pub fn shell_execute(command: &str) -> Result<Value> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command]).output()?
    } else {
        Command::new("sh").args(["-c", command]).output()?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = output.status.code();

    Ok(json!({
        "result": {
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": status
        },
        "error": null
    }))
}
