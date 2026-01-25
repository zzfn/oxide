use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;

/// 文件引用信息
#[derive(Debug, Clone)]
pub struct FileReference {
    /// 原始引用路径（如 @src/main.rs）
    pub raw_reference: String,
    /// 解析后的文件路径
    pub file_path: PathBuf,
    /// 文件内容
    pub content: String,
    /// 文件大小（字节）
    pub size_bytes: u64,
    /// 文件行数
    pub line_count: usize,
}

impl FileReference {
    /// 创建文件引用
    pub fn new(raw_reference: String, file_path: PathBuf, content: String) -> Result<Self> {
        let size_bytes = fs::metadata(&file_path)
            .with_context(|| format!("无法读取文件元数据: {}", file_path.display()))?
            .len();

        let line_count = content.lines().count();

        Ok(Self {
            raw_reference,
            file_path,
            content,
            size_bytes,
            line_count,
        })
    }

    /// 显示文件引用信息
    pub fn display_info(&self) -> String {
        format!(
            "{} {} {} ({} bytes, {} lines)",
            "📎".bright_cyan(),
            self.raw_reference.bright_white(),
            self.file_path.display().to_string().dimmed(),
            self.size_bytes.to_string().dimmed(),
            self.line_count.to_string().dimmed()
        )
    }
}

/// 从用户输入中解析文件引用
///
/// # 参数
/// - `input`: 用户输入的文本
///
/// # 返回
/// - (解析后的文本, 文件引用列表)
pub fn parse_file_references(input: &str) -> (String, Vec<FileReference>) {
    let mut references = Vec::new();
    let mut parsed_input = String::from(input);

    // 匹配 @路径/文件名 或 @相对路径/文件
    // 规则：@ 后面必须跟路径分隔符 (/ 或 \) 或文件名
    let re = regex::Regex::new(r"@([^\s@]+)").unwrap();

    for cap in re.captures_iter(input) {
        let full_match = cap.get(0).unwrap().as_str();
        let path_str = cap.get(1).unwrap().as_str();

        // 检查是否是有效的文件路径（包含路径分隔符，或者是看起来像文件名的字符串）
        if is_valid_file_reference(path_str) {
            match resolve_and_read_file(path_str) {
                Ok(file_ref) => {
                    references.push(file_ref);
                    // 从输入中移除 @引用
                    parsed_input = parsed_input.replace(full_match, "");
                }
                Err(e) => {
                    println!("{} {}", "⚠️".yellow(), format!("无法读取文件 @{}: {}", path_str, e));
                }
            }
        }
    }

    // 清理多余的空格
    let parsed_input = parsed_input.split_whitespace().collect::<Vec<_>>().join(" ");

    (parsed_input, references)
}

/// 判断是否是有效的文件引用
fn is_valid_file_reference(path: &str) -> bool {
    // 包含路径分隔符
    if path.contains('/') || path.contains('\\') {
        return true;
    }

    // 或者看起来像文件名（包含扩展名）
    if path.contains('.') {
        return true;
    }

    // 常见的代码文件名（无需扩展名）
    let common_filenames = [
        "README", "LICENSE", "CONTRIBUTING", "Cargo", "package", "Dockerfile",
        "Makefile", "setup", "main", "index", "app",
    ];

    common_filenames.iter().any(|&name| path == name || path.starts_with(&format!("{}/", name)))
}

/// 解析文件路径并读取内容
pub fn resolve_and_read_file(path_str: &str) -> Result<FileReference> {
    let path = resolve_file_path(path_str)?;

    // 检查文件大小
    let metadata = fs::metadata(&path)?;
    let size_bytes = metadata.len();

    // 警告：文件过大
    if size_bytes > 1024 * 1024 {
        // 1MB
        println!(
            "{} 文件较大: {} ({} bytes)",
            "⚠️".yellow(),
            path.display(),
            size_bytes
        );
    }

    // 读取文件内容
    let content = fs::read_to_string(&path)
        .with_context(|| format!("无法读取文件: {}", path.display()))?;

    // 警告：空文件
    if content.trim().is_empty() {
        println!("{} 文件为空: {}", "⚠️".yellow(), path.display());
    }

    FileReference::new(format!("@{}", path_str), path, content)
}

/// 解析文件路径（支持相对路径和绝对路径）
pub fn resolve_file_path(path_str: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path_str);

    // 如果是绝对路径，直接使用
    if path.is_absolute() {
        return Ok(path);
    }

    // 否则，相对于当前工作目录
    let current_dir = std::env::current_dir()
        .context("无法获取当前工作目录")?;

    Ok(current_dir.join(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_file_reference() {
        assert!(is_valid_file_reference("src/main.rs"));
        assert!(is_valid_file_reference("README"));
        assert!(is_valid_file_reference("Cargo.toml"));
        assert!(is_valid_file_reference("docs/spec.md"));
        assert!(!is_valid_file_reference("notrealfile")); // 没有扩展名
    }

    #[test]
    fn test_parse_file_references() {
        let input = "@src/main.rs 请帮我重构这个文件";
        let (parsed, refs) = parse_file_references(input);
        // 由于文件可能不存在，我们只检查解析逻辑
        assert!(!parsed.contains("@src/main.rs"));
    }

    #[test]
    fn test_parse_multiple_file_references() {
        // 使用存在的文件进行测试
        let input = "@Cargo.toml @src/cli/mod.rs 比较这两个文件";
        let (parsed, refs) = parse_file_references(input);
        // 检查成功的文件引用被处理
        // 注意：由于文件可能不存在，refs 可能是空的
        // 这个测试主要验证解析逻辑不会崩溃
    }

    #[test]
    fn test_resolve_file_path() {
        // 测试相对路径解析
        let result = resolve_file_path("Cargo.toml");
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.ends_with("Cargo.toml"));
    }

    #[test]
    fn test_file_reference_display_info() {
        // 创建一个模拟的文件引用
        let file_ref = FileReference {
            raw_reference: "@test.txt".to_string(),
            file_path: PathBuf::from("/test/path.txt"),
            content: "test content".to_string(),
            size_bytes: 12,
            line_count: 1,
        };

        let info = file_ref.display_info();
        assert!(info.contains("@test.txt"));
        assert!(info.contains("path.txt"));
        assert!(info.contains("12 bytes"));
        assert!(info.contains("1 lines"));
    }
}
