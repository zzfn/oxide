use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;

pub fn create_directory(path: &str) -> Result<Value> {
    fs::create_dir_all(path).with_context(|| format!("无法创建目录: {}", path))?;

    Ok(json!({
        "result": format!("目录已创建: {}", path),
        "error": null
    }))
}
