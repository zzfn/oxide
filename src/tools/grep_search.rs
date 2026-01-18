use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::fs;
use walkdir::{DirEntry, WalkDir};

pub fn grep_search(root_path: &str, query: &str, max_results: Option<usize>) -> Result<Value> {
    let max_results = max_results.unwrap_or(100);

    let regex = Regex::new(query).with_context(|| format!("无效的正则表达式: {}", query))?;

    let mut matches = Vec::new();
    let mut files_searched = 0;

    for entry in WalkDir::new(root_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if matches.len() >= max_results {
            break;
        }

        let entry: DirEntry = entry;
        let path = entry.path();

        if path.is_file() {
            files_searched += 1;

            if let Ok(content) = fs::read_to_string(path) {
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(mat) = regex.find(line) {
                        matches.push(json!({
                            "file_path": path.display().to_string(),
                            "line_number": line_num + 1,
                            "line_content": line,
                            "match_start": mat.start(),
                            "match_end": mat.end()
                        }));

                        if matches.len() >= max_results {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "result": {
            "root_path": root_path,
            "query": query,
            "total_matches": matches.len(),
            "matches": matches,
            "files_searched": files_searched
        },
        "error": null
    }))
}
