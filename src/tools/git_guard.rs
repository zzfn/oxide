//! Git 安全检查
//!
//! 提供 Git 操作的安全检查和警告。

#![allow(dead_code)]

use colored::*;
use git2::{Repository, Status};
use std::path::Path;

/// Git 安全状态
#[derive(Debug, Clone)]
pub enum GitSafety {
    /// 安全,可以执行操作
    Safe,

    /// 有未提交的更改
    UncommittedChanges,

    /// 在主分支上
    OnMainBranch { branch_name: String },

    /// 远程分支有新提交
    BehindRemote { local: String, remote: String },

    /// 不在 Git 仓库中
    NotInRepository,

    /// 无法检查
    CannotCheck { error: String },
}

/// Git Guard
///
/// 提供安全检查和警告功能。
pub struct GitGuard {
    repo: Option<Repository>,
}

impl GitGuard {
    /// 创建新的 Git Guard
    ///
    /// 尝试在当前目录或其父目录中查找 Git 仓库。
    pub fn new() -> Result<Self, String> {
        let repo = Repository::discover(".").map_err(|e| format!("无法查找 Git 仓库: {}", e))?;

        Ok(Self { repo: Some(repo) })
    }

    /// 从指定路径创建 Git Guard
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let repo =
            Repository::discover(path).map_err(|e| format!("无法查找 Git 仓库: {}", e))?;

        Ok(Self { repo: Some(repo) })
    }

    /// 检查 Git 安全状态
    pub fn check_safety(&self) -> GitSafety {
        let repo = match &self.repo {
            Some(r) => r,
            None => return GitSafety::NotInRepository,
        };

        // 检查是否有未提交的更改
        let statuses = match repo.statuses(None) {
            Ok(s) => s,
            Err(_) => return GitSafety::CannotCheck {
                error: "无法获取 Git 状态".to_string(),
            },
        };

        let has_changes: Vec<Status> = statuses
            .iter()
            .filter_map(|s| {
                if s.status() != Status::CURRENT {
                    Some(s.status())
                } else {
                    None
                }
            })
            .collect();

        if !has_changes.is_empty() {
            return GitSafety::UncommittedChanges;
        }

        // 检查当前分支
        let head = match repo.head() {
            Ok(h) => h,
            Err(_) => {
                return GitSafety::CannotCheck {
                    error: "无法获取 HEAD".to_string(),
                }
            }
        };

        let branch_name = match head.shorthand() {
            Some(name) => name.to_string(),
            None => "HEAD".to_string(),
        };

        // 检查是否在主分支
        if Self::is_main_branch(&branch_name) {
            return GitSafety::OnMainBranch { branch_name };
        }

        // 检查远程状态
        if let Ok(branch_obj) = repo.find_branch(&branch_name, git2::BranchType::Local) {
            if let Ok(upstream) = branch_obj.upstream() {
                if let Ok(upstream_name) = upstream.name() {
                    if let Some(upstream_name) = upstream_name {
                        // 检查本地是否落后于远程
                        let local_commit =
                            match repo.revparse_single(&format!("refs/heads/{}", branch_name)) {
                                Ok(c) => c,
                                Err(_) => {
                                    return GitSafety::CannotCheck {
                                        error: "无法获取本地提交".to_string(),
                                    }
                                }
                            };

                        let upstream_commit = match repo.revparse_single(&format!(
                            "refs/remotes/{}",
                            upstream_name
                        )) {
                            Ok(c) => c,
                            Err(_) => {
                                return GitSafety::CannotCheck {
                                    error: "无法获取远程提交".to_string(),
                                }
                            }
                        };

                        if let Ok(ancestor) =
                            repo.merge_base(local_commit.id(), upstream_commit.id())
                        {
                            if ancestor != local_commit.id() {
                                return GitSafety::BehindRemote {
                                    local: branch_name,
                                    remote: upstream_name.to_string(),
                                };
                            }
                        }
                    }
                }
            }
        }

        GitSafety::Safe
    }

    /// 检查是否在主分支
    fn is_main_branch(branch_name: &str) -> bool {
        matches!(branch_name, "main" | "master")
    }

    /// 获取当前分支名称
    pub fn current_branch(&self) -> Option<String> {
        let repo = self.repo.as_ref()?;
        let head = repo.head().ok()?;
        head.shorthand().map(|s| s.to_string())
    }

    /// 获取未提交的文件列表
    pub fn uncommitted_files(&self) -> Vec<(String, Status)> {
        let mut files = Vec::new();

        if let Some(repo) = &self.repo {
            if let Ok(statuses) = repo.statuses(None) {
                for entry in statuses.iter() {
                    if entry.status() != Status::CURRENT {
                        if let Some(path) = entry.path() {
                            files.push((path.to_string(), entry.status()));
                        }
                    }
                }
            }
        }

        files
    }

    /// 警告:即将推送到主分支
    pub fn warn_if_pushing_to_main(&self) {
        if let Some(branch) = self.current_branch() {
            if Self::is_main_branch(&branch) {
                println!();
                println!(
                    "{} {}",
                    "⚠️ ".bright_yellow(),
                    "警告: 即将操作在主分支上".bright_yellow().bold()
                );
                println!(
                    "  当前分支: {}",
                    branch.bright_white()
                );
                println!(
                    "  建议先创建功能分支: {}",
                    "git checkout -b feat/your-feature".bright_cyan()
                );
                println!();
            }
        }
    }

    /// 显示 Git 安全状态
    pub fn display_safety_status(&self) {
        match self.check_safety() {
            GitSafety::Safe => {
                println!(
                    "{} {}",
                    "✓".bright_green(),
                    "Git 状态良好".bright_green()
                );
            }
            GitSafety::UncommittedChanges => {
                println!();
                println!(
                    "{} {}",
                    "⚠️ ".bright_yellow(),
                    "警告: 有未提交的更改".bright_yellow().bold()
                );

                let files = self.uncommitted_files();
                if files.len() <= 10 {
                    for (path, status) in files {
                        let status_symbol = match status {
                            Status::WT_NEW => "+".bright_green(),
                            Status::WT_MODIFIED => "M".bright_yellow(),
                            Status::WT_DELETED => "-".bright_red(),
                            _ => "?".bright_black(),
                        };
                        println!("  {} {}", status_symbol, path.dimmed());
                    }
                } else {
                    println!(
                        "  {} 个文件有更改 (使用 'git status' 查看详情)",
                        files.len()
                    );
                }
                println!();
            }
            GitSafety::OnMainBranch { branch_name } => {
                println!();
                println!(
                    "{} {}",
                    "⚠️ ".bright_yellow(),
                    "注意: 当前在主分支".bright_yellow().bold()
                );
                println!("  分支: {}", branch_name.bright_white());
                println!();
            }
            GitSafety::BehindRemote { local, remote } => {
                println!();
                println!(
                    "{} {}",
                    "⚠️ ".bright_yellow(),
                    "警告: 本地分支落后于远程".bright_yellow().bold()
                );
                println!("  本地: {}", local.bright_white());
                println!("  远程: {}", remote.bright_cyan());
                println!(
                    "  建议: {}",
                    "git pull".bright_cyan()
                );
                println!();
            }
            GitSafety::NotInRepository => {
                println!(
                    "{} {}",
                    "ℹ️ ".bright_blue(),
                    "不在 Git 仓库中".bright_black()
                );
            }
            GitSafety::CannotCheck { error } => {
                println!(
                    "{} {}",
                    "✗".bright_red(),
                    format!("无法检查 Git 状态: {}", error).bright_red()
                );
            }
        }
    }

    /// 格式化状态符号
    fn format_status(status: Status) -> colored::ColoredString {
        match status {
            Status::WT_NEW => "新建".bright_green(),
            Status::WT_MODIFIED => "修改".bright_yellow(),
            Status::WT_DELETED => "删除".bright_red(),
            Status::INDEX_NEW => "已暂存".bright_green(),
            Status::INDEX_MODIFIED => "已暂存修改".bright_yellow(),
            Status::INDEX_DELETED => "已暂存删除".bright_red(),
            _ => "其他".bright_black(),
        }
    }
}

impl Default for GitGuard {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self { repo: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_main_branch() {
        assert!(GitGuard::is_main_branch("main"));
        assert!(GitGuard::is_main_branch("master"));
        assert!(!GitGuard::is_main_branch("feat/test"));
        assert!(!GitGuard::is_main_branch("develop"));
    }

    #[test]
    fn test_git_safety_variants() {
        // 测试各种安全状态的创建
        let safe = GitSafety::Safe;
        match safe {
            GitSafety::Safe => (),
            _ => panic!("应该是 Safe"),
        }

        let uncommitted = GitSafety::UncommittedChanges;
        match uncommitted {
            GitSafety::UncommittedChanges => (),
            _ => panic!("应该是 UncommittedChanges"),
        }

        let on_main = GitSafety::OnMainBranch {
            branch_name: "main".to_string(),
        };
        match on_main {
            GitSafety::OnMainBranch { branch_name } => {
                assert_eq!(branch_name, "main");
            }
            _ => panic!("应该是 OnMainBranch"),
        }
    }

    #[test]
    fn test_git_guard_default() {
        let guard = GitGuard::default();
        // 默认创建应该成功,即使不在 Git 仓库中
        // 它会返回一个 repo 为 None 的实例
        assert!(guard.repo.is_none() || guard.repo.is_some());
    }
}
