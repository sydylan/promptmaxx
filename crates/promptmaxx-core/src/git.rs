use std::path::Path;
use std::process::Command;

/// Git context information
#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub repo: Option<String>,
    pub branch: Option<String>,
}

/// Get git repo and branch info from the current directory
pub fn get_git_info() -> GitInfo {
    GitInfo {
        repo: get_repo_name(),
        branch: get_branch_name(),
    }
}

/// Get git info from a specific directory
pub fn get_git_info_from_dir(dir: &Path) -> GitInfo {
    GitInfo {
        repo: get_repo_name_from_dir(dir),
        branch: get_branch_name_from_dir(dir),
    }
}

fn get_repo_name() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().split('/').last().unwrap_or("").to_string())
            } else {
                None
            }
        })
}

fn get_branch_name() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

fn get_repo_name_from_dir(dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().split('/').last().unwrap_or("").to_string())
            } else {
                None
            }
        })
}

fn get_branch_name_from_dir(dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Get recent commit messages (for context)
pub fn get_recent_commits(count: usize) -> Vec<String> {
    Command::new("git")
        .args([
            "log",
            "--oneline",
            &format!("-{}", count),
            "--pretty=format:%s",
        ])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| {
                    s.lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect()
                })
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Get git diff (staged or unstaged)
pub fn get_git_diff(staged: bool) -> Option<String> {
    let mut args = vec!["diff"];
    if staged {
        args.push("--staged");
    }
    args.push("--stat");

    Command::new("git")
        .args(&args)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let output = String::from_utf8(o.stdout).ok()?;
                if output.trim().is_empty() {
                    None
                } else {
                    Some(output)
                }
            } else {
                None
            }
        })
}
