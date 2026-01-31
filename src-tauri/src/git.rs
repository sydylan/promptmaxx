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
