use anyhow::{Context, Result};
use patch_apply::{apply, Patch};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn edit_file(file_path: &str, patch: &str) -> Result<Value> {
    let path = Path::new(file_path);

    if !path.exists() {
        anyhow::bail!("文件不存在: {}", file_path);
    }

    if !path.is_file() {
        anyhow::bail!("路径不是文件: {}", file_path);
    }

    let current_content =
        fs::read_to_string(file_path).with_context(|| format!("无法读取文件: {}", file_path))?;

    let patch_str_normalized = if !patch.ends_with('\n') {
        format!("{}\n", patch)
    } else {
        patch.to_string()
    };

    let parsed_patch = Patch::from_single(&patch_str_normalized)
        .map_err(|e| anyhow::anyhow!("无法解析 patch: {}", e))?;

    let patched_content = apply(current_content, parsed_patch);

    let mut lines_added = 0usize;
    let mut lines_removed = 0usize;

    for line in patch.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            lines_added += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            lines_removed += 1;
        }
    }

    fs::write(file_path, &patched_content)
        .with_context(|| format!("无法写入文件: {}", file_path))?;

    Ok(json!({
        "result": {
            "file_path": file_path,
            "lines_added": lines_added,
            "lines_removed": lines_removed,
            "message": format!("成功应用 patch 到 '{}': +{} 行, -{} 行", file_path, lines_added, lines_removed)
        },
        "error": null
    }))
}
