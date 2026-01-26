use super::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct SearchReplaceArgs {
    pub file_path: String,
    pub search_content: String,
    pub replace_content: String,
    // Optional: force replacement even if multiple matches found?
    // default to false for safety
    #[serde(default)]
    pub allow_multiple: bool,
}

#[derive(Serialize, Debug)]
pub struct SearchReplaceOutput {
    pub file_path: String,
    pub success: bool,
    pub message: String,
    pub replacements_count: usize,
}

#[derive(Deserialize, Serialize)]
pub struct SearchReplaceTool;

impl Tool for SearchReplaceTool {
    const NAME: &'static str = "search_replace";

    type Error = FileToolError;
    type Args = SearchReplaceArgs;
    type Output = SearchReplaceOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "search_replace".to_string(),
            description: r#"
Perform a robust search-and-replace operation on a file.
This tool is more robust than edit_file (patches) because it doesn't rely on line numbers.

Capabilities:
1. Exact matching of the search block.
2. Robust matching: If exact match fails, it tries to match by ignoring leading/trailing whitespace on lines.
   - This helps when indentation in your request is slightly off.

Rules:
- The search_content must map uniquely to a single location in the file (unless allow_multiple is true).
- If multiple matches are found, it will fail for safety.
- The replace_content will replace the found block exactly.

When to use:
- When you want to replace a block of code (function, class, configuration block).
- When you are unsure about exact line numbers.
- When edit_file fails due to context mismatch.
"#.trim().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The path to the file to edit (relative or absolute)"
                    },
                    "search_content": {
                        "type": "string",
                        "description": "The exact content block to search for. Must be unique in the file."
                    },
                    "replace_content": {
                        "type": "string",
                        "description": "The new content to replace the matched block with."
                    },
                    "allow_multiple": {
                        "type": "boolean",
                        "description": "If true, replace all occurrences if multiple are found. Default is false (safety first)."
                    }
                },
                "required": ["file_path", "search_content", "replace_content"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.file_path);
        
        if !path.exists() {
            return Err(FileToolError::FileNotFound(args.file_path.clone()));
        }
        if !path.is_file() {
            return Err(FileToolError::NotAFile(args.file_path.clone()));
        }

        let content = fs::read_to_string(&args.file_path)?;
        
        // Strategy 1: Exact Match
        let (matches, strategy) = if content.contains(&args.search_content) {
            (
                content.match_indices(&args.search_content)
                    .map(|(start, matched_str)| (start, start + matched_str.len()))
                    .collect::<Vec<(usize, usize)>>(),
                "exact"
            )
        } else {
            // Strategy 2: Robust Match (Ignore indentation/line variations)
            match find_robust_matches(&content, &args.search_content) {
                Some(indices) if !indices.is_empty() => (indices, "robust"),
                _ => (Vec::new(), "none")
            }
        };

        if matches.is_empty() {
            return Err(FileToolError::InvalidInput(format!(
                "Could not find the search content in '{}'. Tried exact match and robust match (ignoring indentation).", 
                args.file_path
            )));
        }

        if matches.len() > 1 && !args.allow_multiple {
            return Err(FileToolError::InvalidInput(format!(
                "Found {} occurrences of the search content in '{}'. Please provide more context to make the search unique, or set allow_multiple=true (careful!).", 
                matches.len(), args.file_path
            )));
        }

        // Apply replacements
        // Sort ranges desc to replace without shifting indices
        let mut sorted_ranges = matches;
        let replacements_count = sorted_ranges.len();
        sorted_ranges.sort_by(|a, b| b.0.cmp(&a.0));

        let mut new_content = content.clone();
        for (start, end) in sorted_ranges {
            new_content.replace_range(start..end, &args.replace_content);
        }

        fs::write(&args.file_path, &new_content)?;

        Ok(SearchReplaceOutput {
            file_path: args.file_path,
            success: true,
            message: format!("Successfully replaced {} occurrence(s) using {} matching.", replacements_count, strategy),
            replacements_count,
        })
    }
}

/// Helper to find robust matches
/// Returns vector of (start_byte_index, end_byte_index) in original content
fn find_robust_matches(file_content: &str, search_block: &str) -> Option<Vec<(usize, usize)>> {
    let file_lines: Vec<&str> = file_content.lines().collect();
    let search_lines: Vec<&str> = search_block.lines().collect();
    
    // We need to map lines back to byte indices in file_content
    // Build a map of line_index -> (start_byte, end_byte_including_newline)
    let mut line_indices = Vec::with_capacity(file_lines.len());
    let mut current_byte = 0;
    for line in file_content.split_inclusive('\n') {
        line_indices.push((current_byte, current_byte + line.len()));
        current_byte += line.len();
    }
    
    // If file ends without newline, split_inclusive might behave differently?
    // split_inclusive preserves the delimiter. 
    // If last line has no \n, it works.
    
    if search_lines.is_empty() {
        return None;
    }

    let mut found_ranges = Vec::new();

    // Iterate through file lines
    for i in 0..file_lines.len() {
        if i + search_lines.len() > file_lines.len() {
            break;
        }

        // Check if block matches starting at line i
        let mut match_found = true;
        for j in 0..search_lines.len() {
            if file_lines[i + j].trim() != search_lines[j].trim() {
                match_found = false;
                break;
            }
        }

        if match_found {
            // Calculate byte range
            let start_line_idx = i;
            let end_line_idx = i + search_lines.len() - 1; // inclusive
            
            // Safety check
            if start_line_idx >= line_indices.len() || end_line_idx >= line_indices.len() {
                continue; 
            }

            let start_byte = line_indices[start_line_idx].0;
            // The match includes the newline of the last line? 
            // Usually search_replace expects to replace the whole lines.
            // If the user's search_content provided includes newlines, we should match inclusive.
            // However, `lines()` strips newlines usually? No, `lines()` strips.
            
            // We'll take the range covering all these lines in original content.
            // Note: `line_indices` stores ranges including newlines.
            
            // But wait, if search_content didn't have a trailing newline, but the file does?
            // "Robust" implies we replace the *lines*.
            
            // Let's use the end of the last matched line.
            let end_byte = line_indices[end_line_idx].1;
            
            found_ranges.push((start_byte, end_byte));
        }
    }
    
    if found_ranges.is_empty() {
        None
    } else {
        Some(found_ranges)
    }
}

// Wrapper for Tool trait just in case we need it (like WrappedEditFileTool)
#[derive(Deserialize, Serialize)]
pub struct WrappedSearchReplaceTool {
    inner: SearchReplaceTool,
}

impl WrappedSearchReplaceTool {
    pub fn new() -> Self {
        Self { inner: SearchReplaceTool }
    }
}

impl Tool for WrappedSearchReplaceTool {
    const NAME: &'static str = "search_replace";

    type Error = FileToolError;
    type Args = SearchReplaceArgs;
    type Output = SearchReplaceOutput;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!();
        println!("{} {}({})", "●".bright_green(), "SearchReplace", args.file_path);
        
        let result = self.inner.call(args).await;
        
        match &result {
            Ok(output) => {
                 println!(
                    "  └─ {} (replaced {} block(s))",
                    "Success".green(),
                    output.replacements_count
                );
            }
            Err(e) => {
                println!("  └─ {}", format!("Error: {}", e).red());
            }
        }
        println!();
        result
    }
}
