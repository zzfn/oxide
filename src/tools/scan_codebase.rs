use anyhow::Result;
use serde_json::{json, Value};
use walkdir::WalkDir;

pub fn scan_codebase(path: &str, max_depth: Option<usize>) -> Result<Value> {
    let max_depth = max_depth.unwrap_or(10);

    let mut tree = Vec::new();

    for entry in WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let display_path = path.display().to_string();

        let entry_type = if path.is_dir() {
            "directory"
        } else if path.is_file() {
            "file"
        } else {
            "other"
        };

        tree.push(json!({
            "path": display_path,
            "type": entry_type,
            "depth": entry.depth()
        }));
    }

    Ok(json!({
        "result": {
            "base_path": path,
            "entries": tree,
            "total_entries": tree.len()
        },
        "error": null
    }))
}
