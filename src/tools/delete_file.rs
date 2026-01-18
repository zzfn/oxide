use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn delete_file(path: &str) -> Result<Value> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("文件不存在: {}", path.display());
    }

    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("无法删除目录: {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("无法删除文件: {}", path.display()))?;
    }

    Ok(json!({
        "result": format!("已删除: {}", path.display()),
        "error": null
    }))
}
