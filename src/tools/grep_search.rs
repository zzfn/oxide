use super::FileToolError;
use colored::*;
use grep_regex::RegexMatcher;
use grep_searcher::{
    Searcher,
    SearcherBuilder,
    Sink,
    SinkMatch,
};
use ignore::WalkBuilder;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GrepSearchArgs {
    pub root_path: String,
    pub query: String,
    pub max_results: Option<usize>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SearchMatch {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

// 用于收集单个文件搜索结果的 sink
struct FileCollector {
    matches: Vec<(usize, String)>,
    max_results: usize,
}

impl Sink for FileCollector {
    type Error = std::io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch) -> Result<bool, Self::Error> {
        if self.matches.len() >= self.max_results {
            return Ok(false);
        }

        let line_content = String::from_utf8_lossy(mat.bytes()).to_string();
        let line_number = mat.line_number().unwrap_or(1) as usize;

        self.matches.push((line_number, line_content));
        Ok(true)
    }
}

#[derive(Serialize, Debug)]
pub struct GrepSearchOutput {
    pub root_path: String,
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub total_matches: usize,
    pub files_searched: usize,
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct GrepSearchTool;

impl Tool for GrepSearchTool {
    const NAME: &'static str = "grep_search";

    type Error = FileToolError;
    type Args = GrepSearchArgs;
    type Output = GrepSearchOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "grep_search".to_string(),
            description: "Search for text patterns in files using regex. Respects .gitignore automatically.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "root_path": {"type": "string", "description": "Root directory to search"},
                    "query": {"type": "string", "description": "Regex pattern to search for"},
                    "max_results": {"type": "integer", "description": "Max matches (default: 100)", "default": 100}
                },
                "required": ["root_path", "query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let max_results = args.max_results.unwrap_or(100);

        // 使用 ripgrep 的 RegexMatcher
        let matcher = RegexMatcher::new(&args.query)
            .map_err(|e| FileToolError::InvalidInput(format!("Invalid regex: {}", e)))?;

        let mut all_matches = Vec::new();
        let mut files_searched = 0;

        // 使用 ignore crate 遍历文件
        for result in WalkBuilder::new(&args.root_path)
            .hidden(false)
            .git_ignore(true)
            .build()
        {
            if all_matches.len() >= max_results {
                break;
            }

            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                files_searched += 1;

                // 为每个文件创建 collector
                let remaining = max_results - all_matches.len();
                let mut collector = FileCollector {
                    matches: Vec::new(),
                    max_results: remaining,
                };

                // 使用 grep-searcher 搜索单个文件
                let mut searcher = SearcherBuilder::new().build();
                if searcher.search_path(&matcher, entry.path(), &mut collector).is_ok() {
                    // 将结果转换为 SearchMatch
                    for (line_num, line_content) in collector.matches {
                        let content_len = line_content.len();
                        all_matches.push(SearchMatch {
                            file_path: entry.path().to_string_lossy().to_string(),
                            line_number: line_num,
                            line_content,
                            match_start: 0,
                            match_end: content_len,
                        });

                        if all_matches.len() >= max_results {
                            break;
                        }
                    }
                }
            }
        }

        let message = format!(
            "Found {} match{} in {} files",
            all_matches.len(),
            if all_matches.len() == 1 { "" } else { "es" },
            files_searched
        );

        Ok(GrepSearchOutput {
            root_path: args.root_path,
            query: args.query,
            total_matches: all_matches.len(),
            matches: all_matches,
            files_searched,
            success: true,
            message,
        })
    }
}

// Wrapper with visual feedback
#[derive(Deserialize, Serialize)]
pub struct WrappedGrepSearchTool {
    inner: GrepSearchTool,
}

impl WrappedGrepSearchTool {
    pub fn new() -> Self {
        Self {
            inner: GrepSearchTool,
        }
    }
}

impl Tool for WrappedGrepSearchTool {
    const NAME: &'static str = "grep_search";
    type Error = FileToolError;
    type Args = <GrepSearchTool as Tool>::Args;
    type Output = <GrepSearchTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("{} {}({})", "●".bright_green(), "Search", args.query);

        let result = self.inner.call(args).await;

        match &result {
            Ok(output) => {
                if output.total_matches > 0 {
                    let preview = &output.matches[0].line_content;
                    let preview = if preview.len() > 50 {
                        format!("{}...", &preview[..50])
                    } else {
                        preview.clone()
                    };
                    println!(
                        "  └─ {} ... +{} matches",
                        preview.dimmed(),
                        output.total_matches
                    );
                } else {
                    println!("  └─ {}", "No matches found".dimmed());
                }
            }
            Err(e) => println!("  └─ {}", format!("Error: {}", e).red()),
        }

        result
    }
}
